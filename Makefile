## Variables
CARGO ?= cargo
PACKAGE ?= project-browser
# Exclude the Tauri app from QA to avoid requiring a built frontend during lint/check/test
QA_EXCLUDES ?= --exclude app

## Composite targets
.PHONY: all setup build run check fmt fmt-fix lint test qa clean help

all: build

help: ## Show this help message
	@echo "Available targets:"
	@echo ""
	@echo "Build & Development:"
	@echo "  all          - Build the project (default target)"
	@echo "  setup        - Install Rust components (clippy, rustfmt)"
	@echo "  build        - Build the workspace"
	@echo "  run          - Run the application (with analysis features)"
	@echo "  clean        - Clean build artifacts"
	@echo ""
	@echo "Code Quality:"
	@echo "  check        - Run cargo check on workspace"
	@echo "  fmt          - Check code formatting"
	@echo "  fmt-fix      - Auto-format code in-place"
	@echo "  lint         - Run clippy linter"
	@echo "  test         - Run tests"
	@echo "  qa           - Run full QA pipeline (fmt-fix, lint, check, test)"
	@echo ""
	@echo "CLI Convenience:"
	@echo "  run-scan     - Scan $$HOME/Code directory"
	@echo "  run-list     - List recent projects (limit 50)"
	@echo "  db-path      - Show database path"
	@echo ""
	@echo "CLI with Analysis Features:"
	@echo "  run-scan-analyzed  - Scan with git & analyzer features"
	@echo "  run-list-analyzed  - List projects with LOC info"
	@echo ""
	@echo "Tauri App:"
	@echo "  tauri-run          - Run Tauri desktop app"
	@echo "  tauri-run-analyzed - Run Tauri app with analysis features"
	@echo ""
	@echo "Web Frontend:"
	@echo "  web-build    - Build web frontend"
	@echo "  web-dev      - Start web development server"
	@echo "  web-preview  - Preview built web frontend"
	@echo ""
	@echo "Use 'make <target>' to run a specific target."

setup:
	@echo "Installing Rust components (clippy, rustfmt)"
	rustup component add clippy rustfmt || true

build:
	@echo "Building web frontend..."
	$(MAKE) web-build
	@echo "Building Rust workspace..."
	$(CARGO) build --workspace

run:
	@echo "Running application with analysis features..."
	$(MAKE) tauri-run-analyzed

check:
	$(CARGO) check --workspace --all-targets $(QA_EXCLUDES)

fmt:
	$(CARGO) fmt --all -- --check

# Auto-format code in-place
fmt-fix:
	$(CARGO) fmt --all

lint:
	$(CARGO) clippy --workspace --all-targets $(QA_EXCLUDES) -- -D warnings

test:
	$(CARGO) test --workspace --all-targets $(QA_EXCLUDES) -- --nocapture

# QA auto-fixes formatting, then runs lints, checks, tests, and build
qa: fmt-fix lint check test

clean:
	$(CARGO) clean

# Convenience run targets
.PHONY: run-scan run-list db-path

run-scan:
	$(CARGO) run -p cli -- scan --root $$HOME/Code

run-list:
	$(CARGO) run -p cli -- list --sort recent --limit 50

db-path:
	$(CARGO) run -p cli -- config --db-path

.PHONY: run-scan-analyzed run-list-analyzed tauri-run-analyzed
run-scan-analyzed:
	$(CARGO) run -p cli -F git,analyzers -- scan --root $$HOME/Code

run-list-analyzed:
	$(CARGO) run -p cli -F git,analyzers -- list --sort loc --limit 50 --show-loc

tauri-run-analyzed:
	$(CARGO) run -p app -F git,analyzers

.PHONY: tauri-run
tauri-run:
	# Ensure frontend is built before launching Tauri
	@if [ ! -d dist ]; then \
		echo "Building web frontend (dist/ missing)..."; \
		$(MAKE) web-build; \
	fi
	$(CARGO) run -p app

.PHONY: web-build web-dev web-preview
web-build:
	npm --prefix web run build
web-dev:
	npm --prefix web run dev
web-preview:
	npm --prefix web run preview
