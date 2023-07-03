use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, sync::mpsc, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Terminal,
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // capture keycode
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        if event::poll(Duration::from_millis(250)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                tx.send(key.code).unwrap();
            }
        }
    });

    // some todo samples
    let todos = vec![
        String::from("make a todo tui app"),
        String::from("learning rust"),
        String::from("make a cup of tea"),
    ];

    let dones = vec![
        String::from("read a rust manual"),
        String::from("read arch linux wiki"),
    ];

    let mut current_select_item_index: usize = 0;
    let mut current_tab_index: usize = 0;
    let labels = ["Tados", "Dones"];
    let mut contents = [todos, dones];
    let mut is_input = false;
    let mut input_content = String::new();

    loop {
        // draw the terminal that has main widgets Tabs, first Tab name "todos",
        // second Tab name "dones", first Tab contains a widget of List that contains ListItems of
        // todos,
        // second Tab contains a widget of List that contains ListItems of dones
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
                .split(size);
            let titles = labels.iter().cloned().map(Spans::from).collect();
            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"))
                .select(current_tab_index);
            f.render_widget(tabs, chunks[0]);

            let block = Block::default()
                .title(labels[current_tab_index])
                .borders(Borders::ALL);
            let items: Vec<ListItem> = contents[current_tab_index]
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    // ListItem::new(s.to_string()).style(Style::default().fg(
                    //     if i == current_select_item_index {
                    //         Color::Yellow
                    //     } else {
                    //         Color::White
                    //     },
                    // ))
                    let mut item = ListItem::new(s.to_string());
                    if current_tab_index == 1 {
                        // 如果在"已完成"标签页，添加删除线
                        item = item.style(
                            Style::default().add_modifier(Modifier::CROSSED_OUT | Modifier::ITALIC),
                        );
                    }
                    // 如果是当前选中项，设置颜色为黄色
                    if i == current_select_item_index {
                        item = item.style(Style::default().fg(Color::Yellow));
                    }
                    item
                })
                .collect::<Vec<ListItem>>();
            let list = List::new(items).block(block).highlight_symbol(">>");
            f.render_stateful_widget(list, chunks[1], &mut ListState::default());

            // add inputbar on the center of terminal when is_input is true
            if is_input {
                let area = centered_rect(60, 10, f.size());
                let inputbar = Block::default().title("Input").borders(Borders::ALL);
                // 在输入内容之后添加一个"|", 用来模拟光标
                let input_with_cursor = format!("{}|", input_content);
                let input = Paragraph::new(input_with_cursor)
                    .style(Style::default().fg(Color::White))
                    .block(inputbar)
                    .alignment(Alignment::Left);
                f.render_widget(input, area);
            }
        })?;

        match rx.recv() {
            Ok(KeyCode::Backspace) if is_input => {
                if !input_content.is_empty() {
                    input_content.pop();
                }
            }
            Ok(KeyCode::Char(c)) if is_input => {
                input_content.push(c);
            }
            Ok(KeyCode::Esc) if is_input => {
                is_input = false;
            }
            Ok(KeyCode::Char('a')) => {
                is_input = true;
            }
            Ok(KeyCode::Enter) if is_input => {
                contents[0].push(input_content.clone());
                input_content.clear();
                is_input = false;
            }
            Ok(KeyCode::Char(' ')) => {
                let item = contents[current_tab_index].remove(current_select_item_index);
                contents[current_tab_index ^ 1].push(item);
                current_select_item_index = std::cmp::min(
                    contents[current_tab_index].len() - 1,
                    current_select_item_index,
                );
            }
            Ok(KeyCode::Char('q')) => break,
            Ok(KeyCode::Char('c')) => {
                terminal.clear()?;
            }
            Ok(KeyCode::Char('e')) => {
                current_select_item_index =
                    (current_select_item_index + 1) % contents[current_tab_index].len();
            }
            Ok(KeyCode::Char('u')) => {
                current_select_item_index =
                    (current_select_item_index + contents[current_tab_index].len() - 1)
                        % contents[current_tab_index].len();
            }
            Ok(KeyCode::Backspace) => {
                if !contents[current_tab_index].is_empty() {
                    contents[current_tab_index].remove(current_select_item_index);
                    current_select_item_index = std::cmp::min(
                        contents[current_tab_index].len() - 1,
                        current_select_item_index,
                    );
                }
            }
            Ok(KeyCode::Tab) => {
                current_tab_index = (current_tab_index + 1) % labels.len();
            }
            Ok(KeyCode::BackTab) => {
                current_tab_index = (current_tab_index + labels.len() - 1) % labels.len();
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
