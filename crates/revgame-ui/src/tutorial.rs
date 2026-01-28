use ratatui::layout::Rect;

/// Direction for tutorial arrows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Which UI element to highlight
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightTarget {
    Disassembly,
    Registers,
    Memory,
    Stack,
    CommandLine,
    StatusBar,
    CurrentInstruction,
    Flags,
}

/// Condition to advance to next step
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TutorialTrigger {
    /// User presses any key
    AnyKey,
    /// User presses Enter/Space
    Continue,
    /// User executes a step (F10)
    Step,
    /// User runs the program (F5)
    Run,
    /// User sets a breakpoint (F9)
    SetBreakpoint,
    /// User resets the puzzle (F4)
    Reset,
    /// User enters command mode
    EnterCommand,
    /// User types a specific command
    Command(String),
    /// User patches memory
    Patch,
    /// User requests a hint
    Hint,
    /// Program halts
    ProgramHalts,
    /// Puzzle is solved
    PuzzleSolved,
}

/// A single step in the tutorial
#[derive(Debug, Clone)]
pub struct TutorialStep {
    /// Title for this step
    pub title: String,
    /// Explanation text (can be multi-line)
    pub text: String,
    /// Which UI element to highlight (if any)
    pub highlight: Option<HighlightTarget>,
    /// Arrow direction to draw (if any)
    pub arrow: Option<ArrowDirection>,
    /// What triggers moving to the next step
    pub trigger: TutorialTrigger,
    /// Hint text shown at bottom
    pub hint: Option<String>,
}

impl TutorialStep {
    pub fn new(title: &str, text: &str) -> Self {
        Self {
            title: title.to_string(),
            text: text.to_string(),
            highlight: None,
            arrow: None,
            trigger: TutorialTrigger::Continue,
            hint: None,
        }
    }

    pub fn highlight(mut self, target: HighlightTarget) -> Self {
        self.highlight = Some(target);
        self
    }

    pub fn arrow(mut self, direction: ArrowDirection) -> Self {
        self.arrow = Some(direction);
        self
    }

    pub fn trigger(mut self, trigger: TutorialTrigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.to_string());
        self
    }
}

/// Tutorial state machine
#[derive(Debug, Clone)]
pub struct Tutorial {
    /// All steps in the tutorial
    pub steps: Vec<TutorialStep>,
    /// Current step index
    pub current_step: usize,
    /// Whether tutorial is active
    pub active: bool,
    /// Whether tutorial is paused (user doing something)
    pub paused: bool,
}

impl Tutorial {
    /// Create the main interface tutorial
    pub fn interface_tutorial() -> Self {
        let steps = vec![
            TutorialStep::new(
                "Welcome to RevGame!",
                "This tutorial will teach you how to use the debugger\n\
                 interface to solve reverse engineering puzzles.\n\n\
                 Press ENTER or SPACE to continue..."
            )
            .trigger(TutorialTrigger::Continue),

            TutorialStep::new(
                "The Disassembly View",
                "This panel shows the program's machine code\n\
                 translated into assembly language.\n\n\
                 The '►' arrow marks the current instruction.\n\
                 The '●' symbol marks breakpoints.\n\n\
                 Press ENTER to continue..."
            )
            .highlight(HighlightTarget::Disassembly)
            .arrow(ArrowDirection::Right)
            .trigger(TutorialTrigger::Continue),

            TutorialStep::new(
                "Registers",
                "Registers are the CPU's fast storage.\n\n\
                 EAX, EBX, ECX, EDX - General purpose\n\
                 ESP - Stack pointer\n\
                 EIP - Instruction pointer (next instruction)\n\n\
                 Watch how values change as you step!"
            )
            .highlight(HighlightTarget::Registers)
            .arrow(ArrowDirection::Left)
            .trigger(TutorialTrigger::Continue),

            TutorialStep::new(
                "The Memory View",
                "This shows raw bytes in memory.\n\n\
                 Left side: hex values\n\
                 Right side: ASCII representation\n\n\
                 You can patch these bytes to modify the program!"
            )
            .highlight(HighlightTarget::Memory)
            .arrow(ArrowDirection::Right)
            .trigger(TutorialTrigger::Continue),

            TutorialStep::new(
                "The Stack",
                "The stack stores temporary values, return\n\
                 addresses, and function arguments.\n\n\
                 It grows downward (lower addresses).\n\
                 ESP points to the top of the stack."
            )
            .highlight(HighlightTarget::Stack)
            .arrow(ArrowDirection::Left)
            .trigger(TutorialTrigger::Continue),

            TutorialStep::new(
                "Let's Step!",
                "Press F10 to execute ONE instruction.\n\n\
                 Watch the registers change!\n\
                 The '►' arrow will move to the next instruction."
            )
            .highlight(HighlightTarget::CurrentInstruction)
            .arrow(ArrowDirection::Right)
            .trigger(TutorialTrigger::Step)
            .hint("Press F10 to step"),

            TutorialStep::new(
                "Great!",
                "Did you see EIP change? That's the instruction\n\
                 pointer moving to the next instruction.\n\n\
                 Changed registers are highlighted in GREEN.\n\n\
                 Step a few more times to see the program flow..."
            )
            .highlight(HighlightTarget::Registers)
            .trigger(TutorialTrigger::Step)
            .hint("Press F10 again"),

            TutorialStep::new(
                "Understanding the Code",
                "Look at the disassembly:\n\n\
                 CMP compares two values\n\
                 JNE jumps if they're Not Equal\n\n\
                 The program checks if EAX equals 0x1337.\n\
                 But EAX is 0xDEAD - so it will FAIL!\n\n\
                 Press ENTER to continue..."
            )
            .highlight(HighlightTarget::Disassembly)
            .trigger(TutorialTrigger::Continue),

            TutorialStep::new(
                "The Challenge",
                "Your goal: Make the program succeed!\n\n\
                 When it ends, EAX should be 1 (success)\n\
                 not 0 (failure).\n\n\
                 One way: patch the JNE to never jump.\n\
                 The NOP instruction (0x90) does nothing!"
            )
            .trigger(TutorialTrigger::Continue),

            TutorialStep::new(
                "Command Mode",
                "Press ':' to enter command mode.\n\n\
                 Then type:\n\
                 patch 0x1005 90 90\n\n\
                 This replaces JNE with two NOPs!"
            )
            .highlight(HighlightTarget::CommandLine)
            .arrow(ArrowDirection::Up)
            .trigger(TutorialTrigger::EnterCommand)
            .hint("Press ':' to enter command mode"),

            TutorialStep::new(
                "Patch the Code",
                "Type: patch 0x1005 90 90\n\n\
                 Then press ENTER to apply the patch.\n\n\
                 0x1005 is the address of JNE\n\
                 90 90 are two NOP instructions"
            )
            .highlight(HighlightTarget::CommandLine)
            .trigger(TutorialTrigger::Patch)
            .hint("Type: patch 0x1005 90 90"),

            TutorialStep::new(
                "Patched!",
                "Look at the disassembly - the JNE is gone!\n\n\
                 Now let's reset and run the patched program.\n\
                 Press F4 to reset to the start."
            )
            .highlight(HighlightTarget::Disassembly)
            .trigger(TutorialTrigger::Reset)
            .hint("Press F4 to reset"),

            TutorialStep::new(
                "Run the Program",
                "Now press F5 to run until the program halts.\n\n\
                 The patched code should take the success path!"
            )
            .trigger(TutorialTrigger::Run)
            .hint("Press F5 to run"),

            TutorialStep::new(
                "Tutorial Complete!",
                "If EAX is 1, you solved it!\n\n\
                 You've learned:\n\
                 • Reading disassembly\n\
                 • Understanding registers\n\
                 • Stepping through code\n\
                 • Patching instructions\n\n\
                 Press ENTER to finish the tutorial."
            )
            .trigger(TutorialTrigger::Continue),
        ];

        Self {
            steps,
            current_step: 0,
            active: true,
            paused: false,
        }
    }

    /// Get the current step
    pub fn current(&self) -> Option<&TutorialStep> {
        if self.active {
            self.steps.get(self.current_step)
        } else {
            None
        }
    }

    /// Check if a trigger advances the tutorial
    pub fn check_trigger(&mut self, trigger: &TutorialTrigger) -> bool {
        if !self.active || self.paused {
            return false;
        }

        if let Some(step) = self.steps.get(self.current_step) {
            let matches = match (&step.trigger, trigger) {
                (TutorialTrigger::AnyKey, _) => true,
                (TutorialTrigger::Continue, TutorialTrigger::Continue) => true,
                (TutorialTrigger::Step, TutorialTrigger::Step) => true,
                (TutorialTrigger::Run, TutorialTrigger::Run) => true,
                (TutorialTrigger::SetBreakpoint, TutorialTrigger::SetBreakpoint) => true,
                (TutorialTrigger::Reset, TutorialTrigger::Reset) => true,
                (TutorialTrigger::EnterCommand, TutorialTrigger::EnterCommand) => true,
                (TutorialTrigger::Patch, TutorialTrigger::Patch) => true,
                (TutorialTrigger::Hint, TutorialTrigger::Hint) => true,
                (TutorialTrigger::ProgramHalts, TutorialTrigger::ProgramHalts) => true,
                (TutorialTrigger::PuzzleSolved, TutorialTrigger::PuzzleSolved) => true,
                (TutorialTrigger::Command(expected), TutorialTrigger::Command(actual)) => {
                    expected == actual
                }
                _ => false,
            };

            if matches {
                self.advance();
                return true;
            }
        }

        false
    }

    /// Advance to the next step
    pub fn advance(&mut self) {
        if self.current_step < self.steps.len() - 1 {
            self.current_step += 1;
        } else {
            self.active = false;
        }
    }

    /// Skip the tutorial
    pub fn skip(&mut self) {
        self.active = false;
    }

    /// Pause tutorial (while user does something)
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume tutorial
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Check if tutorial is finished
    pub fn is_finished(&self) -> bool {
        !self.active || self.current_step >= self.steps.len()
    }

    /// Get progress as (current, total)
    pub fn progress(&self) -> (usize, usize) {
        (self.current_step + 1, self.steps.len())
    }
}
