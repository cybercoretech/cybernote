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

const PALETTES: [Palette; 4] = [
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
    status: String,
}

enum Modal {
    Help,
    Themes,
    Prompt {
        label: &'static str,
        value: String,
        action: PromptAction,
    },
}
enum PromptAction {
    NewNote,
    Attach,
}

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
    let mut app = App {
        vault,
        notes: Vec::new(),
        selected: 0,
        query: String::new(),
        searching: false,
        modal: None,
        palette: load_palette(&home),
        palette_path: palette_path(&home),
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
                } else if is_themes && let KeyCode::Char(n @ '1'..='4') = key.code {
                    app.palette = n.to_digit(10).unwrap_or(1) as usize - 1;
                    store_palette(&app.palette_path, app.palette);
                    app.modal = Some(modal);
                } else {
                    app.modal = Some(modal);
                }
            }
            Modal::Prompt {
                label,
                mut value,
                action,
            } => match key.code {
                KeyCode::Esc => app.status = "Cancelled.".into(),
                KeyCode::Enter => {
                    match action {
                        PromptAction::NewNote if !value.trim().is_empty() => {
                            if let Ok(note) = app.vault.create(value.trim(), "", Vec::new()) {
                                if let Err(error) = edit_note(terminal, &app.vault, &note) {
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
        KeyCode::Char('n') => {
            app.modal = Some(Modal::Prompt {
                label: "New note title",
                value: String::new(),
                action: PromptAction::NewNote,
            })
        }
        KeyCode::Char('e') | KeyCode::Enter => {
            if let Some(note) = app.selected_note().cloned() {
                match edit_note(terminal, &app.vault, &note) {
                    Ok(()) => app.status = "✅ Changes saved with a recovery snapshot.".into(),
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

fn edit_note(terminal: &mut Term, vault: &Vault, note: &Note) -> io::Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".into());
    let mut temp = tempfile::NamedTempFile::new()?;
    temp.write_all(format!("{}\n", note.body).as_bytes())?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let result = Command::new(editor).arg(temp.path()).status();
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;
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

fn draw(frame: &mut Frame, app: &App) {
    let p = PALETTES[app.palette];
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(p.bg)), area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(4),
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
        [
            ("j/k ↑↓", "move"),
            ("/", "search"),
            ("n", "new"),
            ("e", "edit"),
            ("a", "attach"),
        ],
        [
            ("d", "trash"),
            ("u", "restore"),
            ("p", "themes"),
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
        draw_modal(frame, area, modal, p);
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
            ListItem::new(Text::from(vec![
                Line::from(Span::styled(
                    format!("📝 {}", note.title),
                    Style::default().fg(p.text).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!("  {}", preview_text(&note.body, 62)),
                    Style::default().fg(p.muted),
                )),
                Line::from(Span::styled(tags, Style::default().fg(p.secondary))),
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
            format!("↗ {backlinks} backlinks   ·   🕘 updated {}", note.updated),
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

fn draw_modal(frame: &mut Frame, area: Rect, modal: &Modal, p: Palette) {
    let box_area = centered_rect(64, 62, area);
    frame.render_widget(Clear, box_area);
    let (title, text) = match modal {
        Modal::Help => (
            "⌘ CYBERNOTE FIELD GUIDE",
            vec![
                "j/k or ↑/↓   Navigate the note stream",
                "/             Search notes instantly",
                "n             Capture a new note in $EDITOR",
                "e or Enter    Edit the selected note in $EDITOR",
                "a             Attach a file to the selected note",
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
                "",
                "↑/↓ select · Enter apply · Esc close",
                "",
                "Theme changes apply instantly and never alter your notes.",
            ],
        ),
        Modal::Prompt { label, value, .. } => (
            *label,
            vec![value.as_str(), "", "Enter confirm · Esc cancel"],
        ),
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
