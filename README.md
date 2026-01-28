# 🔧 RevGame

**An interactive reverse engineering learning game with real x86 emulation**

RevGame is a terminal-based (TUI) educational game where you learn reverse engineering by solving progressively challenging puzzles. Unlike other "hacking games" with fake instructions, RevGame uses **real x86/x86-64 opcodes** and a **real CPU emulator** to give you authentic reverse engineering experience.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        🎮 REVGAME v0.1.0 🎮                          │
│                                                                       │
│              Master Reverse Engineering Through Play                  │
└─────────────────────────────────────────────────────────────────────┘

  [1] Start Tutorial (with walkthrough)
  [2] Quick Start (skip tutorial)
  [3] Puzzle Select (coming soon)
  [A] Achievements 🏆
  [Q] Quit
```

## ✨ Features

### 🎯 Real x86 Emulation
- **Authentic x86/x86-64 instruction set** - No fake opcodes!
- 30+ implemented instructions: MOV, ADD, SUB, JMP, CALL, RET, and more
- Real CPU flags (ZF, SF, CF, OF, etc.)
- Stack operations (PUSH, POP, CALL/RET)
- Proper memory management with segmentation

### 🎓 Progressive Learning
- **Interactive tutorial** with visual arrows guiding you through the interface
- 11+ puzzles ranging from beginner to intermediate
- Hint system with 3 levels per puzzle
- Prerequisites system - unlock harder puzzles as you progress
- Category-based progression (Basics → Control Flow → Crackmes)

### 🏆 Achievement System
- **15 unique achievements** with points and icons
- Track statistics: patches made, undos used, win streaks
- Special achievements:
  - 🎯 **One Shot, One Kill** - Solve with a single patch (100 pts)
  - 🧠 **Self Taught** - No hints used (25 pts)
  - ⚡ **Speed Runner** - Beat par time (50 pts)
  - 🔥 **Win Streaks** - Solve puzzles consecutively (50-500 pts)
  - 💎 **Perfectionist** - Unlock all achievements in a category (1000 pts)

### ⏪ Time-Travel Debugging
- **Undo/Redo system** for memory patches (up to 100 actions)
- **VHS rewind effect** - Watch a retro visual effect when you undo!
- Never lose progress - experiment freely and rewind mistakes
- Track all patch history with full state restoration

### 🎨 Beautiful TUI Interface
- **Catppuccin-inspired color scheme**
- Real-time disassembly view with syntax highlighting
- Register panel with change tracking
- Memory hex viewer with ASCII display
- Stack visualization
- Breakpoint support
- Command-line interface for power users

### 🔍 Full-Featured Debugger
- Step through instructions one at a time
- Run until breakpoint or halt
- Set/remove breakpoints at any address
- View memory, registers, and stack in real-time
- Patch memory with hex values
- Reset to initial state anytime

## 🚀 Installation

### Prerequisites
- Rust 1.70+ (get it at [rustup.rs](https://rustup.rs))

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/revgame.git
cd revgame

# Build release version
cargo build --release

# Run the game
cargo run --release -p revgame-native
```

The binary will be at `target/release/revgame`.

## 🎮 Quick Start

1. **Launch the game:**
   ```bash
   cargo run --release -p revgame-native
   ```

2. **Start the tutorial** (recommended for first-time players):
   - Press `1` from the main menu
   - Follow the visual walkthrough with arrows pointing to each UI element

3. **Solve your first puzzle:**
   - Use `F10` or `s` to step through instructions
   - Use `F5` or `r` to run until halt
   - Use `:` to open command line
   - Type `patch <address> <bytes>` to modify memory
   - Example: `patch 0x1000 90` (NOP out an instruction)

4. **Check your achievements:**
   - Press `A` from the main menu
   - See your progress, points, and unlocked achievements

## 📚 Controls

### Navigation
- `Tab` - Cycle through panels
- `Shift+Tab` - Cycle backwards
- `Esc` - Return to menu (when not in tutorial)

### Debugging
- `F5` or `r` - Run until breakpoint/halt
- `F10` or `s` - Step one instruction
- `F9` or `b` - Toggle breakpoint at cursor
- `F4` - Reset puzzle to initial state
- `:` or `/` - Open command line

### Editing
- `u` - Undo last patch
- `Ctrl+Y` or `Ctrl+Shift+Z` - Redo

### Help
- `F1` or `?` - Show help
- `h` - Show hint (costs points for achievements)

## 🎯 Puzzle Categories

### 01-basics (5 puzzles)
Learn the fundamentals:
- Memory patching with NOPs
- Changing immediate values
- Jump reversal (JE ↔ JNE)
- Conditional to unconditional jumps
- Multi-byte patches

### 02-control-flow (5 puzzles)
Master control flow:
- Bypassing multiple checks
- Loop analysis and modification
- Stack operations (PUSH/POP)
- CPU flags (TEST, ZF, SF, etc.)
- Arithmetic operations

### 03-crackmes (1 puzzle)
Real crackme challenges:
- Serial validation
- XOR encryption
- String comparison
- Key generation

## 🛠️ Creating Custom Puzzles

Puzzles are defined in TOML format:

```toml
[metadata]
id = "basic-001"
title = "Your First Patch"
difficulty = 1
category = "patching"
tags = ["nop", "beginner"]
estimated_time_minutes = 3

[description]
brief = "Learn to NOP out an instruction"
detailed = """
This program checks if EAX equals 0x42.
Try NOPing out the comparison to always succeed!
"""

[setup]
memory_size = 16384
code_start = 0x1000
data_start = 0x2000
stack_start = 0x3000

[setup.code]
# Assembly: cmp eax, 0x42; je success; fail...
bytes = "83 F8 42 74 05 B8 00 00 00 00 EB 05 B8 01 00 00 00 F4"
entry_point = 0

[validation]
type = "register_value"
register = "eax"
expected = 1

[hints]
level1 = "Look at the CMP instruction..."
level2 = "The NOP instruction is 0x90"
level3 = "Try: patch 0x1000 90 90 90"
```

Save puzzles in `puzzles/<category>/<number>-<name>.toml`.

## 🏗️ Project Structure

```
RevGame/
├── crates/
│   ├── revgame-core/      # Core emulation & game logic
│   │   ├── emulator/      # x86 CPU emulator
│   │   ├── debugger/      # Debugger with undo/redo
│   │   ├── puzzle/        # Puzzle loader & validator
│   │   └── game/          # Achievement system
│   ├── revgame-ui/        # TUI components
│   │   ├── screens/       # UI screens
│   │   ├── widgets/       # Reusable widgets
│   │   └── tutorial/      # Tutorial system
│   ├── revgame-native/    # Native terminal app
│   └── revgame-web/       # WASM web version (planned)
└── puzzles/               # Puzzle definitions
    ├── 01-basics/
    ├── 02-control-flow/
    └── 03-crackmes/
```

## 🎨 Architecture

### Core Components

**Emulator** (`revgame-core/emulator/`):
- Pure Rust x86 CPU emulator
- Supports 32-bit x86 instruction subset
- Memory management with regions and permissions
- Uses [iced-x86](https://github.com/icedland/iced) for disassembly

**Debugger** (`revgame-core/debugger/`):
- Step/run execution control
- Breakpoint system
- Undo/redo with full state history
- Memory patching with validation

**Achievement System** (`revgame-core/game/`):
- 15 achievements tracking various skills
- Statistics: patches, undos, streaks, time
- Category completion detection
- Points and progress tracking

**TUI** (`revgame-ui/`):
- Built with [ratatui](https://github.com/ratatui-org/ratatui)
- Multiple views: disassembly, registers, memory, stack
- Tutorial overlay with visual arrows
- VHS rewind effect for undo operations

## 🧪 Testing

Run all tests:
```bash
cargo test
```

Run specific test suite:
```bash
cargo test -p revgame-core
cargo test -p revgame-ui
```

All 28 core emulator tests pass! ✅

## 🗺️ Roadmap

### Near-term (v0.2)
- [ ] 20+ more puzzles (intermediate & advanced)
- [ ] Built-in x86 instruction reference (press `?` on any instruction)
- [ ] Search functionality (find bytes, strings, instructions)
- [ ] Save/load puzzle progress
- [ ] Syntax highlighting for instruction types

### Mid-term (v0.3)
- [ ] Bookmarks & notes system
- [ ] Cross-references (show jump destinations)
- [ ] Control flow visualization
- [ ] More achievements & leaderboards
- [ ] Puzzle editor GUI

### Long-term (v1.0)
- [ ] WASM web version
- [ ] 64-bit x86-64 support
- [ ] ARM architecture puzzles
- [ ] Multiplayer challenges
- [ ] Custom puzzle sharing

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. **Report bugs** - Open an issue with reproduction steps
2. **Suggest features** - Share your ideas in discussions
3. **Create puzzles** - Submit new puzzle TOML files
4. **Improve docs** - Help make the game more accessible
5. **Add features** - Check the roadmap and submit PRs

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch

# Run with auto-reload
cargo watch -x 'run -p revgame-native'

# Run tests on file change
cargo watch -x test
```

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [iced-x86](https://github.com/icedland/iced) - Blazingly fast x86 decoder/disassembler
- [ratatui](https://github.com/ratatui-org/ratatui) - Amazing terminal UI library
- [Catppuccin](https://github.com/catppuccin/catppuccin) - Soothing pastel color theme
- Inspired by [Microcorruption](https://microcorruption.com/), [pwn.college](https://pwn.college/), and crackmes.one

## 📞 Contact

- GitHub: [@yourusername](https://github.com/yourusername)
- Issues: [GitHub Issues](https://github.com/yourusername/revgame/issues)

---

**Made with ❤️ and Rust** • Learn by doing • Master reverse engineering through play

⭐ Star this repo if you find it useful!
