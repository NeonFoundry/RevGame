use ratatui::style::{Color, Modifier, Style};

/// Color theme for the TUI
#[derive(Debug, Clone)]
pub struct Theme {
    /// Background color
    pub bg: Color,
    /// Primary foreground
    pub fg: Color,
    /// Accent color (highlights)
    pub accent: Color,
    /// Success color
    pub success: Color,
    /// Error/danger color
    pub error: Color,
    /// Warning color
    pub warning: Color,
    /// Muted/secondary text
    pub muted: Color,
    /// Border color
    pub border: Color,
    /// Selection/highlight background
    pub selection_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Dark theme (default)
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 46),        // Catppuccin base
            fg: Color::Rgb(205, 214, 244),     // Catppuccin text
            accent: Color::Rgb(137, 180, 250), // Catppuccin blue
            success: Color::Rgb(166, 227, 161), // Catppuccin green
            error: Color::Rgb(243, 139, 168),  // Catppuccin red
            warning: Color::Rgb(249, 226, 175), // Catppuccin yellow
            muted: Color::Rgb(127, 132, 156),  // Catppuccin overlay0
            border: Color::Rgb(88, 91, 112),   // Catppuccin surface2
            selection_bg: Color::Rgb(69, 71, 90), // Catppuccin surface1
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            bg: Color::Rgb(239, 241, 245),
            fg: Color::Rgb(76, 79, 105),
            accent: Color::Rgb(30, 102, 245),
            success: Color::Rgb(64, 160, 43),
            error: Color::Rgb(210, 15, 57),
            warning: Color::Rgb(223, 142, 29),
            muted: Color::Rgb(140, 143, 161),
            border: Color::Rgb(172, 176, 190),
            selection_bg: Color::Rgb(204, 208, 218),
        }
    }

    /// Get style for normal text
    pub fn normal(&self) -> Style {
        Style::default().fg(self.fg)
    }

    /// Get style for muted text
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Get style for highlighted/accent text
    pub fn highlight(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Get style for success text
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Get style for error text
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Get style for warning text
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Get style for selected item
    pub fn selected(&self) -> Style {
        Style::default()
            .fg(self.fg)
            .bg(self.selection_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for borders
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Get style for focused borders
    pub fn border_focused(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Get style for the current instruction
    pub fn current_instruction(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for breakpoints
    pub fn breakpoint(&self) -> Style {
        Style::default()
            .fg(self.error)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for changed values
    pub fn changed(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// Get style for register names
    pub fn register_name(&self) -> Style {
        Style::default()
            .fg(self.accent)
    }

    /// Get style for register values
    pub fn register_value(&self) -> Style {
        Style::default()
            .fg(self.fg)
    }

    /// Get style for memory addresses
    pub fn address(&self) -> Style {
        Style::default()
            .fg(self.muted)
    }

    /// Get style for hex bytes
    pub fn hex_byte(&self) -> Style {
        Style::default()
            .fg(self.fg)
    }

    /// Get style for ASCII representation
    pub fn ascii(&self) -> Style {
        Style::default()
            .fg(self.muted)
    }
}
