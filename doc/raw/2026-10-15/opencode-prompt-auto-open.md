# Task Prompt — Auto-open HTML report in default browser

Add automatic browser launching to `rustydeps scan` upon successful HTML report generation.

## Technical Requirements
1. **Dependency**: Add the `open` crate (or `webbrowser`) to `Cargo.toml`.
2. **CLI Flag (`src/cli.rs`)**:
   - Add a `--no-open` flag (boolean) to suppress opening the browser.
   - Opening should happen automatically when generating an HTML report (`--format html` or `-o`), UNLESS `--no-open` or `--format json` is specified.
3. **Execution (`src/main.rs`)**:
   - After writing the HTML report file and printing the path / `file://` link:
     - Check if auto-open is enabled.
     - Call `open::that(&report_path)` to launch the report in the user's default browser.
     - Log a warning if opening fails, but do NOT crash or fail the scan command.
4. **Verification**:
   - `cargo fmt --all --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

Write a brief task document in `doc/raw/` detailing the change and execute the updates cleanly.
