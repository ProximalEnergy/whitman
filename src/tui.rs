use std::io;
use std::path::Path;

use anyhow::{Result, anyhow};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::profile::{
    Profile, create_profile_file, filter_profiles, validate_profile_description,
    validate_profile_name,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Mode {
    Pick,
    Confirm,
    CreateName,
    CreateDescription,
}

#[derive(Debug)]
struct PickerState {
    query: String,
    selected: usize,
    mode: Mode,
    create_name: String,
    create_description: String,
    error: Option<String>,
}

pub fn run_profile_picker(
    agents_dir: &Path,
    profiles: &mut Vec<Profile>,
) -> Result<Option<Profile>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_picker_loop(&mut terminal, agents_dir, profiles);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_picker_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    agents_dir: &Path,
    profiles: &mut Vec<Profile>,
) -> Result<Option<Profile>> {
    let mut state = PickerState {
        query: String::new(),
        selected: 0,
        mode: Mode::Pick,
        create_name: String::new(),
        create_description: String::new(),
        error: None,
    };

    loop {
        let visible = visible_profiles(profiles, &state.query);
        clamp_selection(&mut state, visible.len());
        terminal.draw(|frame| draw(frame, &state, &visible))?;
        drop(visible);

        if let Some(selection) = handle_event(&mut state, agents_dir, profiles)? {
            return Ok(selection);
        }
    }
}

fn handle_event(
    state: &mut PickerState,
    agents_dir: &Path,
    profiles: &mut Vec<Profile>,
) -> Result<Option<Option<Profile>>> {
    let Event::Key(KeyEvent {
        code, modifiers, ..
    }) = event::read()?
    else {
        return Ok(None);
    };

    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        return Ok(Some(None));
    }

    match state.mode {
        Mode::Pick => {
            let visible = visible_profiles(profiles, &state.query);
            clamp_selection(state, visible.len());
            match code {
                KeyCode::Esc | KeyCode::Char('q') => Ok(Some(None)),
                KeyCode::Char('+')
                    if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    state.mode = Mode::CreateName;
                    state.create_name.clear();
                    state.create_description.clear();
                    state.error = None;
                    Ok(None)
                }
                KeyCode::Down => {
                    move_down(state, visible.len());
                    Ok(None)
                }
                KeyCode::Up => {
                    move_up(state, visible.len());
                    Ok(None)
                }
                KeyCode::Enter if !visible.is_empty() => {
                    state.mode = Mode::Confirm;
                    Ok(None)
                }
                KeyCode::Backspace => {
                    state.query.pop();
                    state.selected = 0;
                    state.error = None;
                    Ok(None)
                }
                KeyCode::Char(character)
                    if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    state.query.push(character);
                    state.selected = 0;
                    state.error = None;
                    Ok(None)
                }
                _ => Ok(None),
            }
        }
        Mode::Confirm => {
            let visible = visible_profiles(profiles, &state.query);
            clamp_selection(state, visible.len());
            match code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => Ok(Some(
                    visible
                        .get(state.selected)
                        .map(|profile| (*profile).clone()),
                )),
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    state.mode = Mode::Pick;
                    Ok(None)
                }
                _ => Ok(None),
            }
        }
        Mode::CreateName => handle_create_name_event(state, profiles, code, modifiers),
        Mode::CreateDescription => {
            handle_create_description_event(state, agents_dir, profiles, code, modifiers)
        }
    }
}

fn handle_create_name_event(
    state: &mut PickerState,
    profiles: &[Profile],
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Result<Option<Option<Profile>>> {
    match code {
        KeyCode::Esc => {
            state.mode = Mode::Pick;
            state.error = None;
            Ok(None)
        }
        KeyCode::Enter => {
            let name = state.create_name.trim();
            match validate_create_name(name, profiles) {
                Ok(()) => {
                    state.create_name = name.to_string();
                    state.mode = Mode::CreateDescription;
                    state.error = None;
                }
                Err(error) => state.error = Some(format!("{error:#}")),
            }
            Ok(None)
        }
        KeyCode::Backspace => {
            state.create_name.pop();
            state.error = None;
            Ok(None)
        }
        KeyCode::Char(character)
            if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            state.create_name.push(character);
            state.error = None;
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn handle_create_description_event(
    state: &mut PickerState,
    agents_dir: &Path,
    profiles: &mut Vec<Profile>,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Result<Option<Option<Profile>>> {
    match code {
        KeyCode::Esc => {
            state.mode = Mode::CreateName;
            state.error = None;
            Ok(None)
        }
        KeyCode::Enter => {
            let description = state.create_description.trim();
            match validate_profile_description(description) {
                Ok(_) => match create_profile_file(agents_dir, &state.create_name, description) {
                    Ok(profile) => {
                        profiles.push(profile.clone());
                        profiles.sort_by(|left, right| left.name.cmp(&right.name));
                        Ok(Some(Some(profile)))
                    }
                    Err(error) => {
                        state.error = Some(format!("{error:#}"));
                        Ok(None)
                    }
                },
                Err(error) => {
                    state.error = Some(format!("{error:#}"));
                    Ok(None)
                }
            }
        }
        KeyCode::Backspace => {
            state.create_description.pop();
            state.error = None;
            Ok(None)
        }
        KeyCode::Char(character)
            if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            state.create_description.push(character);
            state.error = None;
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn validate_create_name(name: &str, profiles: &[Profile]) -> Result<()> {
    validate_profile_name(name)?;
    if profiles.iter().any(|profile| profile.name == name) {
        return Err(anyhow!("profile already exists: {name}"));
    }
    Ok(())
}

fn draw(frame: &mut ratatui::Frame<'_>, state: &PickerState, visible: &[&Profile]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(match state.mode {
                Mode::Pick => 3,
                Mode::Confirm | Mode::CreateName | Mode::CreateDescription => 5,
            }),
        ])
        .split(frame.area());

    let search_text = match state.mode {
        Mode::CreateName => state.create_name.as_str(),
        Mode::CreateDescription => state.create_description.as_str(),
        Mode::Pick | Mode::Confirm => state.query.as_str(),
    };
    let search_title = match state.mode {
        Mode::CreateName => "New Profile Name",
        Mode::CreateDescription => "New Profile Description",
        Mode::Pick | Mode::Confirm => "Search",
    };
    let search = Paragraph::new(search_text)
        .block(Block::default().title(search_title).borders(Borders::ALL));
    frame.render_widget(search, chunks[0]);

    let items: Vec<ListItem<'_>> = visible
        .iter()
        .map(|profile| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    profile.name.as_str(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::raw(profile.description.as_str()),
            ]))
        })
        .collect();
    let mut list_state = ListState::default();
    if !visible.is_empty() {
        list_state.select(Some(state.selected));
    }

    let list = List::new(items)
        .block(Block::default().title("Profiles").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    let footer = match state.mode {
        Mode::Confirm => {
            if let Some(profile) = visible.get(state.selected) {
                Paragraph::new(format!(
                    "Use '{}' for ./AGENTS.md?\nEnter/y confirms. n/Esc returns.",
                    profile.name
                ))
            } else {
                Paragraph::new("No matching profile.")
            }
        }
        Mode::CreateName => Paragraph::new(status_text(
            state,
            "Enter saves name. Esc returns. Use ASCII letters, numbers, underscores, and hyphens.",
        )),
        Mode::CreateDescription => Paragraph::new(status_text(
            state,
            "Enter creates and selects. Esc returns to name. Description must be under 100 characters.",
        )),
        Mode::Pick => Paragraph::new(status_text(
            state,
            "Type to filter. Arrow keys move. Enter selects. + creates. q exits.",
        )),
    }
    .block(Block::default().title("Confirm").borders(Borders::ALL))
    .wrap(Wrap { trim: true });
    frame.render_widget(footer, chunks[2]);
}

fn status_text(state: &PickerState, fallback: &str) -> String {
    if let Some(error) = &state.error {
        format!("Error: {error}\n{fallback}")
    } else {
        fallback.to_string()
    }
}

fn visible_profiles<'a>(profiles: &'a [Profile], query: &str) -> Vec<&'a Profile> {
    filter_profiles(profiles, query)
}

fn clamp_selection(state: &mut PickerState, visible_len: usize) {
    if visible_len == 0 {
        state.selected = 0;
    } else if state.selected >= visible_len {
        state.selected = visible_len - 1;
    }
}

fn move_down(state: &mut PickerState, visible_len: usize) {
    if visible_len > 0 {
        state.selected = (state.selected + 1).min(visible_len - 1);
    }
}

fn move_up(state: &mut PickerState, visible_len: usize) {
    if visible_len > 0 {
        state.selected = state.selected.saturating_sub(1);
    }
}
