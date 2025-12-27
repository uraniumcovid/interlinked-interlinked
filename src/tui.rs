use crate::{FileIndex};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io;

#[derive(Debug, Clone, Copy)]
enum Tab {
    Links,
    Backlinks,
    Tags,
}

impl Tab {
    fn titles() -> Vec<&'static str> {
        vec!["Links", "Backlinks", "Tags"]
    }
    
    fn from_index(index: usize) -> Self {
        match index {
            0 => Tab::Links,
            1 => Tab::Backlinks,
            2 => Tab::Tags,
            _ => Tab::Links,
        }
    }
}

pub struct App {
    current_tab: Tab,
    tab_index: usize,
    list_state: ListState,
    index: FileIndex,
    search_mode: bool,
    search_query: String,
    filtered_items: Vec<String>,
}

impl App {
    pub fn new(index: FileIndex) -> Self {
        let mut app = Self {
            current_tab: Tab::Links,
            tab_index: 0,
            list_state: ListState::default(),
            index,
            search_mode: false,
            search_query: String::new(),
            filtered_items: Vec::new(),
        };
        app.update_filtered_items();
        app.list_state.select(Some(0));
        app
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if self.search_mode {
                        match key.code {
                            KeyCode::Enter => {
                                self.search_mode = false;
                            }
                            KeyCode::Esc => {
                                self.search_mode = false;
                                self.search_query.clear();
                                self.update_filtered_items();
                            }
                            KeyCode::Char(c) => {
                                self.search_query.push(c);
                                self.update_filtered_items();
                            }
                            KeyCode::Backspace => {
                                self.search_query.pop();
                                self.update_filtered_items();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('/') => {
                                self.search_mode = true;
                                self.search_query.clear();
                            }
                            KeyCode::Tab => self.next_tab(),
                            KeyCode::BackTab => self.previous_tab(),
                            KeyCode::Down => self.next_item(),
                            KeyCode::Up => self.previous_item(),
                            KeyCode::Char('j') => self.next_item(),
                            KeyCode::Char('k') => self.previous_item(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % 3;
        self.current_tab = Tab::from_index(self.tab_index);
        self.update_filtered_items();
        self.list_state.select(Some(0));
    }

    fn previous_tab(&mut self) {
        self.tab_index = if self.tab_index > 0 { self.tab_index - 1 } else { 2 };
        self.current_tab = Tab::from_index(self.tab_index);
        self.update_filtered_items();
        self.list_state.select(Some(0));
    }

    fn next_item(&mut self) {
        if !self.filtered_items.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i >= self.filtered_items.len() - 1 { 0 } else { i + 1 }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    fn previous_item(&mut self) {
        if !self.filtered_items.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 { self.filtered_items.len() - 1 } else { i - 1 }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    fn update_filtered_items(&mut self) {
        let mut items: Vec<(String, usize)> = match self.current_tab {
            Tab::Links => {
                self.index.backlinks.iter()
                    .map(|(link, sources)| (link.clone(), sources.len()))
                    .collect()
            }
            Tab::Backlinks => {
                self.index.backlinks.iter()
                    .map(|(link, sources)| (link.clone(), sources.len()))
                    .collect()
            }
            Tab::Tags => {
                self.index.tags.iter()
                    .map(|(tag, files)| (tag.clone(), files.len()))
                    .collect()
            }
        };

        // Sort by connection count (descending) then alphabetically
        items.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        if self.search_query.is_empty() {
            self.filtered_items = items.into_iter().map(|(name, _)| name).collect();
        } else {
            self.filtered_items = items
                .into_iter()
                .filter(|(name, _)| name.to_lowercase().contains(&self.search_query.to_lowercase()))
                .map(|(name, _)| name)
                .collect();
        }

        if !self.filtered_items.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
    }

    fn get_selected_item_details(&self) -> Vec<Line<'_>> {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(selected_item) = self.filtered_items.get(selected_idx) {
                match self.current_tab {
                    Tab::Links | Tab::Backlinks => {
                        if let Some(sources) = self.index.backlinks.get(selected_item) {
                            let mut lines = vec![
                                Line::from(vec![
                                    Span::styled("[[", Style::default().fg(Color::Rgb(255, 107, 138))),
                                    Span::styled(selected_item, Style::default().fg(Color::White).bold()),
                                    Span::styled("]]", Style::default().fg(Color::Rgb(255, 107, 138))),
                                    Span::styled(format!(" ({} connections)", sources.len()), 
                                        Style::default().fg(Color::Rgb(255, 107, 138))),
                                ]),
                                Line::from(""),
                                Line::from(Span::styled("Connected files:", Style::default().fg(Color::Rgb(255, 107, 138)).bold())),
                            ];
                            
                            for (i, source) in sources.iter().enumerate() {
                                if i >= 20 { // Limit display to avoid overwhelming
                                    lines.push(Line::from(Span::styled(
                                        format!("  ... and {} more", sources.len() - i),
                                        Style::default().fg(Color::White).italic()
                                    )));
                                    break;
                                }
                                lines.push(Line::from(vec![
                                    Span::styled("  • ", Style::default().fg(Color::Rgb(255, 107, 138))),
                                    Span::styled(source, Style::default().fg(Color::White)),
                                ]));
                            }
                            return lines;
                        }
                    }
                    Tab::Tags => {
                        if let Some(files) = self.index.tags.get(selected_item) {
                            let mut lines = vec![
                                Line::from(vec![
                                    Span::styled("#", Style::default().fg(Color::Rgb(255, 107, 138))),
                                    Span::styled(selected_item, Style::default().fg(Color::White).bold()),
                                    Span::styled(format!(" ({} files)", files.len()), 
                                        Style::default().fg(Color::Rgb(255, 107, 138))),
                                ]),
                                Line::from(""),
                                Line::from(Span::styled("Tagged files:", Style::default().fg(Color::Rgb(255, 107, 138)).bold())),
                            ];
                            
                            for (i, file) in files.iter().enumerate() {
                                if i >= 20 {
                                    lines.push(Line::from(Span::styled(
                                        format!("  ... and {} more", files.len() - i),
                                        Style::default().fg(Color::White).italic()
                                    )));
                                    break;
                                }
                                lines.push(Line::from(vec![
                                    Span::styled("  • ", Style::default().fg(Color::Rgb(255, 107, 138))),
                                    Span::styled(file, Style::default().fg(Color::White)),
                                ]));
                            }
                            return lines;
                        }
                    }
                }
            }
        }
        
        vec![Line::from(Span::styled(
            "Select an item to see details...",
            Style::default().fg(Color::White).italic()
        ))]
    }

    fn ui(&mut self, f: &mut Frame) {
        let size = f.area();
        
        // Set black background for entire terminal
        let background = Block::default()
            .style(Style::default().bg(Color::Black));
        f.render_widget(background, size);

        // Main layout: tabs, content, help
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tabs
                Constraint::Min(0),     // Content area
                Constraint::Length(3),  // Help
            ])
            .split(size);

        // Tabs with improved styling
        let tab_titles: Vec<Line> = Tab::titles()
            .iter()
            .map(|t| Line::from(*t))
            .collect();
        let tabs = Tabs::new(tab_titles)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" 🔗 Interlinked ")
                .title_style(Style::default().fg(Color::White).bold())
                .border_style(Style::default().fg(Color::Rgb(255, 107, 138))))
            .select(self.tab_index)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Black).bold().bg(Color::Rgb(255, 107, 138)));
        f.render_widget(tabs, main_chunks[0]);

        // Content area: split into list and details
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),  // List
                Constraint::Percentage(60),  // Details
            ])
            .split(main_chunks[1]);

        // Create styled list items with connection counts
        let list_items: Vec<ListItem> = self
            .filtered_items
            .iter()
            .map(|item| {
                let count = match self.current_tab {
                    Tab::Links | Tab::Backlinks => {
                        self.index.backlinks.get(item).map(|v| v.len()).unwrap_or(0)
                    }
                    Tab::Tags => {
                        self.index.tags.get(item).map(|v| v.len()).unwrap_or(0)
                    }
                };

                let icon_style = match self.current_tab {
                    Tab::Links | Tab::Backlinks => Style::default().fg(Color::Rgb(255, 107, 138)),
                    Tab::Tags => Style::default().fg(Color::Rgb(255, 107, 138)),
                };

                let icon = match self.current_tab {
                    Tab::Links | Tab::Backlinks => "[[",
                    Tab::Tags => "#",
                };

                let line = Line::from(vec![
                    Span::styled(icon, icon_style),
                    Span::styled(item, Style::default().fg(Color::White).bold()),
                    Span::styled(
                        if matches!(self.current_tab, Tab::Links | Tab::Backlinks) { "]]" } else { "" },
                        icon_style
                    ),
                    Span::styled(
                        format!(" ({})", count),
                        Style::default().fg(Color::Rgb(255, 107, 138))
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list_title = match self.current_tab {
            Tab::Links => format!(" 🔗 Links ({}) ", self.filtered_items.len()),
            Tab::Backlinks => format!(" ↩️  Backlinks ({}) ", self.filtered_items.len()),
            Tab::Tags => format!(" 🏷️  Tags ({}) ", self.filtered_items.len()),
        };

        let list = List::new(list_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .title_style(Style::default().fg(Color::White).bold())
                .border_style(Style::default().fg(Color::Rgb(255, 107, 138))))
            .highlight_style(Style::default().bg(Color::Rgb(255, 107, 138)).fg(Color::Black).bold());

        f.render_stateful_widget(list, content_chunks[0], &mut self.list_state);

        // Details panel
        let details = self.get_selected_item_details();
        let details_paragraph = Paragraph::new(details)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" 📄 Details ")
                .title_style(Style::default().fg(Color::White).bold())
                .border_style(Style::default().fg(Color::Rgb(255, 107, 138))))
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(details_paragraph, content_chunks[1]);

        // Help bar with search
        let help_text = if self.search_mode {
            "ESC: Exit search | Enter: Finish search | Type to filter"
        } else {
            "q: Quit | Tab/Shift+Tab: Switch views | ↑/↓ or j/k: Navigate | /: Search"
        };

        let search_indicator = if self.search_mode {
            format!(" 🔍 {}", self.search_query)
        } else if !self.search_query.is_empty() {
            format!(" 🔍 {} (filtered)", self.search_query)
        } else {
            String::new()
        };

        let help_content = vec![
            Span::styled(help_text, Style::default().fg(Color::White)),
            if !search_indicator.is_empty() {
                Span::styled(search_indicator, Style::default().fg(Color::Rgb(255, 107, 138)).bold())
            } else {
                Span::raw("")
            },
        ];

        let help = Paragraph::new(Line::from(help_content))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(255, 107, 138))));
        f.render_widget(help, main_chunks[2]);
    }
}

pub fn run_tui(index: FileIndex) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(index);
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res?;
    Ok(())
}