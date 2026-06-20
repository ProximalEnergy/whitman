use std::io;

use anyhow::Result;
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

use crate::profile::{Profile, filter_profiles};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Mode {
    Pick,
    Confirm,
}

#[derive(Debug)]
struct PickerState {
    query: String,
    selected: usize,
    mode: Mode,
}

pub fn run_profile_picker(profiles: &[Profile]) -> Result<Option<Profile>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_picker_loop(&mut terminal, profiles);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_picker_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    profiles: &[Profile],
) -> Result<Option<Profile>> {
    let mut state = PickerState {
        query: String::new(),
        selected: 0,
        mode: Mode::Pick,
    };

    loop {
        let visible = visible_profiles(profiles, &state.query);
        clamp_selection(&mut state, visible.len());
        terminal.draw(|frame| draw(frame, &state, &visible))?;

        if let Some(selection) = handle_event(&mut state, &visible)? {
            return Ok(selection);
        }
    }
}

fn handle_event(state: &mut PickerState, visible: &[&Profile]) -> Result<Option<Option<Profile>>> {
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
        Mode::Pick => match code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(Some(None)),
            KeyCode::Char('j') | KeyCode::Down => {
                move_down(state, visible.len());
                Ok(None)
            }
            KeyCode::Char('k') | KeyCode::Up => {
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
                Ok(None)
            }
            KeyCode::Char(character)
                if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                state.query.push(character);
                state.selected = 0;
                Ok(None)
            }
            _ => Ok(None),
        },
        Mode::Confirm => match code {
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
        },
    }
}

fn draw(frame: &mut ratatui::Frame<'_>, state: &PickerState, visible: &[&Profile]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(if state.mode == Mode::Confirm { 5 } else { 3 }),
        ])
        .split(frame.area());

    let search = Paragraph::new(state.query.as_str())
        .block(Block::default().title("Search").borders(Borders::ALL));
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

    let footer = if state.mode == Mode::Confirm {
        if let Some(profile) = visible.get(state.selected) {
            Paragraph::new(format!(
                "Use '{}' for ./AGENTS.md?\nEnter/y confirms. n/Esc returns.",
                profile.name
            ))
        } else {
            Paragraph::new("No matching profile.")
        }
    } else {
        Paragraph::new("Type to filter. j/k or arrows move. Enter selects. q exits.")
    }
    .block(Block::default().title("Confirm").borders(Borders::ALL))
    .wrap(Wrap { trim: true });
    frame.render_widget(footer, chunks[2]);
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
