//! Local-first Markdown vault, derived search index, and recoverable history.

use rusqlite::{Connection, params};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("note not found: {0}")]
    NotFound(String),
    #[error("invalid note metadata: {0}")]
    InvalidNote(String),
}

pub type Result<T> = std::result::Result<T, VaultError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub folder: String,
    pub tags: Vec<String>,
    pub created: u64,
    pub updated: u64,
    pub body: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct Vault {
    root: PathBuf,
    notes_dir: PathBuf,
    history_dir: PathBuf,
    trash_dir: PathBuf,
    index_path: PathBuf,
}

impl Vault {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let notes_dir = root.join("notes");
        let state_dir = root.join(".cybernote");
        let history_dir = state_dir.join("history");
        let trash_dir = state_dir.join("trash");
        fs::create_dir_all(&notes_dir)?;
        fs::create_dir_all(&history_dir)?;
        fs::create_dir_all(&trash_dir)?;
        fs::create_dir_all(root.join("attachments"))?;
        let vault = Self {
            index_path: state_dir.join("index.sqlite3"),
            root,
            notes_dir,
            history_dir,
            trash_dir,
        };
        vault.initialize_index()?;
        vault.rebuild_index()?;
        Ok(vault)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn connect(&self) -> Result<Connection> {
        let db = Connection::open(&self.index_path)?;
        db.busy_timeout(std::time::Duration::from_secs(3))?;
        Ok(db)
    }

    fn initialize_index(&self) -> Result<()> {
        let db = self.connect()?;
        db.execute_batch("PRAGMA journal_mode=WAL;
            CREATE VIRTUAL TABLE IF NOT EXISTS note_search USING fts5(id UNINDEXED, title, body, tags, tokenize='unicode61 remove_diacritics 2');")?;
        Ok(())
    }

    fn rebuild_index(&self) -> Result<()> {
        let db = self.connect()?;
        db.execute("DELETE FROM note_search", [])?;
        for path in markdown_files(&self.notes_dir)? {
            if let Ok(note) = read_note(&path) {
                db.execute(
                    "INSERT INTO note_search (id, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
                    params![note.id, note.title, note.body, note.tags.join(" ")],
                )?;
            }
        }
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Note>> {
        let mut notes = Vec::new();
        for path in markdown_files(&self.notes_dir)? {
            if let Ok(note) = read_note(&path) {
                notes.push(note);
            }
        }
        notes.sort_by(|a, b| {
            b.updated
                .cmp(&a.updated)
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });
        Ok(notes)
    }

    /// Import the previous Python TUI's four Markdown sections without
    /// changing the legacy files. The marker makes the operation one-time.
    pub fn import_legacy_sections(&self, legacy_root: &Path) -> Result<usize> {
        let marker = self.root.join(".cybernote").join("legacy-import-v1.done");
        if marker.exists() || !self.list()?.is_empty() {
            return Ok(0);
        }
        let sections = [
            ("Notes.md", "notes"),
            ("Todo.md", "todo"),
            ("Projects.md", "projects"),
            ("Timeline.md", "timeline"),
        ];
        let mut imported = 0;
        for (filename, category) in sections {
            let source = legacy_root.join(filename);
            if !source.is_file() {
                continue;
            }
            let content = fs::read_to_string(source)?;
            let mut current: Option<(String, Vec<String>)> = None;
            for line in content.lines() {
                if let Some(title) = line.strip_prefix("### ") {
                    if let Some((title, body)) = current.take() {
                        self.create(&title, &body.join("\n"), vec![category.into()])?;
                        imported += 1;
                    }
                    current = Some((title.trim().to_owned(), Vec::new()));
                } else if let Some((_, body)) = current.as_mut() {
                    body.push(line.to_owned());
                }
            }
            if let Some((title, body)) = current {
                self.create(&title, &body.join("\n"), vec![category.into()])?;
                imported += 1;
            }
        }
        fs::write(marker, format!("Imported {imported} legacy entries.\n"))?;
        Ok(imported)
    }

    pub fn get(&self, id: &str) -> Result<Note> {
        read_note(&self.find_note_path(id)?)
    }

    pub fn create(&self, title: &str, body: &str, tags: Vec<String>) -> Result<Note> {
        let id = Uuid::new_v4().simple().to_string();
        self.save_in_folder(&id, title, body, tags, "", None)
    }

    pub fn create_folder(&self, folder: &str) -> Result<String> {
        let folder = validate_folder(folder)?;
        if folder.is_empty() {
            return Err(VaultError::InvalidNote(
                "folder name cannot be empty".into(),
            ));
        }
        fs::create_dir_all(self.notes_dir.join(&folder))?;
        Ok(folder)
    }

    pub fn folders(&self) -> Result<Vec<String>> {
        let mut folders = Vec::new();
        collect_folders(&self.notes_dir, &self.notes_dir, &mut folders)?;
        folders.sort();
        Ok(folders)
    }

    pub fn move_note_to_folder(&self, id: &str, folder: &str) -> Result<Note> {
        let folder = validate_folder(folder)?;
        let note = self.get(id)?;
        if note.folder == folder {
            return Ok(note);
        }
        let original_path = note.path.clone();
        if !folder.is_empty() {
            fs::create_dir_all(self.notes_dir.join(&folder))?;
        }
        let moved = self.save_in_folder(
            &note.id,
            &note.title,
            &note.body,
            note.tags.clone(),
            &folder,
            Some(note.updated),
        )?;
        fs::remove_file(original_path)?;
        Ok(moved)
    }

    pub fn save(
        &self,
        id: &str,
        title: &str,
        body: &str,
        tags: Vec<String>,
        expected_updated: Option<u64>,
    ) -> Result<Note> {
        let folder = match self.find_note_path(id) {
            Ok(path) => read_note(&path)?.folder,
            Err(VaultError::NotFound(_)) => String::new(),
            Err(error) => return Err(error),
        };
        self.save_in_folder(id, title, body, tags, &folder, expected_updated)
    }

    fn save_in_folder(
        &self,
        id: &str,
        title: &str,
        body: &str,
        tags: Vec<String>,
        folder: &str,
        expected_updated: Option<u64>,
    ) -> Result<Note> {
        let folder = validate_folder(folder)?;
        let path = self.notes_dir.join(&folder).join(format!("{id}.md"));
        let prior = self
            .find_note_path(id)
            .ok()
            .map(|prior_path| read_note(&prior_path))
            .transpose()?;
        if let (Some(expected), Some(old)) = (expected_updated, prior.as_ref())
            && expected != old.updated
        {
            return Err(VaultError::InvalidNote(format!(
                "note changed since it was opened (expected {expected}, found {})",
                old.updated
            )));
        }
        let wall_time = epoch_millis();
        let created = prior.as_ref().map(|note| note.created).unwrap_or(wall_time);
        let now = prior.as_ref().map_or(wall_time, |note| {
            wall_time.max(note.updated.saturating_add(1))
        });
        if let Some(old) = prior.as_ref() {
            self.snapshot(old)?;
        }
        let normalized_tags = tags
            .into_iter()
            .map(|tag| tag.trim().trim_start_matches('#').to_lowercase())
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>();
        let note = Note {
            id: id.to_owned(),
            title: title.trim().to_owned(),
            folder,
            tags: normalized_tags,
            created,
            updated: now,
            body: body.trim().to_owned(),
            path,
        };
        if let Some(parent) = note.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&note.path, serialize_note(&note))?;
        let db = self.connect()?;
        db.execute("DELETE FROM note_search WHERE id = ?1", params![note.id])?;
        db.execute(
            "INSERT INTO note_search (id, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
            params![note.id, note.title, note.body, note.tags.join(" ")],
        )?;
        Ok(note)
    }

    fn find_note_path(&self, id: &str) -> Result<PathBuf> {
        let expected = format!("{id}.md");
        for path in markdown_files(&self.notes_dir)? {
            if path
                .file_name()
                .is_some_and(|name| name.to_string_lossy() == expected)
                && read_note(&path).is_ok_and(|note| note.id == id)
            {
                return Ok(path);
            }
        }
        Err(VaultError::NotFound(id.into()))
    }

    pub fn search(&self, query: &str) -> Result<Vec<Note>> {
        let query = query
            .split_whitespace()
            .map(|word| format!("\"{}\"*", word.trim_start_matches('#').replace('"', "")))
            .collect::<Vec<_>>()
            .join(" AND ");
        if query.is_empty() {
            return self.list();
        }
        let db = self.connect()?;
        let mut stmt = db.prepare(
            "SELECT id FROM note_search WHERE note_search MATCH ?1 ORDER BY rank LIMIT 200",
        )?;
        let ids = stmt
            .query_map([query], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ids.into_iter()
            .filter_map(|id| self.get(&id).ok())
            .collect::<Vec<_>>()
            .pipe(Ok)
    }

    pub fn backlinks(&self, note: &Note) -> Result<Vec<Note>> {
        let needles = [format!("[[{}]]", note.id), format!("[[{}]]", note.title)];
        Ok(self
            .list()?
            .into_iter()
            .filter(|other| other.id != note.id && needles.iter().any(|n| other.body.contains(n)))
            .collect())
    }

    pub fn attach(&self, note_id: &str, source: &Path) -> Result<PathBuf> {
        let note = self.get(note_id)?;
        let filename = source
            .file_name()
            .ok_or_else(|| VaultError::InvalidNote("attachment has no filename".into()))?;
        let destination_dir = self.root.join("attachments").join(note_id);
        fs::create_dir_all(&destination_dir)?;
        let destination = destination_dir.join(filename);
        let destination = if destination.exists() {
            destination_dir.join(format!(
                "{}-{}.{}",
                source.file_stem().unwrap_or_default().to_string_lossy(),
                Uuid::new_v4().simple(),
                source.extension().unwrap_or_default().to_string_lossy()
            ))
        } else {
            destination
        };
        fs::copy(source, &destination)?;
        let relative = format!("../attachments/{note_id}/{}", filename.to_string_lossy());
        let body = format!(
            "{}\n\n📎 [{}]({})",
            note.body,
            filename.to_string_lossy(),
            relative
        );
        self.save(
            &note.id,
            &note.title,
            &body,
            note.tags.clone(),
            Some(note.updated),
        )?;
        Ok(destination)
    }

    pub fn trash(&self, id: &str) -> Result<()> {
        let note = self.get(id)?;
        let destination = self
            .trash_dir
            .join(format!("{}-{}.md", epoch_seconds(), id));
        fs::rename(&note.path, destination)?;
        let db = self.connect()?;
        db.execute("DELETE FROM note_search WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn restore_latest_trash(&self) -> Result<Option<Note>> {
        let mut candidates = fs::read_dir(&self.trash_dir)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
            .collect::<Vec<_>>();
        candidates.sort();
        let Some(source) = candidates.pop() else {
            return Ok(None);
        };
        let note = read_note(&source)?;
        let destination = self
            .notes_dir
            .join(validate_folder(&note.folder)?)
            .join(format!("{}.md", note.id));
        if destination.exists() {
            return Err(VaultError::InvalidNote(format!(
                "note {} already exists in the vault",
                note.id
            )));
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(source, &destination)?;
        let restored = read_note(&destination)?;
        let db = self.connect()?;
        db.execute(
            "INSERT INTO note_search (id, title, body, tags) VALUES (?1, ?2, ?3, ?4)",
            params![
                restored.id,
                restored.title,
                restored.body,
                restored.tags.join(" ")
            ],
        )?;
        Ok(Some(restored))
    }

    fn snapshot(&self, note: &Note) -> Result<()> {
        let note_history = self.history_dir.join(&note.id);
        fs::create_dir_all(&note_history)?;
        let destination =
            note_history.join(format!("{}-{}.md", note.updated, Uuid::new_v4().simple()));
        if !destination.exists() {
            fs::copy(&note.path, destination)?;
        }
        Ok(())
    }
}

fn epoch_seconds() -> u64 {
    epoch_millis()
}

fn epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn validate_folder(folder: &str) -> Result<String> {
    let folder = folder.trim();
    if folder.is_empty() {
        return Ok(String::new());
    }
    let path = Path::new(folder);
    let mut normalized = PathBuf::new();
    for component in path.components() {
        let std::path::Component::Normal(component) = component else {
            return Err(VaultError::InvalidNote(format!(
                "invalid vault folder path: {folder}"
            )));
        };
        normalized.push(component);
    }
    if path.is_absolute() {
        return Err(VaultError::InvalidNote(format!(
            "invalid vault folder path: {folder}"
        )));
    }
    Ok(normalized.to_string_lossy().into_owned())
}

fn markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_markdown_files(root, &mut files)?;
    Ok(files)
}

fn collect_markdown_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_markdown_files(&entry.path(), files)?;
        } else if file_type.is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "md")
        {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn collect_folders(root: &Path, directory: &Path, folders: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path);
        folders.push(relative.to_string_lossy().into_owned());
        collect_folders(root, &path, folders)?;
    }
    Ok(())
}

fn serialize_note(note: &Note) -> String {
    let tags = note
        .tags
        .iter()
        .map(|tag| format!("#{tag}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "---\nid: {}\ntitle: {}\nfolder: {}\ntags: {}\ncreated: {}\nupdated: {}\n---\n\n{}\n",
        note.id,
        note.title.replace('\n', " "),
        note.folder,
        tags,
        note.created,
        note.updated,
        note.body.trim()
    )
}

fn read_note(path: &Path) -> Result<Note> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return Err(VaultError::InvalidNote(path.display().to_string()));
    }
    let mut id = None;
    let mut title = None;
    let mut folder = String::new();
    let mut tags = Vec::new();
    let mut created = 0;
    let mut updated = 0;
    let mut in_body = false;
    let mut body_lines = Vec::new();
    for line in lines {
        if !in_body && line == "---" {
            in_body = true;
            continue;
        }
        if in_body {
            body_lines.push(line);
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim();
            match key.trim() {
                "id" => id = Some(value.to_owned()),
                "title" => title = Some(value.to_owned()),
                "folder" => folder = validate_folder(value)?,
                "tags" => {
                    tags = value
                        .split_whitespace()
                        .map(|tag| tag.trim_start_matches('#').to_owned())
                        .filter(|tag| !tag.is_empty())
                        .collect()
                }
                "created" => created = value.parse().unwrap_or_default(),
                "updated" => updated = value.parse().unwrap_or_default(),
                _ => {}
            }
        }
    }
    let id = id.unwrap_or_else(|| {
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    });
    Ok(Note {
        id,
        title: title.unwrap_or_else(|| "Untitled".into()),
        folder,
        tags,
        created,
        updated,
        body: body_lines.join("\n").trim().to_owned(),
        path: path.to_path_buf(),
    })
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}
impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_notes_search_tags_backlinks_and_history_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::open(dir.path()).unwrap();
        let source = vault
            .create("Rust notes", "Learning [[Cybernote]]", vec!["Rust".into()])
            .unwrap();
        let linked = vault
            .create(
                "Cybernote",
                "A local-first notebook",
                vec!["project".into()],
            )
            .unwrap();
        assert_eq!(vault.search("local-first").unwrap().len(), 1);
        assert_eq!(vault.backlinks(&linked).unwrap()[0].id, source.id);
        vault
            .save(
                &source.id,
                &source.title,
                "Updated body",
                source.tags.clone(),
                Some(source.updated),
            )
            .unwrap();
        assert_eq!(
            fs::read_dir(dir.path().join(".cybernote/history").join(&source.id))
                .unwrap()
                .count(),
            1
        );
        assert_eq!(vault.get(&source.id).unwrap().body, "Updated body");
    }

    #[test]
    fn stale_editor_cannot_overwrite_newer_note() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::open(dir.path()).unwrap();
        let original = vault.create("Title", "first", vec![]).unwrap();
        vault
            .save(
                &original.id,
                "Title",
                "second",
                vec![],
                Some(original.updated),
            )
            .unwrap();
        assert!(
            vault
                .save(
                    &original.id,
                    "Title",
                    "stale",
                    vec![],
                    Some(original.updated)
                )
                .is_err()
        );
    }

    #[test]
    fn legacy_sections_import_without_mutating_source() {
        let legacy = tempfile::tempdir().unwrap();
        fs::write(
            legacy.path().join("Notes.md"),
            "# Notes\n\n### Field note\n\nKeep this idea.\n",
        )
        .unwrap();
        let destination = tempfile::tempdir().unwrap();
        let vault = Vault::open(destination.path()).unwrap();
        assert_eq!(vault.import_legacy_sections(legacy.path()).unwrap(), 1);
        let imported = vault.list().unwrap();
        assert_eq!(imported[0].title, "Field note");
        assert_eq!(imported[0].body, "Keep this idea.");
        assert!(legacy.path().join("Notes.md").exists());
        assert_eq!(vault.import_legacy_sections(legacy.path()).unwrap(), 0);
    }

    #[test]
    fn trash_can_be_restored_with_index_rebuilt() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::open(dir.path()).unwrap();
        let note = vault
            .create("Keep me", "A recoverable thought", vec!["important".into()])
            .unwrap();
        vault.trash(&note.id).unwrap();
        assert!(vault.get(&note.id).is_err());
        let restored = vault.restore_latest_trash().unwrap().unwrap();
        assert_eq!(restored.id, note.id);
        assert_eq!(vault.search("important").unwrap().len(), 1);
        assert!(vault.restore_latest_trash().unwrap().is_none());
    }

    #[test]
    fn folders_can_organize_notes_without_breaking_search_or_restore() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::open(dir.path()).unwrap();
        let note = vault
            .create("Folder test", "A searchable note", vec!["folders".into()])
            .unwrap();

        vault.create_folder("Projects/Cybernote").unwrap();
        let moved = vault
            .move_note_to_folder(&note.id, "Projects/Cybernote")
            .unwrap();
        assert_eq!(moved.folder, "Projects/Cybernote");
        assert_eq!(vault.get(&note.id).unwrap().path, moved.path);
        assert_eq!(vault.search("searchable").unwrap()[0].folder, moved.folder);
        assert_eq!(vault.folders().unwrap(), ["Projects", "Projects/Cybernote"]);

        vault.trash(&note.id).unwrap();
        let restored = vault.restore_latest_trash().unwrap().unwrap();
        assert_eq!(restored.folder, "Projects/Cybernote");
        assert_eq!(vault.get(&note.id).unwrap().body, "A searchable note");
    }

    #[test]
    fn folder_paths_cannot_escape_the_vault() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::open(dir.path()).unwrap();
        assert!(vault.create_folder("../outside").is_err());
        assert!(vault.create_folder("/tmp/outside").is_err());
    }
}
