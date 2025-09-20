# Local Code Visualizer

A powerful desktop application for browsing, analyzing, and managing your local code projects. Built with Tauri (Rust + React) for fast performance and native desktop integration.

## Features

### 🔍 **Project Discovery & Analysis**
- **Automatic scanning** of your code directories
- **Multi-language support** (Python, Node.js, Rust, Java, etc.)
- **Lines of Code (LOC) analysis** with tokei integration
- **File size and project metrics** tracking
- **Git repository detection** and metadata

### 📊 **Interactive Project Browser**
- **Sortable columns** with ascending/descending order
- **Real-time search** and filtering
- **Pagination** with customizable page sizes (100, 250, 500, 1000)
- **Total project count** display
- **Last edit date** tracking with human-readable formatting

### 🚀 **Editor Integration**
- **One-click project opening** in your favorite editors
- **Windsurf integration** (`windsurf <path>`)
- **Cursor integration** (`cursor <path>`)
- **Fallback to clipboard** if editor not found

### 💫 **Modern UI/UX**
- **Professional dark theme** with zinc color palette
- **Responsive layout** with fixed header/footer
- **Loading indicators** with animated spinners
- **Human-readable file sizes** (B, KB, MB, GB, TB)
- **Relative date formatting** (Today, Yesterday, 3d ago, etc.)

## Installation

### Prerequisites
- **Rust** (latest stable)
- **Node.js** (v16+)
- **npm** or **yarn**

### Build from Source
```bash
# Clone the repository
git clone <repository-url>
cd local-code-visualizer

# Install dependencies and build
make setup
make build

# Run the desktop app
make tauri-run

# Or run with analysis features (recommended)
make tauri-run-analyzed
```

## Usage

### 🖥️ **Desktop Application**

1. **Launch the app**:
   ```bash
   make tauri-run-analyzed
   ```

2. **Scan your projects**:
   - Click the **"Scan"** button to discover projects
   - Default scan location: `$HOME/Code`
   - Scans recursively for project files

3. **Browse and sort**:
   - Click any **column header** to sort (Name, Type, Size, LOC, Last Edit)
   - Click again to **reverse sort direction**
   - Use the **search box** to filter projects

4. **Adjust view**:
   - Select **page size** (100, 250, 500, 1000 items)
   - Navigate with **Previous/Next** buttons
   - View **total project count** in footer

5. **Open projects**:
   - Click any **project path** to open "Open In..." modal
   - Choose **Windsurf** or **Cursor** to launch editor
   - Project opens directly in your chosen editor

### 🖱️ **CLI Interface**

```bash
# Scan projects
make run-scan

# List projects with analysis
make run-list-analyzed

# Show database path
make db-path

# Run full QA pipeline
make qa
```

### ⚙️ **Configuration**

Default scan roots can be configured in the application. The scanner looks for:
- **Python projects**: `requirements.txt`, `pyproject.toml`, `setup.py`
- **Node.js projects**: `package.json`
- **Rust projects**: `Cargo.toml`
- **Java projects**: `pom.xml`, `build.gradle`
- **Git repositories**: `.git` directories

## Development

### 🛠️ **Available Commands**

```bash
# Development
make setup          # Install Rust components
make build          # Build the workspace
make qa             # Run full QA pipeline

# Desktop App
make tauri-run              # Run Tauri app
make tauri-run-analyzed     # Run with analysis features

# Web Frontend
make web-dev        # Start development server
make web-build      # Build for production
make web-preview    # Preview built frontend

# CLI Tools
make run-scan       # Scan projects
make run-list       # List projects
make run-scan-analyzed      # Scan with analysis
make run-list-analyzed      # List with LOC info

# Code Quality
make fmt            # Check formatting
make fmt-fix        # Auto-format code
make lint           # Run clippy linter
make test           # Run tests
make clean          # Clean build artifacts
```

### 📁 **Project Structure**

```
local-code-visualizer/
├── src-tauri/              # Tauri backend (Rust)
│   ├── src/main.rs         # Main application logic
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── web/                    # Frontend (React + TypeScript)
│   ├── src/ui/App.tsx      # Main UI component
│   ├── package.json        # Node.js dependencies
│   └── vite.config.ts      # Vite configuration
├── crates/
│   ├── indexer/            # Core indexing logic
│   └── cli/                # Command-line interface
├── Makefile                # Build automation
└── README.md               # This file
```

### 🔧 **Architecture**

- **Backend**: Rust with Tauri for native desktop integration
- **Frontend**: React with TypeScript and Tailwind CSS
- **Database**: SQLite for project metadata storage
- **Analysis**: Tokei for lines of code counting
- **Build**: Vite for fast frontend bundling

## Troubleshooting

### Editor Integration Issues

If **Windsurf** or **Cursor** don't open:

1. **Check if editor is installed**:
   ```bash
   which windsurf
   which cursor
   ```

2. **Add to PATH** if needed:
   ```bash
   # For Windsurf
   export PATH="/Applications/Windsurf.app/Contents/Resources/app/bin:$PATH"
   
   # For Cursor
   export PATH="/Applications/Cursor.app/Contents/Resources/app/bin:$PATH"
   ```

3. **Fallback**: Commands are copied to clipboard if editor not found

### Performance Tips

- Use **page size 500** (default) for best performance
- **Scan periodically** to keep project data fresh
- Use **search/filter** for large project collections

## Contributing

1. Fork the repository
2. Create a feature branch
3. Run `make qa` to ensure code quality
4. Submit a pull request

## License

[Add your license here]

---

**Built with ❤️ using Tauri, Rust, and React**