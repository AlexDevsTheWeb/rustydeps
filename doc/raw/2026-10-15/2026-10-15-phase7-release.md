# Task Prompt — Phase 6: README Update & Release v0.1.0 Automation

## Part 1: Update README.md
Update the repository  to include:
1. Live website link: https://alexdevstheweb.github.io/rustydeps/
2. ASCII Banner & Version v0.1.0 badges
3. Supported Ecosystems table (Cargo, npm, PyPI, Go, Composer)
4. Quickstart commands (, )
5. Feature overview (--offline, --no-open, --format json, auto-open report)

## Part 2: GitHub Actions CI/CD for Release Automation
1. Verify  has .
2. Create  with automated release workflow:
   - Triggers on push of tags matching  (e.g. ).
   - Builds release binaries for Linux () and macOS.
   - Automatically attaches compiled binaries and changelog summary to the GitHub Release.
3. Tag git release  and push tag to origin.
