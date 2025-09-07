# To-Do CLI (Rust)

A simple, colorful terminal To-Do app in Rust.  
Features a **TUI menu** (ratatui), **friendly prompts** (dialoguer), colored **table output** (prettytable + colored), and **JSON persistence**.

---

## Features

- Add tasks (auto-incrementing IDs)
- List tasks in a formatted table with colored statuses
- Update task status (`Todo` / `InProgress` / `Done`)
- Remove tasks by ID
- Auto-save & (load tasks soon) from `tasks.json`
- TUI menu hotkeys: `1–6`, `q` to quit
- *(Windows optional)* App icon embedding & non-resizable console window

---

## Screenshots

```
![Menu](https://github.com/user-attachments/assets/9d005a03-ccb4-42f1-b531-15801e3675c8)
![Creating A Task](https://github.com/user-attachments/assets/be0fbb58-5475-48b3-be77-291c5e0d6bce)
![Table Of Tasks](https://github.com/user-attachments/assets/20e35548-2030-4421-a41d-56baebce51d6)
```

---

## Requirements

- Rust (stable)
- Windows, macOS, or Linux  
  *(Windows-only enhancements noted below)*

---

## Build

```bash
# Debug
cargo build

# Release
cargo build --release
```

---

## Run

```bash
cargo run
# or after release build
target/release/main
```

---

## Usage

On launch you’ll see a full-screen menu:

```
1) Add task
2) List tasks
3) Remove task
4) Save (JSON)
5) Update status
6) Exit
```

- **Add**: interactive prompts for title / description / status  
- **List**: pretty table with colored status  
- **Remove**: choose a task to delete  
- **Save**: writes `tasks.json`
- **Update**: change status for a selected task  

---

## Data & Persistence

Tasks are stored as JSON at `./tasks.json` (working directory).

- After **add / update / remove**, the app you can save back to `tasks.json`.

- **Coming Soon:** On startup, the app loads `tasks.json` if it exists.

---

## Dependencies

Example `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
prettytable = "0.10"
colored = "3.0"
crossterm = "0.29"
ratatui = { version = "0.29", default-features = false, features = ["crossterm"] }
dialoguer = "0.12"

# Windows-only (optional UI tweaks)
windows = { version = "0.62", features = [
  "Win32_Foundation",
  "Win32_System_Console",
  "Win32_UI_WindowsAndMessaging"
] }

[build-dependencies]        # Windows icon embedding (optional)
winres = "0.1"
```

---

## Windows-only (optional)

### Embed an `.ico` App Icon

1. Place `icon.ico` next to `Cargo.toml`.  
2. Create `build.rs` in the project root:

```rust
fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        // Optional metadata:
        // res.set("FileDescription", "To-Do CLI");
        // res.set("ProductName", "To-Do CLI");
        res.compile().expect("Failed to compile Windows resources");
    }
}
```

3. Build with `cargo build --release` → `target\release\main.exe` includes the icon.

### Make the Console Non-Resizable (disable maximize)

Call early in `main()`:

```rust
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
        if hwnd.0.is_null() { return; }
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        let new_style = style & !(WS_MAXIMIZEBOX.0 as i32) & !(WS_THICKFRAME.0 as i32);
        SetWindowLongW(hwnd, GWL_STYLE, new_style);
        let _ = SetWindowPos(hwnd, None, 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);
    }
}
```

## Credits

- [ratatui](https://github.com/ratatui-org/ratatui)  
- [dialoguer](https://github.com/console-rs/dialoguer)  
- [prettytable-rs](https://github.com/phsym/prettytable-rs)
