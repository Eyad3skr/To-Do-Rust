fn main() {
    // Only add resources on Windows builds
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        // Path to your .ico (relative to project root)
        res.set_icon("icon.ico");

        // Optional version metadata (shows in File Properties → Details)
        res.set("FileDescription", "CLI To Do App");
        res.set("ProductName", "Nebula To Do");
        res.set("CompanyName", "Your Team");
        res.set("LegalCopyright", "© 2025 Eyad Radwan");

        res.compile().expect("Failed to compile Windows resources");
    }
}
