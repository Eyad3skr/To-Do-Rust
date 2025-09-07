use std::io::{self, Write};

use colored::*;
use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};

// ======================
// Domain types & helpers
// ======================

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: u32,
    title: String,
    description: String,
    status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl Task {
    fn new(id: u32, title: String, description: String, status: TaskStatus) -> Task {
        Task { id, title, description, status }
    }
}

use dialoguer::{theme::ColorfulTheme, Input, Select, Confirm};

fn prompt_status(theme: &ColorfulTheme, prompt: &str) -> Option<TaskStatus> {
    let statuses = ["Todo", "InProgress", "Done"];
    let idx = Select::with_theme(theme)
        .with_prompt(prompt)
        .items(&statuses)
        .default(0)
        .interact()
        .ok()?;
    Some(match statuses[idx] {
        "Todo" => TaskStatus::Todo,
        "InProgress" => TaskStatus::InProgress,
        _ => TaskStatus::Done,
    })
}

fn prompt_add_task(next_id: u32) -> Option<Task> {
    let theme = ColorfulTheme::default();

    let title: String = Input::with_theme(&theme)
        .with_prompt("Title")
        .validate_with(|s: &String| {
            if s.trim().is_empty() { Err("Title cannot be empty") } else { Ok(()) }
        })
        .interact_text()
        .ok()?;

    let description: String = Input::with_theme(&theme)
        .with_prompt("Description")
        .allow_empty(true)
        .interact_text()
        .ok()?;

    let status = prompt_status(&theme, "Status")?;

    Some(Task::new(next_id, title.trim().into(), description.trim().into(), status))
}

fn prompt_select_task_id(tasks: &[Task], prompt: &str) -> Option<u32> {
    if tasks.is_empty() {
        println!("No tasks available.");
        return None;
    }
    let theme = ColorfulTheme::default();
    let items: Vec<String> = tasks.iter()
        .map(|t| format!("#{:<3} {:<12} {}", t.id, format!("{:?}", t.status), t.title))
        .collect();

    let idx = Select::with_theme(&theme)
        .with_prompt(prompt)
        .items(&items)
        .default(0)
        .interact()
        .ok()?;
    Some(tasks[idx].id)
}

fn prompt_confirm(theme: &ColorfulTheme, msg: &str) -> bool {
    Confirm::with_theme(theme)
        .with_prompt(msg)
        .default(true)
        .interact()
        .unwrap_or(false)
}

// fn parse_status(s: &str) -> Option<TaskStatus> {
//     match s.trim().to_ascii_lowercase().as_str() {
//         "todo" => Some(TaskStatus::Todo),
//         "inprogress" | "in_progress" | "in progress" => Some(TaskStatus::InProgress),
//         "done" => Some(TaskStatus::Done),
//         _ => None,
//     }
// }

fn add_task(tasks: &mut Vec<Task>, task: Task) {
    tasks.push(task);
    println!("Task added successfully.");
}

fn remove_task(tasks: &mut Vec<Task>, id: u32) {
    let before = tasks.len();
    tasks.retain(|t| t.id != id);
    if tasks.len() < before {
        println!("Task with ID {} removed successfully.", id);
    } else {
        println!("Task with ID {} not found.", id);
    }
}

fn list_tasks(tasks: &[Task]) {
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("ID").style_spec("bFg"),
        Cell::new("Title").style_spec("bFc"),
        Cell::new("Description").style_spec("bFy"),
        Cell::new("Status").style_spec("bFr"),
    ]));

    for t in tasks {
        let status = match t.status {
            TaskStatus::Todo => "Todo".yellow().to_string(),
            TaskStatus::InProgress => "In Progress".blue().to_string(),
            TaskStatus::Done => "Done".green().to_string(),
        };
        table.add_row(Row::new(vec![
            Cell::new(&t.id.to_string()),
            Cell::new(&t.title),
            Cell::new(&t.description),
            Cell::new(&status),
        ]));
    }
    table.printstd();
}

fn wait_enter() {
    print!("\nPress Enter to continue...");
    let _ = io::stdout().flush();
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
}

// ==============
// TUI (ratatui)
// ==============

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};


#[derive(Copy, Clone, Debug)]
enum MenuChoice {
    Add = 1,
    List = 2,
    Remove = 3,
    Save = 4,
    Update = 5,
    Exit = 6,
}

struct MenuLine {
    title: &'static str,
    sub:   &'static str,
    right: &'static str,
}

fn draw_divider_line(f: &mut Frame, inner: Rect, y: u16) {
    if inner.height == 0 { return; }
    if y < inner.y || y >= inner.y + inner.height { return; }
    let line = symbols::line::THICK_HORIZONTAL.repeat(inner.width as usize);
    let p = Paragraph::new(line).style(Style::default().fg(Color::Gray));
    f.render_widget(p, Rect::new(inner.x, y, inner.width, 1));
}

fn draw_menu(f: &mut Frame, area: Rect, items: &[MenuLine]) {
    // Outer box
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            " header ",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(outer, area);

    // Inner content area
    let inner = area.inner(Margin { horizontal: 2, vertical: 1 });
    if inner.height == 0 { return; }
    let y_min = inner.y;
    let y_max = inner.y + inner.height - 1; // last valid row

    // Cursor row
    let mut y = y_min;

    // Helper to render a single-line Paragraph at `y` and advance y safely
    fn render_line(f: &mut Frame, inner: Rect, y: &mut u16, y_max: u16, p: Paragraph, align: Alignment) {
        if *y <= y_max {
            let mut w = p;
            // set alignment on a copy (Paragraph builder style)
            w = w.alignment(align);
            f.render_widget(w, Rect::new(inner.x, *y, inner.width, 1));
        }
        *y = y.saturating_add(1);
    }

    for (i, it) in items.iter().enumerate() {
        // Title (left) and Right label (same row)
        if y <= y_max {
            let row = Rect::new(inner.x, y, inner.width, 1);

            let title = Paragraph::new(Line::from(Span::styled(
                it.title,
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Left);

            let right = Paragraph::new(Line::from(Span::styled(
                it.right,
                Style::default().fg(Color::Magenta),
            )))
            .alignment(Alignment::Right);

            // Render both on the same row
            f.render_widget(title, row);
            f.render_widget(right, row);
        }
        y = y.saturating_add(1);

        // Subtitle line
        let sub = Paragraph::new(Line::from(Span::styled(
            it.sub,
            Style::default().fg(Color::Gray),
        )));
        render_line(f, inner, &mut y, y_max, sub, Alignment::Left);

        // Divider between items
        if i < items.len() - 1 {
            // optional blank spacer
            render_line(f, inner, &mut y, y_max, Paragraph::new(""), Alignment::Left);
            draw_divider_line(f, inner, y);
            y = y.saturating_add(1);
        }

        // Stop if we ran out of vertical space
        if y > y_max { break; }
    }

    // Footer hint on the **last valid row** of the outer area
    if area.height > 0 {
        let footer_y = area.y + area.height - 1;
        let hint = Paragraph::new(Line::from(vec![
            Span::raw("Press "),
            Span::styled("1-6", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" to select • "),
            Span::styled("q", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" to quit"),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
        f.render_widget(hint, Rect::new(area.x, footer_y, area.width, 1));
    }
}


fn run_menu_tui() -> io::Result<Option<MenuChoice>> {
    let items = [
        MenuLine { title: "1) Add task",        sub: "Create a new task (auto-ID)",                  right: "default" },
        MenuLine { title: "2) List tasks",      sub: "Pretty table with colored status",             right: "view"    },
        MenuLine { title: "3) Remove task",     sub: "Delete by ID",                                 right: "danger"  },
        MenuLine { title: "4) Save (JSON)",     sub: "Write tasks.json (pretty JSON)",               right: "persist" },
        MenuLine { title: "5) Update status",   sub: "Change Todo/InProgress/Done by ID",            right: "edit"    },
        MenuLine { title: "6) Exit",            sub: "Close program",                                right: "quit"    },
    ];

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let choice = loop {
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(area);
            draw_menu(f, chunks[0], &items);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('1') => break Some(MenuChoice::Add),
                    KeyCode::Char('2') => break Some(MenuChoice::List),
                    KeyCode::Char('3') => break Some(MenuChoice::Remove),
                    KeyCode::Char('4') => break Some(MenuChoice::Save),
                    KeyCode::Char('5') => break Some(MenuChoice::Update),
                    KeyCode::Char('6') | KeyCode::Esc => break Some(MenuChoice::Exit),
                    KeyCode::Char('q') => break None,
                    _ => {}
                }
            }
        }
    };

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(choice)
}

#[cfg(windows)]
fn disable_resize() {
    use windows::Win32::System::Console::GetConsoleWindow;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_STYLE,
        WS_MAXIMIZEBOX, WS_THICKFRAME,
        SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    };

    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.0.is_null() {
            return; // no console window (e.g., detached)
        }

        let style = GetWindowLongW(hwnd, GWL_STYLE);
        let new_style = style & !(WS_MAXIMIZEBOX.0 as i32) & !(WS_THICKFRAME.0 as i32);
        SetWindowLongW(hwnd, GWL_STYLE, new_style);

        // Apply style changes immediately
        let _ = SetWindowPos(
            hwnd,
            None, // <- Option<HWND>
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
        );
    }
}


#[cfg(windows)]
fn maybe_relaunch_in_terminal() -> bool {
    use std::{env, fs, path::PathBuf, process::Command};

    if env::var("RUN_IN_TERM").is_ok() {
        return false; // already relaunched
    }

    // 1) Current exe
    let exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => { eprintln!("current_exe() failed: {e}"); return false; }
    };
    if !exe.exists() {
        eprintln!("Executable not found: {}", exe.display());
        return false;
    }

    // 2) Create ps1 in %TEMP%
    let mut ps1: PathBuf = env::temp_dir();
    ps1.push("launch_my_app.ps1");

    // Use double quotes; PowerShell treats this as a literal path invocation.
    let script = format!("& \"{}\"\n", exe.display());
    if let Err(e) = fs::write(&ps1, &script) {
        eprintln!("Failed to write temp ps1: {e}");
        return false;
    }
    if !ps1.exists() {
        eprintln!("Temp ps1 not found: {}", ps1.display());
        return false;
    }
    eprintln!("Temp script at: {}", ps1.display()); // debug print

    // 3) Full path to Windows PowerShell (avoid PATH issues)
    let sysroot = env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let ps_exe = format!(
        r"{}\System32\WindowsPowerShell\v1.0\powershell.exe",
        sysroot
    );

    // 4) Start a NEW window running that script
    let spawn_res = Command::new("cmd")
        .args([
            "/c", "start", "", &ps_exe,
            "-NoLogo", "-NoExit", "-ExecutionPolicy", "Bypass",
            "-File", &ps1.display().to_string(),
        ])
        .env("RUN_IN_TERM", "1")
        .spawn();

    match spawn_res {
        Ok(_) => true, // spawned → caller should exit
        Err(e) => {
            eprintln!("Failed to start PowerShell: {e}");
            false
        }
    }
}



// ===================
// Program entry point
// ===================

fn main() -> io::Result<()> {
#[cfg(windows)]
    {
        if maybe_relaunch_in_terminal() {
            // Exit the original process cleanly (type is io::Result<()>)
            return Ok(());
        }
    }

    #[cfg(windows)]
    disable_resize();

    let mut tasks: Vec<Task> = Vec::new();
    let mut next_id: u32 = 1;

    loop {
        // Show the TUI menu; returns a choice or None (q)
        let Some(choice) = run_menu_tui()? else { break };

        match choice {
            MenuChoice::Add => {
                if let Some(task) = prompt_add_task(next_id) {
                    add_task(&mut tasks, task);
                    next_id += 1;
                }
                wait_enter();
            }

 MenuChoice::List => {
                if tasks.is_empty() {
                    println!("No tasks yet.");
                } else {
                    list_tasks(&tasks);
                }
                wait_enter();
            }

            MenuChoice::Remove => {
                if let Some(id) = prompt_select_task_id(&tasks, "Pick a task to remove") {
                    let theme = ColorfulTheme::default();
                    if prompt_confirm(&theme, &format!("Delete task #{}?", id)) {
                        remove_task(&mut tasks, id);
                    } else {
                        println!("Cancelled.");
                    }
                }
                wait_enter();
            }

            MenuChoice::Save => {
                let json = serde_json::to_string_pretty(&tasks).unwrap();
                match std::fs::write("tasks.json", json) {
                    Ok(_) => println!("Saved to tasks.json"),
                    Err(e) => println!("Failed to save: {e}"),
                }
                wait_enter();
            }

            MenuChoice::Update => {
                if let Some(id) = prompt_select_task_id(&tasks, "Pick a task to update") {
                    let theme = ColorfulTheme::default();
                    if let Some(new_status) = prompt_status(&theme, "New status") {
                        let mut found = false;
                        for t in &mut tasks {
                            if t.id == id {
                                t.status = new_status.clone();
                                found = true;
                                println!("Task #{} updated.", id);
                                break;
                            }
                        }
                        if !found {
                            println!("Task not found.");
                        }
                    }
                }
                wait_enter();
            }

            MenuChoice::Exit => {
                let theme = ColorfulTheme::default();
                if prompt_confirm(&theme, "Quit?") {
                    break;
                }
            }
        }
    }

    println!("Goodbye!");
    Ok(())
}
