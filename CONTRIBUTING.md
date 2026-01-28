# Contributing to RevGame

Thank you for your interest in contributing to RevGame! This document provides guidelines and information for contributors.

## 🎯 Ways to Contribute

### 1. 🐛 Report Bugs
Found a bug? Help us fix it:
- Check if the bug is already reported in [Issues](https://github.com/yourusername/revgame/issues)
- If not, open a new issue with:
  - Clear title describing the problem
  - Steps to reproduce
  - Expected vs actual behavior
  - Your environment (OS, Rust version, terminal emulator)
  - Screenshots if relevant

### 2. 💡 Suggest Features
Have an idea? We'd love to hear it:
- Check existing [Discussions](https://github.com/yourusername/revgame/discussions) first
- Open a new discussion describing:
  - What problem does it solve?
  - How would it work?
  - Why would it benefit users?

### 3. 🧩 Create Puzzles
The easiest way to contribute! We always need more puzzles:
- Follow the [puzzle format](#puzzle-format) below
- Start with beginner-friendly puzzles
- Include clear descriptions and hints
- Test your puzzle thoroughly
- Submit via Pull Request

### 4. 📖 Improve Documentation
Help make RevGame more accessible:
- Fix typos or unclear explanations
- Add examples to the README
- Write tutorials or guides
- Improve inline code comments

### 5. 💻 Code Contributions
Ready to dive into the code:
- Pick an issue labeled `good first issue`
- Comment that you're working on it
- Follow the [development guide](#development-guide)
- Submit a well-tested Pull Request

## 🔧 Development Guide

### Setup

1. **Fork and clone:**
   ```bash
   git clone https://github.com/yourusername/revgame.git
   cd revgame
   ```

2. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Install development tools:**
   ```bash
   cargo install cargo-watch
   cargo install cargo-fmt
   cargo install cargo-clippy
   ```

4. **Build and run:**
   ```bash
   cargo build
   cargo run -p revgame-native
   ```

### Development Workflow

1. **Create a branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make changes:**
   - Write clear, documented code
   - Follow Rust conventions
   - Add tests for new functionality

3. **Test your changes:**
   ```bash
   # Run all tests
   cargo test

   # Run specific tests
   cargo test -p revgame-core

   # Run with output
   cargo test -- --nocapture
   ```

4. **Format and lint:**
   ```bash
   # Format code
   cargo fmt

   # Run clippy (linter)
   cargo clippy -- -D warnings
   ```

5. **Commit:**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

6. **Push and create PR:**
   ```bash
   git push origin feature/your-feature-name
   ```

### Code Style

- **Format:** Use `cargo fmt` (rustfmt)
- **Linting:** Pass `cargo clippy` with zero warnings
- **Comments:** Document public APIs with `///`
- **Tests:** Add tests for new functionality
- **Errors:** Use proper error types (thiserror)

### Commit Messages

Follow conventional commits:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Adding or fixing tests
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `chore:` - Maintenance tasks

Examples:
```
feat: add undo/redo system for memory patches
fix: correct JZ condition flag check
docs: update puzzle creation guide
test: add tests for achievement tracking
```

## 🧩 Puzzle Format

Puzzles are defined in TOML format. Here's a complete example:

```toml
[metadata]
id = "category-001"              # Unique ID
title = "Puzzle Title"           # Display name
difficulty = 1                   # 1-5 scale
category = "basics"              # Category name
tags = ["nop", "beginner"]       # Searchable tags
estimated_time_minutes = 5       # Expected solve time
prerequisites = []               # Required puzzle IDs

[description]
brief = "One-line summary"
detailed = """
Detailed description with multiple lines.
Explain what the player needs to do.
Give context about the challenge.
"""

[setup]
memory_size = 16384              # Total memory size
code_start = 0x1000              # Code section start
data_start = 0x2000              # Data section start
stack_start = 0x3000             # Stack pointer initial value

[setup.registers]                # Optional: Set initial registers
eax = 0x42
ebx = 0x1337

[setup.code]
# Assembly comments explaining the code
bytes = "90 90 C3"               # Hex bytes (space-separated)
entry_point = 0                  # Offset from code_start

[setup.data]                     # Optional: Initialize data section
bytes = "48 65 6C 6C 6F"         # Hex bytes

[validation]
type = "register_value"          # or "memory_value", "flag_set"
register = "eax"                 # Which register to check
expected = 1                     # Expected value

# For memory validation:
# address = 0x2000
# expected_bytes = "01 02 03 04"

[hints]
level1 = "Vague hint"
level2 = "More specific hint"
level3 = "Almost the solution"
```

### Puzzle Guidelines

**Good puzzles:**
- ✅ Have clear objectives
- ✅ Teach one concept at a time
- ✅ Include 3 levels of hints
- ✅ Are thoroughly tested
- ✅ Have appropriate difficulty ratings
- ✅ Use real x86 instructions

**Avoid:**
- ❌ Unclear or ambiguous goals
- ❌ Requiring external knowledge
- ❌ Frustrating or unfair tricks
- ❌ Unrealistic scenarios
- ❌ Missing or unhelpful hints

### Testing Puzzles

1. **Test manually:**
   - Load your puzzle in the game
   - Solve it using your hints
   - Verify the validation works
   - Check difficulty rating

2. **Test edge cases:**
   - What if someone patches wrong address?
   - What if they use a different solution?
   - Does it handle invalid input?

3. **Get feedback:**
   - Ask someone else to try it
   - Observe where they get stuck
   - Improve hints based on feedback

## 📋 Pull Request Process

1. **Before submitting:**
   - [ ] All tests pass (`cargo test`)
   - [ ] Code is formatted (`cargo fmt`)
   - [ ] No clippy warnings (`cargo clippy`)
   - [ ] Documentation is updated
   - [ ] Commit messages follow convention

2. **PR Description:**
   - Explain what changes you made
   - Why you made them
   - Link related issues
   - Add screenshots if UI changes

3. **Review process:**
   - Maintainers will review your PR
   - Address feedback and requested changes
   - Keep your branch up to date with main
   - Be patient and respectful

4. **After merge:**
   - Your contribution will be in the next release!
   - You'll be added to CONTRIBUTORS.md
   - Thank you for making RevGame better! 🎉

## 🏗️ Project Architecture

### Crate Structure

```
revgame/
├── revgame-core     # Core logic (no dependencies on UI)
│   ├── emulator     # x86 CPU emulation
│   ├── debugger     # Debugger with undo/redo
│   ├── puzzle       # Puzzle loading & validation
│   └── game         # State & achievements
├── revgame-ui       # TUI components (ratatui)
│   ├── screens      # Full screen views
│   ├── widgets      # Reusable components
│   └── tutorial     # Tutorial system
├── revgame-native   # Terminal binary
└── revgame-web      # WASM web version (future)
```

### Key Concepts

**Separation of concerns:**
- Core logic is UI-agnostic
- UI only depends on core, never the reverse
- Puzzles are data files, not code

**Error handling:**
- Use `Result<T, E>` for fallible operations
- Custom error types with `thiserror`
- Propagate errors with `?`

**Testing:**
- Unit tests in same file as code
- Integration tests in `tests/` directory
- Puzzle validation is automated

## 🤔 Questions?

- 💬 Open a [Discussion](https://github.com/yourusername/revgame/discussions)
- 🐛 Report an [Issue](https://github.com/yourusername/revgame/issues)
- 📧 Email: your.email@example.com

## 📜 Code of Conduct

Be respectful, inclusive, and constructive. We want RevGame to be welcoming to everyone:
- Be kind and courteous
- Respect differing opinions
- Focus on what's best for the community
- Show empathy towards others

## 🙏 Thank You!

Every contribution, no matter how small, makes RevGame better. Thank you for taking the time to contribute!

---

**Happy Hacking!** 🔧✨
