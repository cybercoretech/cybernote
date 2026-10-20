use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use cybernote_core::{Note, Vault};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    env, fs,
    io::{self, Stdout, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

type Term = Terminal<CrosstermBackend<Stdout>>;

#[derive(Clone, Copy)]
struct Palette {
    name: &'static str,
    bg: Color,
    panel: Color,
    accent: Color,
    secondary: Color,
    text: Color,
    muted: Color,
}

const PALETTES: [Palette; 12] = [
    Palette {
        name: "CYBERCORE / Neon Cyan",
        bg: Color::Rgb(7, 12, 18),
        panel: Color::Rgb(14, 23, 32),
        accent: Color::Rgb(0, 235, 255),
        secondary: Color::Rgb(255, 60, 196),
        text: Color::Rgb(222, 244, 249),
        muted: Color::Rgb(104, 139, 151),
    },
    Palette {
        name: "CYBERCORE / Plasma",
        bg: Color::Rgb(13, 8, 24),
        panel: Color::Rgb(26, 15, 44),
        accent: Color::Rgb(188, 111, 255),
        secondary: Color::Rgb(0, 232, 207),
        text: Color::Rgb(241, 231, 255),
        muted: Color::Rgb(149, 127, 179),
    },
    Palette {
        name: "CYBERCORE / Ember",
        bg: Color::Rgb(22, 12, 9),
        panel: Color::Rgb(39, 20, 14),
        accent: Color::Rgb(255, 136, 67),
        secondary: Color::Rgb(255, 55, 115),
        text: Color::Rgb(255, 239, 224),
        muted: Color::Rgb(170, 131, 108),
    },
    Palette {
        name: "CYBERCORE / Biohazard",
        bg: Color::Rgb(9, 18, 12),
        panel: Color::Rgb(17, 32, 21),
        accent: Color::Rgb(166, 255, 72),
        secondary: Color::Rgb(0, 224, 174),
        text: Color::Rgb(231, 248, 218),
        muted: Color::Rgb(125, 157, 114),
    },
    Palette {
        name: "CYBERCORE / Aurora",
        bg: Color::Rgb(5, 15, 24),
        panel: Color::Rgb(10, 29, 39),
        accent: Color::Rgb(64, 255, 210),
        secondary: Color::Rgb(116, 126, 255),
        text: Color::Rgb(222, 250, 245),
        muted: Color::Rgb(112, 159, 162),
    },
    Palette {
        name: "CYBERCORE / Rose Quartz",
        bg: Color::Rgb(20, 9, 18),
        panel: Color::Rgb(39, 17, 34),
        accent: Color::Rgb(255, 117, 187),
        secondary: Color::Rgb(255, 205, 112),
        text: Color::Rgb(255, 235, 246),
        muted: Color::Rgb(174, 132, 157),
    },
    Palette {
        name: "CYBERCORE / Arctic",
        bg: Color::Rgb(7, 13, 24),
        panel: Color::Rgb(14, 23, 42),
        accent: Color::Rgb(113, 197, 255),
        secondary: Color::Rgb(196, 151, 255),
        text: Color::Rgb(231, 243, 255),
        muted: Color::Rgb(126, 151, 183),
    },
    Palette {
        name: "CYBERCORE / Solar Flare",
        bg: Color::Rgb(22, 13, 5),
        panel: Color::Rgb(43, 27, 9),
        accent: Color::Rgb(255, 208, 64),
        secondary: Color::Rgb(255, 104, 69),
        text: Color::Rgb(255, 245, 220),
        muted: Color::Rgb(176, 147, 99),
    },
    Palette {
        name: "CYBERCORE / OLED Black",
        bg: Color::Rgb(0, 0, 0),
        panel: Color::Rgb(10, 12, 16),
        accent: Color::Rgb(0, 235, 255),
        secondary: Color::Rgb(255, 60, 196),
        text: Color::Rgb(237, 244, 250),
        muted: Color::Rgb(133, 148, 164),
    },
    Palette {
        name: "CYBERCORE / Graphite",
        bg: Color::Rgb(11, 12, 14),
        panel: Color::Rgb(24, 26, 30),
        accent: Color::Rgb(166, 255, 72),
        secondary: Color::Rgb(82, 220, 198),
        text: Color::Rgb(236, 239, 242),
        muted: Color::Rgb(151, 158, 167),
    },
    Palette {
        name: "CYBERCORE / Charcoal",
        bg: Color::Rgb(17, 18, 20),
        panel: Color::Rgb(34, 36, 40),
        accent: Color::Rgb(255, 190, 92),
        secondary: Color::Rgb(255, 111, 128),
        text: Color::Rgb(245, 243, 239),
        muted: Color::Rgb(169, 170, 172),
    },
    Palette {
        name: "CYBERCORE / Gunmetal",
        bg: Color::Rgb(12, 15, 19),
        panel: Color::Rgb(25, 31, 38),
        accent: Color::Rgb(111, 205, 255),
        secondary: Color::Rgb(196, 151, 255),
        text: Color::Rgb(229, 237, 245),
        muted: Color::Rgb(145, 158, 174),
    },
];

struct App {
    vault: Vault,
    notes: Vec<Note>,
    selected: usize,
    query: String,
    searching: bool,
    modal: Option<Modal>,
    palette: usize,
    palette_path: PathBuf,
    vault_paths: Vec<PathBuf>,
    vaults_path: PathBuf,
    status: String,
}

enum Modal {
    Help,
    Themes,
    Templates {
        selected: usize,
    },
    EditMenu {
        selected: usize,
    },
    Vaults {
        selected: usize,
    },
    Prompt {
        label: &'static str,
        value: String,
        action: PromptAction,
    },
}
enum PromptAction {
    NewNote { template: usize },
    Attach,
    CreateFolder,
    MoveNote,
    AddVault,
    RenameNote,
    EditTags,
}

const EDIT_OPTIONS: [&str; 9] = [
    "Edit body · preferred editor",
    "Edit body · Neovim",
    "Edit body · Nano",
    "Rename note",
    "Edit tags",
    "Append daily note template",
    "Append meeting notes template",
    "Append project brief template",
    "Append checklist",
];

const NOTE_TEMPLATES: [(&str, &str); 8] = [
    ("Blank note", ""),
    (
        "Daily note",
        "## Focus\n\n- [ ] \n\n## Notes\n\n## Wrap-up\n",
    ),
    (
        "Meeting notes",
        "## Attendees\n\n## Agenda\n\n## Notes\n\n## Decisions\n\n## Action items\n- [ ] \n",
    ),
    (
        "Project brief",
        "## Outcome\n\n## Context\n\n## Plan\n1. \n\n## Risks\n\n## Next steps\n- [ ] \n",
    ),
    (
        "Research notes",
        "## Question\n\n## Summary\n\n## Findings\n\n## Sources\n\n## Open questions\n",
    ),
    (
        "Reading notes",
        "## Book / article\n\n## Key ideas\n\n## Quotes\n\n## My response\n\n## Follow-up\n- [ ] \n",
    ),
    (
        "Weekly review",
        "## Wins\n\n## What I learned\n\n## Open loops\n- [ ] \n\n## Next week\n",
    ),
    ("Checklist", "- [ ] \n- [ ] \n- [ ] \n"),
];

impl App {
    fn refresh(&mut self) {
        let result = if self.query.trim().is_empty() {
            self.vault.list()
        } else {
            self.vault.search(&self.query)
        };
        match result {
            Ok(notes) => {
                self.notes = notes;
                self.selected = self.selected.min(self.notes.len().saturating_sub(1));
            }
            Err(error) => self.status = format!("Index error: {error}"),
        }
    }
    fn selected_note(&self) -> Option<&Note> {
        self.notes.get(self.selected)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let explicit_root = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| env::var_os("CYBERNOTE_VAULT").map(PathBuf::from));
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let root = explicit_root
        .clone()
        .unwrap_or_else(|| home.join("Documents/Cybernote"));
    let root = absolute_path(&root)?;
    let vault = Vault::open(&root)?;
    if explicit_root.is_none() {
        let legacy = home.join("Documents/darkstar/notes");
        if legacy.is_dir() && legacy != root {
            match vault.import_legacy_sections(&legacy) {
                Ok(count) if count > 0 => {
                    eprintln!("Imported {count} notes from the previous Cybernote Markdown files.")
                }
                Ok(_) => {}
                Err(error) => eprintln!("Legacy note import skipped: {error}"),
            }
        }
    }
    let vaults_path = vaults_path(&home);
    let mut vault_paths = load_vault_paths(&vaults_path);
    if !vault_paths.contains(&root) {
        vault_paths.push(root);
    }
    store_vault_paths(&vaults_path, &vault_paths)?;
    let mut app = App {
        vault,
        notes: Vec::new(),
        selected: 0,
        query: String::new(),
        searching: false,
        modal: None,
        palette: load_palette(&home),
        palette_path: palette_path(&home),
        vault_paths,
        vaults_path,
        status: "✨ Your thoughts, in your hands. Press ? for the controls.".into(),
    };
    app.refresh();
    let mut terminal = setup_terminal()?;
    let outcome = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    outcome?;
    Ok(())
}

fn setup_terminal() -> io::Result<Term> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Term) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn run_app(terminal: &mut Term, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;
        if !event::poll(Duration::from_millis(200))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if handle_key(terminal, app, key)? {
            return Ok(());
        }
    }
}

fn handle_key(terminal: &mut Term, app: &mut App, key: KeyEvent) -> io::Result<bool> {
    if let Some(modal) = app.modal.take() {
        match modal {
            modal @ (Modal::Help | Modal::Themes) => {
                let is_themes = matches!(&modal, Modal::Themes);
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    return Ok(false);
                }
                if matches!(key.code, KeyCode::Enter) && is_themes {
                    app.status = format!("Theme selected: {}", PALETTES[app.palette].name);
                    store_palette(&app.palette_path, app.palette);
                } else if is_themes && matches!(key.code, KeyCode::Up | KeyCode::Char('k')) {
                    app.palette = (app.palette + PALETTES.len() - 1) % PALETTES.len();
                    store_palette(&app.palette_path, app.palette);
                    app.modal = Some(modal);
                } else if is_themes && matches!(key.code, KeyCode::Down | KeyCode::Char('j')) {
                    app.palette = (app.palette + 1) % PALETTES.len();
                    store_palette(&app.palette_path, app.palette);
                    app.modal = Some(modal);
                } else if is_themes && let KeyCode::Char(n @ '1'..='9') = key.code {
                    app.palette = n.to_digit(10).unwrap_or(1) as usize - 1;
                    store_palette(&app.palette_path, app.palette);
                    app.modal = Some(modal);
                } else if is_themes && key.code == KeyCode::Char('0') {
                    app.palette = 9;
                    store_palette(&app.palette_path, app.palette);
                    app.modal = Some(modal);
                } else if is_themes && key.code == KeyCode::Char('-') {
                    app.palette = 10;
                    store_palette(&app.palette_path, app.palette);
                    app.modal = Some(modal);
                } else if is_themes && key.code == KeyCode::Char('=') {
                    app.palette = 11;
                    store_palette(&app.palette_path, app.palette);
                    app.modal = Some(modal);
                } else {
                    app.modal = Some(modal);
                }
            }
            Modal::Templates { mut selected } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {}
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.saturating_sub(1);
                    app.modal = Some(Modal::Templates { selected });
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    selected = (selected + 1).min(NOTE_TEMPLATES.len() - 1);
                    app.modal = Some(Modal::Templates { selected });
                }
                KeyCode::Enter => {
                    app.modal = Some(Modal::Prompt {
                        label: "New note title",
                        value: String::new(),
                        action: PromptAction::NewNote { template: selected },
                    })
                }
                _ => app.modal = Some(Modal::Templates { selected }),
            },
            Modal::EditMenu { mut selected } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {}
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.saturating_sub(1);
                    app.modal = Some(Modal::EditMenu { selected });
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    selected = (selected + 1).min(EDIT_OPTIONS.len() - 1);
                    app.modal = Some(Modal::EditMenu { selected });
                }
                KeyCode::Enter => {
                    if let Some(note) = app.selected_note().cloned() {
                        match selected {
                            0..=2 => {
                                let editor = match selected {
                                    1 => Some("nvim"),
                                    2 => Some("nano"),
                                    _ => None,
                                };
                                match edit_note(terminal, &app.vault, &note, editor) {
                                    Ok(()) => {
                                        app.status =
                                            "✅ Body saved with a recovery snapshot.".into()
                                    }
                                    Err(error) => app.status = format!("Editor failed: {error}"),
                                }
                                app.refresh();
                            }
                            3 | 4 => {
                                let (label, value, action) = if selected == 3 {
                                    ("New note title", note.title, PromptAction::RenameNote)
                                } else {
                                    (
                                        "Tags separated by commas",
                                        note.tags.join(", "),
                                        PromptAction::EditTags,
                                    )
                                };
                                app.modal = Some(Modal::Prompt {
                                    label,
                                    value,
                                    action,
                                });
                            }
                            5..=8 => {
                                let template_index = [1, 2, 3, 7][selected - 5];
                                let addition = NOTE_TEMPLATES[template_index].1;
                                let body = if note.body.trim().is_empty() {
                                    addition.to_owned()
                                } else {
                                    format!("{}\n\n{}", note.body.trim_end(), addition)
                                };
                                match app.vault.save(
                                    &note.id,
                                    &note.title,
                                    &body,
                                    note.tags.clone(),
                                    Some(note.updated),
                                ) {
                                    Ok(_) => {
                                        app.status = format!(
                                            "✨ Added {}",
                                            EDIT_OPTIONS[selected].to_lowercase()
                                        )
                                    }
                                    Err(error) => {
                                        app.status = format!("Could not update note: {error}")
                                    }
                                }
                                app.refresh();
                            }
                            _ => {}
                        }
                    }
                }
                _ => app.modal = Some(Modal::EditMenu { selected }),
            },
            Modal::Vaults { mut selected } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {}
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.saturating_sub(1);
                    app.modal = Some(Modal::Vaults { selected });
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    selected = (selected + 1).min(app.vault_paths.len().saturating_sub(1));
                    app.modal = Some(Modal::Vaults { selected });
                }
                KeyCode::Enter => {
                    if let Some(root) = app.vault_paths.get(selected).cloned() {
                        match Vault::open(&root) {
                            Ok(vault) => {
                                app.vault = vault;
                                app.query.clear();
                                app.selected = 0;
                                app.refresh();
                                app.status = format!("🗃️ Opened vault {}", root.display());
                            }
                            Err(error) => app.status = format!("Could not open vault: {error}"),
                        }
                    }
                }
                KeyCode::Char('a') => {
                    app.modal = Some(Modal::Prompt {
                        label: "Path to existing vault folder (~/ supported)",
                        value: String::new(),
                        action: PromptAction::AddVault,
                    })
                }
                KeyCode::Char('x') if app.vault_paths.len() > 1 => {
                    let active = app.vault.root();
                    if app
                        .vault_paths
                        .get(selected)
                        .is_some_and(|path| path != active)
                    {
                        app.vault_paths.remove(selected);
                        selected = selected.min(app.vault_paths.len().saturating_sub(1));
                        if let Err(error) = store_vault_paths(&app.vaults_path, &app.vault_paths) {
                            app.status = format!("Could not save vault list: {error}");
                        }
                    }
                    app.modal = Some(Modal::Vaults { selected });
                }
                _ => app.modal = Some(Modal::Vaults { selected }),
            },
            Modal::Prompt {
                label,
                mut value,
                action,
            } => match key.code {
                KeyCode::Esc => app.status = "Cancelled.".into(),
                KeyCode::Enter => {
                    match action {
                        PromptAction::NewNote { template } if !value.trim().is_empty() => {
                            if let Ok(note) = app.vault.create(
                                value.trim(),
                                NOTE_TEMPLATES[template].1,
                                Vec::new(),
                            ) {
                                if let Err(error) = edit_note(terminal, &app.vault, &note, None) {
                                    app.status = format!("Editor failed: {error}");
                                } else {
                                    app.status = "📝 Note created and saved.".into();
                                }
                                app.refresh();
                            }
                        }
                        PromptAction::Attach => {
                            if let Some(note) = app.selected_note().cloned() {
                                match app.vault.attach(&note.id, Path::new(value.trim())) {
                                    Ok(path) => {
                                        app.status = format!(
                                            "📎 Attached {}",
                                            path.file_name().unwrap_or_default().to_string_lossy()
                                        )
                                    }
                                    Err(error) => {
                                        app.status = format!("Attachment failed: {error}")
                                    }
                                }
                            }
                        }
                        PromptAction::CreateFolder => match app.vault.create_folder(value.trim()) {
                            Ok(folder) => app.status = format!("📁 Created folder {folder}"),
                            Err(error) => app.status = format!("Could not create folder: {error}"),
                        },
                        PromptAction::MoveNote => {
                            if let Some(note) = app.selected_note().cloned() {
                                match app.vault.move_note_to_folder(&note.id, value.trim()) {
                                    Ok(note) => {
                                        app.status = if note.folder.is_empty() {
                                            "Moved note to the vault root.".into()
                                        } else {
                                            format!("Moved note to 📁 {}", note.folder)
                                        };
                                    }
                                    Err(error) => {
                                        app.status = format!("Could not move note: {error}")
                                    }
                                }
                            }
                        }
                        PromptAction::RenameNote | PromptAction::EditTags => {
                            if let Some(note) = app.selected_note().cloned() {
                                if matches!(action, PromptAction::RenameNote)
                                    && value.trim().is_empty()
                                {
                                    app.status = "A note title cannot be empty.".into();
                                    app.refresh();
                                    return Ok(false);
                                }
                                let (title, tags) = if matches!(action, PromptAction::RenameNote) {
                                    (value.trim().to_owned(), note.tags.clone())
                                } else {
                                    (
                                        note.title.clone(),
                                        value
                                            .split(',')
                                            .map(str::trim)
                                            .filter(|tag| !tag.is_empty())
                                            .map(str::to_owned)
                                            .collect(),
                                    )
                                };
                                match app.vault.save(
                                    &note.id,
                                    &title,
                                    &note.body,
                                    tags,
                                    Some(note.updated),
                                ) {
                                    Ok(_) => {
                                        app.status = if matches!(action, PromptAction::RenameNote) {
                                            "✏️ Note renamed and history saved.".into()
                                        } else {
                                            "🏷️ Tags updated and history saved.".into()
                                        }
                                    }
                                    Err(error) => {
                                        app.status = format!("Could not update note: {error}")
                                    }
                                }
                            }
                        }
                        PromptAction::AddVault => {
                            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                            let candidate = expand_path(value.trim(), &home);
                            match candidate.canonicalize() {
                                Ok(root) if root.is_dir() => match Vault::open(&root) {
                                    Ok(vault) => {
                                        if !app.vault_paths.contains(&root) {
                                            app.vault_paths.push(root.clone());
                                        }
                                        app.vault = vault;
                                        app.query.clear();
                                        app.selected = 0;
                                        app.refresh();
                                        if let Err(error) =
                                            store_vault_paths(&app.vaults_path, &app.vault_paths)
                                        {
                                            app.status = format!(
                                                "Vault opened, but list save failed: {error}"
                                            );
                                        } else {
                                            app.status =
                                                format!("🗃️ Added vault {}", root.display());
                                        }
                                    }
                                    Err(error) => {
                                        app.status = format!("Could not open vault: {error}")
                                    }
                                },
                                _ => app.status = "Choose an existing folder path.".into(),
                            }
                        }
                        _ => {}
                    }
                    app.refresh();
                }
                KeyCode::Backspace => {
                    value.pop();
                    app.modal = Some(Modal::Prompt {
                        label,
                        value,
                        action,
                    });
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    value.push(c);
                    app.modal = Some(Modal::Prompt {
                        label,
                        value,
                        action,
                    });
                }
                _ => {
                    app.modal = Some(Modal::Prompt {
                        label,
                        value,
                        action,
                    })
                }
            },
        }
        return Ok(false);
    }
    if app.searching {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => app.searching = false,
            KeyCode::Backspace => {
                app.query.pop();
                app.refresh();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.query.push(c);
                app.refresh();
            }
            _ => {}
        }
        return Ok(false);
    }
    match key.code {
        KeyCode::Char('q') if key.modifiers.is_empty() => return Ok(true),
        KeyCode::Char('?') => app.modal = Some(Modal::Help),
        KeyCode::Char('p') => app.modal = Some(Modal::Themes),
        KeyCode::Char('t') => {
            app.palette = (app.palette + 1) % PALETTES.len();
            store_palette(&app.palette_path, app.palette);
            app.status = format!("🎨 {}", PALETTES[app.palette].name);
        }
        KeyCode::Char('/') => {
            app.searching = true;
            app.query.clear();
            app.refresh();
        }
        KeyCode::Char('n') => app.modal = Some(Modal::Templates { selected: 0 }),
        KeyCode::Char('e') if app.selected_note().is_some() => {
            app.modal = Some(Modal::EditMenu { selected: 0 })
        }
        KeyCode::Enter => {
            if let Some(note) = app.selected_note().cloned() {
                match edit_note(terminal, &app.vault, &note, None) {
                    Ok(()) => app.status = "✅ Body saved with a recovery snapshot.".into(),
                    Err(error) => app.status = format!("Editor failed: {error}"),
                }
                app.refresh();
            }
        }
        KeyCode::Char('a') => {
            if app.selected_note().is_some() {
                app.modal = Some(Modal::Prompt {
                    label: "Path to attachment",
                    value: String::new(),
                    action: PromptAction::Attach,
                });
            }
        }
        KeyCode::Char('s') => {
            if app.selected_note().is_some()
                && let Err(error) = capture_screenshot(terminal, app)
            {
                app.status = format!("Screenshot capture failed: {error}");
            }
        }
        KeyCode::Char('f') => {
            app.modal = Some(Modal::Prompt {
                label: "New folder path inside this vault",
                value: String::new(),
                action: PromptAction::CreateFolder,
            })
        }
        KeyCode::Char('m') => {
            if app.selected_note().is_some() {
                app.modal = Some(Modal::Prompt {
                    label: "Move note to folder (blank = vault root)",
                    value: String::new(),
                    action: PromptAction::MoveNote,
                })
            }
        }
        KeyCode::Char('v') => {
            let selected = app
                .vault_paths
                .iter()
                .position(|path| path == app.vault.root())
                .unwrap_or(0);
            app.modal = Some(Modal::Vaults { selected });
        }
        KeyCode::Char('d') => {
            if let Some(note) = app.selected_note().cloned() {
                match app.vault.trash(&note.id) {
                    Ok(()) => {
                        app.status = "🗃️ Moved to Cybernote trash; recoverable on disk.".into()
                    }
                    Err(error) => app.status = format!("Could not move note: {error}"),
                }
                app.refresh();
            }
        }
        KeyCode::Char('u') => match app.vault.restore_latest_trash() {
            Ok(Some(note)) => {
                app.status = format!("♻️ Restored {} from Cybernote trash.", note.title)
            }
            Ok(None) => app.status = "Cybernote trash is empty.".into(),
            Err(error) => app.status = format!("Could not restore note: {error}"),
        },
        KeyCode::Char('j') | KeyCode::Down => {
            app.selected = (app.selected + 1).min(app.notes.len().saturating_sub(1))
        }
        KeyCode::Char('k') | KeyCode::Up => app.selected = app.selected.saturating_sub(1),
        KeyCode::Char('g') => app.selected = 0,
        KeyCode::Char('G') => app.selected = app.notes.len().saturating_sub(1),
        KeyCode::Char('r') => {
            app.refresh();
            app.status = "🔄 Vault re-indexed.".into();
        }
        _ => {}
    }
    Ok(false)
}

fn palette_path(home: &Path) -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| home.join(".config"))
        .join("cybernote/palette")
}

fn vaults_path(home: &Path) -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| home.join(".config"))
        .join("cybernote/vaults.json")
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn load_vault_paths(path: &Path) -> Vec<PathBuf> {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Vec<String>>(&bytes).ok())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|path| absolute_path(Path::new(&path)).ok())
        .filter(|path| path.is_dir())
        .collect()
}

fn store_vault_paths(path: &Path, vaults: &[PathBuf]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let roots = vaults
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    fs::write(
        path,
        serde_json::to_vec_pretty(&roots).map_err(io::Error::other)?,
    )
}

fn expand_path(value: &str, home: &Path) -> PathBuf {
    if value == "~" {
        home.to_path_buf()
    } else if let Some(relative) = value.strip_prefix("~/") {
        home.join(relative)
    } else {
        PathBuf::from(value)
    }
}

fn load_palette(home: &Path) -> usize {
    fs::read_to_string(palette_path(home))
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|index| *index < PALETTES.len())
        .unwrap_or(0)
}

fn store_palette(path: &Path, palette: usize) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, palette.to_string());
}

fn edit_note(
    terminal: &mut Term,
    vault: &Vault,
    note: &Note,
    requested_editor: Option<&str>,
) -> io::Result<()> {
    let configured = requested_editor
        .map(str::to_owned)
        .or_else(|| env::var("VISUAL").ok())
        .or_else(|| env::var("EDITOR").ok());
    let mut command_parts = configured
        .as_deref()
        .map(str::split_whitespace)
        .map(Iterator::collect::<Vec<_>>)
        .unwrap_or_default();
    if command_parts.is_empty() {
        command_parts = ["nvim", "nano", "vi"]
            .into_iter()
            .find(|candidate| command_exists(candidate))
            .map(|candidate| vec![candidate])
            .unwrap_or_default();
    }
    let Some((program, args)) = command_parts.split_first() else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No editor found. Install Neovim or Nano, or set $EDITOR.",
        ));
    };
    let mut temp = tempfile::NamedTempFile::new()?;
    temp.write_all(format!("{}\n", note.body).as_bytes())?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let result = Command::new(program).args(args).arg(temp.path()).status();
    let restore = enable_raw_mode()
        .and_then(|()| execute!(terminal.backend_mut(), EnterAlternateScreen))
        .and_then(|()| terminal.clear());
    restore?;
    match result {
        Ok(status) if status.success() => {
            let body = fs::read_to_string(temp.path())?;
            vault
                .save(
                    &note.id,
                    &note.title,
                    &body,
                    note.tags.clone(),
                    Some(note.updated),
                )
                .map_err(|e| io::Error::other(e.to_string()))?;
        }
        Ok(_) => return Err(io::Error::other("editor returned an error")),
        Err(error) => return Err(error),
    }
    Ok(())
}

fn command_exists(program: &str) -> bool {
    if program.contains('/') {
        return Path::new(program).is_file();
    }
    env::var_os("PATH").is_some_and(|paths| {
        env::split_paths(&paths).any(|directory| directory.join(program).is_file())
    })
}

fn capture_screenshot(terminal: &mut Term, app: &mut App) -> io::Result<()> {
    let note = app
        .selected_note()
        .cloned()
        .ok_or_else(|| io::Error::other("select a note before capturing a screenshot"))?;
    let capture_dir = tempfile::tempdir()?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let result = Command::new("omarchy")
        .args(["capture", "screenshot", "region", "save"])
        .env("OMASNAP_SCREENSHOT_DIR", capture_dir.path())
        .status();
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;
    let status = result?;
    if !status.success() {
        return Err(io::Error::other("screenshot tool returned an error"));
    }
    let image = fs::read_dir(capture_dir.path())?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .find(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    matches!(
                        extension.to_ascii_lowercase().as_str(),
                        "png" | "jpg" | "jpeg" | "webp"
                    )
                })
        })
        .ok_or_else(|| io::Error::other("screenshot tool did not save an image"))?;
    let attached = app
        .vault
        .attach(&note.id, &image)
        .map_err(|error| io::Error::other(error.to_string()))?;
    app.status = format!(
        "📸 Screenshot attached: {}",
        attached.file_name().unwrap_or_default().to_string_lossy()
    );
    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let p = PALETTES[app.palette];
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(p.bg)), area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(5),
        ])
        .split(area);
    draw_header(frame, rows[0], app, p);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(36), Constraint::Percentage(64)])
        .split(rows[1]);
    draw_notes(frame, columns[0], app, p);
    draw_preview(frame, columns[1], app, p);
    let prompt = if app.searching {
        format!("🔎 Search: {}▌", app.query)
    } else {
        app.status.clone()
    };
    let controls = [
        vec![
            ("j/k ↑↓", "move"),
            ("/", "search"),
            ("n", "new"),
            ("e", "edit"),
        ],
        vec![
            ("a", "attach"),
            ("s", "screenshot"),
            ("f", "folder"),
            ("m", "move note"),
        ],
        vec![
            ("v", "vaults"),
            ("d", "trash"),
            ("u", "restore"),
            ("p", "colors"),
            ("?", "help"),
            ("q", "quit"),
        ],
    ];
    let control_lines = controls.into_iter().map(|group| {
        let mut spans = Vec::new();
        for (index, (key, label)) in group.into_iter().enumerate() {
            if index > 0 {
                spans.push(Span::styled(
                    "  ·  ",
                    Style::default().fg(p.secondary).bg(p.panel),
                ));
            }
            spans.push(Span::styled(
                format!(" {key} "),
                Style::default()
                    .fg(p.bg)
                    .bg(p.accent)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(" {label} "),
                Style::default()
                    .fg(p.text)
                    .bg(p.panel)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        Line::from(spans)
    });
    let footer = control_lines
        .chain(std::iter::once(Line::from(Span::styled(
            prompt,
            Style::default().fg(p.text).bg(p.panel),
        ))))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(Text::from(footer))
            .style(Style::default().fg(p.text).bg(p.panel))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(p.accent))
                    .style(Style::default().bg(p.panel)),
            ),
        rows[2],
    );
    if let Some(modal) = &app.modal {
        draw_modal(frame, area, modal, app, p);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, p: Palette) {
    let title = Line::from(vec![
        Span::styled(
            " ✦ CYBERNOTE ",
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled("LOCAL-FIRST FIELD NOTES", Style::default().fg(p.muted)),
        Span::raw("   "),
        Span::styled(format!("🎨 {}", p.name), Style::default().fg(p.secondary)),
    ]);
    let subtitle = Line::from(vec![
        Span::styled(
            format!("  📁 {}", app.vault.root().display()),
            Style::default().fg(p.muted),
        ),
        Span::raw("   "),
        Span::styled(
            format!("{} notes", app.notes.len()),
            Style::default().fg(p.text),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(Text::from(vec![title, subtitle])).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(p.accent)),
        ),
        area,
    );
}

fn draw_notes(frame: &mut Frame, area: Rect, app: &App, p: Palette) {
    let items = app
        .notes
        .iter()
        .map(|note| {
            let tags = if note.tags.is_empty() {
                String::new()
            } else {
                format!("\n  🏷 {}", note.tags.join("  #"))
            };
            let folder = if note.folder.is_empty() {
                "📁 root".to_owned()
            } else {
                format!("📁 {}", note.folder)
            };
            ListItem::new(Text::from(vec![
                Line::from(Span::styled(
                    format!("📝 {}", note.title),
                    Style::default().fg(p.text).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!("  {}", preview_text(&note.body, 62)),
                    Style::default().fg(p.muted),
                )),
                Line::from(vec![
                    Span::styled(folder, Style::default().fg(p.accent)),
                    Span::styled(tags, Style::default().fg(p.secondary)),
                ]),
            ]))
        })
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select((!app.notes.is_empty()).then_some(app.selected));
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(p.panel)
                .fg(p.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▌ ")
        .block(
            Block::default()
                .title("  🗂 NOTES  ")
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(p.accent))
                .style(Style::default().bg(p.bg)),
        );
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_preview(frame: &mut Frame, area: Rect, app: &App, p: Palette) {
    let Some(note) = app.selected_note() else {
        frame.render_widget(
            Paragraph::new("  🌌 No notes yet. Press n to create your first one.")
                .style(Style::default().fg(p.muted))
                .block(
                    Block::default()
                        .title("  PREVIEW  ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(p.accent)),
                ),
            area,
        );
        return;
    };
    let backlinks = app
        .vault
        .backlinks(note)
        .map(|n| n.len())
        .unwrap_or_default();
    let tags = if note.tags.is_empty() {
        "🏷 no tags".to_owned()
    } else {
        format!(
            "🏷 {}",
            note.tags
                .iter()
                .map(|t| format!("#{t}"))
                .collect::<Vec<_>>()
                .join("  ")
        )
    };
    let lines = vec![
        Line::from(Span::styled(
            format!("✦ {}", note.title),
            Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(tags, Style::default().fg(p.secondary))),
        Line::from(Span::styled(
            format!(
                "📁 {}   ·   ↗ {backlinks} backlinks   ·   🕘 updated {}",
                if note.folder.is_empty() {
                    "root"
                } else {
                    &note.folder
                },
                note.updated
            ),
            Style::default().fg(p.muted),
        )),
        Line::from(""),
        Line::from(note.body.clone()),
    ];
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title("  ✨ LIVE PREVIEW  ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(p.accent))
                    .style(Style::default().bg(p.bg)),
            ),
        area,
    );
}

fn draw_modal(frame: &mut Frame, area: Rect, modal: &Modal, app: &App, p: Palette) {
    if let Modal::Vaults { selected } = modal {
        let box_area = centered_rect(78, 70, area);
        frame.render_widget(Clear, box_area);
        let mut lines = vec![Line::from(Span::styled(
            "↑/↓ select · Enter open · a add existing folder · x remove from list",
            Style::default().fg(p.text),
        ))];
        for (index, root) in app.vault_paths.iter().enumerate() {
            let active = root == app.vault.root();
            let is_selected = index == *selected;
            let marker = if is_selected { "▶ " } else { "  " };
            let active_mark = if active { "  ● CURRENT" } else { "" };
            let style = if is_selected {
                Style::default()
                    .fg(p.bg)
                    .bg(p.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(p.text).bg(p.panel)
            };
            lines.push(Line::from(Span::styled(
                format!("{marker}{}{active_mark}", root.display()),
                style,
            )));
        }
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .style(Style::default().fg(p.text).bg(p.panel))
                .block(
                    Block::default()
                        .title("  🗃 VAULT SELECTOR  ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(p.secondary))
                        .style(Style::default().bg(p.panel)),
                ),
            box_area,
        );
        return;
    }
    let box_area = if matches!(modal, Modal::Themes) {
        centered_rect(84, 90, area)
    } else {
        centered_rect(72, 72, area)
    };
    frame.render_widget(Clear, box_area);
    let (title, text) = match modal {
        Modal::Help => (
            "⌘ CYBERNOTE FIELD GUIDE",
            vec![
                "j/k or ↑/↓   Navigate the note stream",
                "/             Search notes instantly",
                "n             Choose a starter template and create a note",
                "e             Open the full edit menu",
                "Enter         Quick edit in your preferred editor",
                "a             Attach a file to the selected note",
                "s             Capture a region screenshot into the selected note",
                "f             Create an in-vault folder",
                "m             Move the selected note to a folder",
                "v             Switch or add a vault",
                "d             Move a note into recoverable trash",
                "u             Restore the most recently trashed note",
                "p             Open the palette chooser",
                "t             Cycle palettes",
                "r             Rebuild the search index",
                "q             Quit",
                "",
                "Markdown is canonical. Search indexes are rebuildable.",
            ],
        ),
        Modal::Themes => (
            "🎨 CYBERCORE PALETTE LAB",
            vec![
                "1  NEON CYAN       electric blue + hot pink",
                "2  PLASMA          violet + ion teal",
                "3  EMBER           molten orange + magenta",
                "4  BIOHAZARD       acid green + bio teal",
                "5  AURORA          mint + electric indigo",
                "6  ROSE QUARTZ     neon rose + warm gold",
                "7  ARCTIC          ice blue + violet",
                "8  SOLAR FLARE     radiant gold + hot orange",
                "9  OLED BLACK      pure black + cyan",
                "0  GRAPHITE        near-black + acid green",
                "-  CHARCOAL       dark gray + warm amber",
                "=  GUNMETAL       blue-black + ice blue",
                "",
                "1–9, 0, -, = choose · ↑/↓ browse · Enter apply · Esc close",
                "",
                "Theme changes apply instantly and never alter your notes.",
            ],
        ),
        Modal::Templates { selected } => {
            let mut options = vec![
                "Pick a starter, then name your note · Esc to cancel".to_owned(),
                String::new(),
            ];
            options.extend(NOTE_TEMPLATES.iter().enumerate().map(|(index, (name, _))| {
                format!(
                    "{} {}  {}",
                    if index == *selected { "▶" } else { " " },
                    index + 1,
                    name
                )
            }));
            let lines = options.into_iter().map(Line::from).collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(Text::from(lines))
                    .style(Style::default().fg(p.text).bg(p.panel))
                    .wrap(Wrap { trim: false })
                    .block(
                        Block::default()
                            .title("  ✨ NOTE TEMPLATE LIBRARY  ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(p.secondary))
                            .style(Style::default().bg(p.panel)),
                    ),
                box_area,
            );
            return;
        }
        Modal::EditMenu { selected } => {
            let mut options = vec![
                "↑/↓ navigate · Enter choose · Esc close".to_owned(),
                String::new(),
            ];
            options.extend(EDIT_OPTIONS.iter().enumerate().map(|(index, option)| {
                format!(
                    "{} {}  {}",
                    if index == *selected { "▶" } else { " " },
                    index + 1,
                    option
                )
            }));
            options.push(String::new());
            options.push(
                "Body edits use $VISUAL or $EDITOR; Neovim/Nano are available directly.".into(),
            );
            let lines = options.into_iter().map(Line::from).collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(Text::from(lines))
                    .style(Style::default().fg(p.text).bg(p.panel))
                    .wrap(Wrap { trim: false })
                    .block(
                        Block::default()
                            .title("  ✏️ NOTE EDIT STUDIO  ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(p.secondary))
                            .style(Style::default().bg(p.panel)),
                    ),
                box_area,
            );
            return;
        }
        Modal::Prompt { label, value, .. } => (
            *label,
            vec![value.as_str(), "", "Enter confirm · Esc cancel"],
        ),
        Modal::Vaults { .. } => unreachable!("vault picker is rendered above"),
    };
    let lines = text
        .into_iter()
        .map(|line| Line::from(line.to_owned()))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .style(Style::default().fg(p.text).bg(p.panel))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title(format!("  {title}  "))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(p.secondary))
                    .style(Style::default().bg(p.panel)),
            ),
        box_area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn preview_text(text: &str, width: usize) -> String {
    let flat = text.lines().next().unwrap_or_default().trim();
    if flat.chars().count() <= width {
        flat.to_owned()
    } else {
        format!(
            "{}…",
            flat.chars()
                .take(width.saturating_sub(1))
                .collect::<String>()
        )
    }
}
