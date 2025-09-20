# Configuration

Default config is stored at the following path on macOS:

- `~/Library/Application Support/ProjectBrowser/config.json`

Fields:
- `roots`: array of directories to scan. Default: `["~/Code"]`.
- `global_ignores`: additional patterns ignored in all scans. Default:
  - `.git`, `node_modules`, `target`, `build`, `dist`, `.venv`, `Pods`, `DerivedData`, `.cache`
- `size_mode`: one of `exact_cached` (default), `none`.
- `concurrency`: number of worker tasks. Default: `8`.
- `git.use_cli_fallback`: use `git` CLI if `git2` fails. Default: `false`.

Ignore precedence:
1. Repository/local `.gitignore`
2. App `global_ignores`
3. Optional user ignore files (gitignore-style, one pattern per line):
   - App config dir: `~/Library/Application Support/ProjectBrowser/ignore`
   - Legacy path: `~/.config/project-browser/ignore`

You can preview effective config via:
- `cargo run -p cli -- config --print`

You can run a dry scan preview via:
- `cargo run -p cli -- scan --dry-run`
