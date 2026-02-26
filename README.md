
## 🚀 Getting Started

    1. Install Rust: Download the toolchain from rust-lang.org/tools/install.

    2. Setup IDE: Install rust-analyzer and CodeLLDB extensions in VS Code for a better developer experience.

    3. Clone Repository: ```bash
    git clone https://www.google.com/search?q=https://github.com/your-username/project-name.git
    cd project-name

    4. Build & Run: ```bash
    cargo run

    5. Build for production ```bash 
    cargo build --release

## 📂 Project Structure

```text
src/
├── main.rs          # Entry point
├── messages.rs      # Enums for application events (Message, SshMessage, etc.)
├── models.rs        # Data structures (Profile, EditSection, Config)
├── ssh.rs           # Network logic and russh implementation
├── ui.rs            # Main UI controller (App state, update, and view)
└── ui/              # Private UI module
    ├── constants.rs # Layout constants (paddings, font sizes, etc.)
    ├── dashboard.rs # Main window (profile, ...)
    ├── terminal.rs  # Terminal emulation and rendering logic
    ├── theme.rs     # Styling, colors, and custom widget themes
    ├── ui.rs        # View router / navigation logic
    ├── views/       # High-level application screens
    └── components/  # Reusable UI building blocks (forms, buttons, etc.)
```

