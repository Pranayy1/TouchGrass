# 🤝 Contributing to TouchGrass

Thank you for your interest in contributing to TouchGrass. Contributions of all sizes are appreciated, from bug reports to feature implementations and documentation improvements.

---

## Ways to Contribute

There are many ways to help:

- **Reporting bugs** — open an issue with steps to reproduce.
- **Suggesting features** — share your idea before building it.
- **Improving documentation** — fix typos, clarify instructions, add examples.
- **Improving UI/UX** — suggest or implement interface improvements.
- **Fixing bugs** — pick up an existing issue and submit a fix.
- **Refactoring code** — improve readability without changing behavior.
- **Performance improvements** — optimize slow paths and reduce resource usage.

---

## Before You Start

- **Search existing issues** before opening a new one to avoid duplicates.
- **Discuss large changes** in an issue before starting implementation. This saves effort and aligns direction.
- **Keep pull requests focused** — one PR per feature or fix. Avoid bundling unrelated changes.

---

## Development Setup

```bash
# Clone the repository
git clone https://github.com/Pranayy1/TouchGrass.git
cd TouchGrass

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

The compiled installer will be located in `src-tauri/target/release/bundle/`.

---

## Project Structure

- `src/` — Frontend code (vanilla JavaScript, HTML, CSS)
- `src/assets/` — Images and static resources
- `src-tauri/` — Rust backend powered by Tauri

---

## Coding Guidelines

- Use descriptive variable and function names.
- Keep functions focused on a single responsibility.
- Prefer readable code over clever code.
- Maintain the existing code style throughout the project.
- Avoid introducing unnecessary dependencies.
- Add comments only where they improve understanding of non-obvious logic.

---

## Commit Messages

This project follows [Conventional Commits](https://www.conventionalcommits.org/). Examples:

```
feat: add notification search

fix: resolve timer popup race condition

docs: update README download section

refactor: simplify notification manager
```

---

## Branch Naming

Use descriptive branch names to make collaboration easier:

```
feature/notification-search

feature/dark-mode

fix/timer-popup

docs/readme-update

refactor/storage-manager
```

---

## Pull Requests

- Test your changes before submitting.
- Keep PRs focused on a single topic.
- Update README, CHANGELOG, or other documentation whenever user-facing behavior changes.
- Include screenshots for UI changes.
- Reference related issues using `Closes #N` or `Fixes #N` where applicable.

---

## Testing

Before opening a Pull Request, verify the following:

- The project builds successfully (`npm run tauri dev` or `cargo check` in `src-tauri`).
- New functionality works as expected.
- Existing functionality is not broken.
- UI changes have been manually tested on Windows.

---

## Reporting Bugs

When opening an issue, please include:

- Operating system and version
- TouchGrass version
- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Screenshots (if applicable)

Please use GitHub Issues for bug reports only.

---

## Feature Requests

When suggesting a feature, please explain:

- The problem you are trying to solve
- Your proposed solution
- Any alternatives you considered

---

## Looking for Your First Contribution?

New contributors are welcome. Good starting points include:

- Documentation improvements
- UI polish and accessibility fixes
- Bug fixes
- Code cleanup
- Small feature enhancements

Starting small is encouraged. Familiarize yourself with the codebase and work your way up.

---

## Getting Help

- Open a **GitHub Issue** if you have questions about the codebase or need help with a bug.
- Use **GitHub Discussions** (if available) for general questions and ideas.
- Contact the maintainer when necessary through the repository's issue tracker.

Please reserve GitHub Issues for actual bugs and feature requests.

---

## Code of Conduct

Contributors are expected to follow the project's [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

---

## Thank You

Every contribution, no matter how small, helps make TouchGrass better. Thank you for taking the time to contribute.
