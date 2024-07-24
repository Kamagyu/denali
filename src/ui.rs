use ratatui::{crossterm::event::{self, Event, KeyCode, KeyEventKind}, layout::{Alignment, Constraint, Direction, Layout}, prelude::Backend, style::{Color, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph}, Frame, Terminal};

use crate::parser;

#[derive(PartialEq, Debug)]
pub enum InputMode {
    Normal,
    Editing,
    SearchBarSelecting,
    InstalledModsSelecting,
    Downloading,
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    pub mc_version: String,
    search_list: Vec<String>,
    search_index: usize,
    installation_list: Vec<String>,
    installation_index: usize,
}

impl App {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            character_index: 0,
            mc_version: String::new(),
            search_index: 0,
            search_list: Vec::new(),
            installation_list: Vec::new(),
            installation_index: 0,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&mut self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn send_query(&mut self) {

        self.search_list.clear();


        let projects = parser::search_projects(self.input.as_str());

        for project in projects.unwrap() {
            self.search_list.push(format!("{} ({} downloads) | {}", project.title, project.downloads, project.id ));
        }

        self.input.clear();
        self.reset_cursor();
        self.input_mode = InputMode::SearchBarSelecting;
    }

    fn send_project_to_installation(&mut self) {
        self.input_mode = InputMode::Normal;
        
        self.installation_list.push(self.search_list[self.search_index].clone());
        self.search_index = 0;
        self.search_list.clear();
    }
}



pub fn ui(frame: &mut Frame, app: &App) {
    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ],
    )
    .split(frame.size());

    frame.render_widget(
        Block::new().borders(Borders::TOP).title(format!(" Minecraft version: {} ", app.mc_version)).title_alignment(Alignment::Center).light_red(),
        main_layout[0],
    );

    frame.render_widget(
        Block::new().borders(Borders::TOP).title(" Download Progress : "),
        main_layout[2],
    );

    let inner_layout = Layout::new(
        Direction::Horizontal,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .split(main_layout[1]);

    let search_bar_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ]
    );

    let [_, help_area, messages_area, input_area] = search_bar_layout.areas(inner_layout[0]);

    let installed_mods_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Min(1),
            Constraint::Length(3),
        ]
    );
    let [installed_mods_area, download_button_area] = installed_mods_layout.areas(inner_layout[1]);




    let input = Paragraph::new(app.input.as_str())
    .style(match app.input_mode {
        InputMode::Normal => Style::default(),
        InputMode::Editing => Style::default().fg(Color::Yellow),
        InputMode::SearchBarSelecting => Style::default(),
        InputMode::InstalledModsSelecting => Style::default(),
        InputMode::Downloading => Style::default(),
    })
    .block(Block::bordered().title("Input"));

    let messages: Vec<ListItem> = app
        .search_list
        .iter()
        .enumerate()
        .map(|(i, m)| match i == app.search_index {
            true => {
                let content = Line::from(Span::raw(format!("{i}: {m}"))).bold().yellow();
                ListItem::new(content)
            },
            false => {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            },
        })
        .collect();


    let messages = List::new(messages).block(Block::bordered().borders(Borders::LEFT));

    let installation_mods_list: Vec<ListItem> = app
        .installation_list
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let index = app.installation_index;
            
            if i == index && app.input_mode == InputMode::InstalledModsSelecting {
                let content = Line::from(Span::raw(format!("{i}: {m}"))).bold().yellow();
                ListItem::new(content)
            } 
            else {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            }
        })
        .collect();


    let installation_mods_list = List::new(installation_mods_list).block(Block::bordered().title("Installed mods list".bold().into_centered_line()));
    let search_bar_help = Paragraph::new("Press E to start editing    Press Esc to stop editing ".light_blue().italic()).block(Block::default().borders(Borders::NONE)).centered();
    let download_button: Paragraph;
    let download_button_text = match app.input_mode {
        InputMode::Downloading => "Downloading".bold(),
        _ => "Download".light_yellow(),
    };

    if app.installation_index == app.installation_list.len() && !app.installation_list.is_empty() {
        download_button = Paragraph::new(download_button_text).block(Block::default().borders(Borders::ALL)).centered().on_light_yellow();
    } else {
        download_button = Paragraph::new(download_button_text.light_yellow()).block(Block::default().borders(Borders::ALL)).centered();
    };

    match app.input_mode {
        InputMode::Editing => {
            #[allow(clippy::cast_possible_truncation)]
            frame.set_cursor(
                input_area.x + app.character_index as u16 + 1,
                input_area.y + 1,
            );
        },
        _ => {}
    }
        

    frame.render_widget(Block::bordered().title(" Mods search bar ".bold()).title_alignment(Alignment::Center), inner_layout[0]);
    frame.render_widget(input, input_area);
    frame.render_widget(messages, messages_area);
    frame.render_widget(search_bar_help, help_area);
    frame.render_widget(installation_mods_list, installed_mods_area);
    frame.render_widget(download_button, download_button_area);
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;


        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Right => {
                        app.input_mode = InputMode::InstalledModsSelecting;
                    }
                    _ => {}
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => app.send_query(),
                    KeyCode::Char(to_insert) => {
                        app.enter_char(to_insert);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    KeyCode::Left => {
                        app.move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_cursor_right();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::Editing => {},
                InputMode::SearchBarSelecting if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Up => {
                        if app.search_index > 0 {
                            app.search_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.search_index < app.search_list.len() - 1 {
                            app.search_index += 1;
                        }
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        app.send_project_to_installation();
                    }
                    _ => {},
                }
                InputMode::SearchBarSelecting => {},
                InputMode::InstalledModsSelecting if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Up => {
                        if app.installation_index > 0 {
                            app.installation_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.installation_index < app.installation_list.len() {
                            app.installation_index += 1;
                        }
                    }
                    KeyCode::Left => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Backspace => {
                        app.installation_list.remove(app.installation_index);
                        if app.installation_index > 0 {
                            app.installation_index -= 1;
                        }
                    }
                    KeyCode::Enter if app.installation_index == app.installation_list.len() && !app.installation_list.is_empty() => {

                        app.input_mode = InputMode::Downloading;
                        
                        
                        let download_links: Vec<String> = app.installation_list.iter().map(|project_name| {
                            let project_id = project_name.split("| ").last().unwrap();
                            parser::get_link_by_id(project_id, &app.mc_version).unwrap().to_string()
                        }).collect();

                        for link in download_links {
                            parser::download_project(&link).unwrap();
                        }
                        
                        app.installation_index = 0;
                        app.installation_list.clear();

                        
                    }
                    _ => {},
                }
                InputMode::InstalledModsSelecting => {},
                InputMode::Downloading if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {},
                }
                InputMode::Downloading => {},
            }
        }
    }
}
