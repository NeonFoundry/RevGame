/// VHS-style rewind effect overlay
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

/// State of the rewind animation
#[derive(Debug, Clone)]
pub struct RewindEffect {
    /// Frame counter for animation
    frame: u32,
    /// Duration of effect in frames
    max_frames: u32,
    /// Whether effect is active
    active: bool,
}

impl RewindEffect {
    pub fn new() -> Self {
        Self {
            frame: 0,
            max_frames: 15,
            active: false,
        }
    }

    /// Start the rewind effect
    pub fn trigger(&mut self) {
        self.frame = 0;
        self.active = true;
    }

    /// Advance the animation by one frame
    pub fn tick(&mut self) {
        if self.active {
            self.frame += 1;
            if self.frame >= self.max_frames {
                self.active = false;
                self.frame = 0;
            }
        }
    }

    /// Check if effect is currently active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get current progress (0.0 to 1.0)
    fn progress(&self) -> f32 {
        if !self.active {
            return 0.0;
        }
        self.frame as f32 / self.max_frames as f32
    }
}

impl Default for RewindEffect {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget that renders the VHS rewind effect
pub struct RewindOverlay<'a> {
    effect: &'a RewindEffect,
}

impl<'a> RewindOverlay<'a> {
    pub fn new(effect: &'a RewindEffect) -> Self {
        Self { effect }
    }

    /// Calculate scanline pattern
    fn scanline_char(&self, y: u16) -> char {
        // Alternating scanline pattern
        if y % 2 == 0 {
            '▀'
        } else {
            '▄'
        }
    }

    /// Calculate color based on rewind progress
    fn rewind_color(&self, progress: f32) -> Color {
        // Start bright blue, fade to dark
        let intensity = ((1.0 - progress) * 200.0) as u8;
        Color::Rgb(
            intensity / 4,
            intensity / 3,
            intensity.saturating_add(55),
        )
    }

    /// Calculate noise/tracking lines
    fn should_draw_tracking_line(&self, y: u16, progress: f32) -> bool {
        // Moving horizontal tracking lines
        let line_pos = ((progress * 100.0) as u16 * 3) % 100;
        let dist = if y > line_pos {
            y - line_pos
        } else {
            line_pos - y
        };
        dist < 2
    }
}

impl<'a> Widget for RewindOverlay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.effect.is_active() {
            return;
        }

        let progress = self.effect.progress();

        // Calculate intensity (fades in then out)
        let intensity = if progress < 0.5 {
            progress * 2.0 // Fade in
        } else {
            (1.0 - progress) * 2.0 // Fade out
        };

        let base_color = self.rewind_color(progress);

        // Draw effect across entire area
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if x >= buf.area().width || y >= buf.area().height {
                    continue;
                }

                let cell = &mut buf[(x, y)];
                let frame_u16 = self.effect.frame as u16;

                // Apply scanline effect
                if (y + frame_u16 / 2) % 3 == 0 {
                    let scanline_intensity = (intensity * 128.0) as u8;
                    let scanline_color = Color::Rgb(
                        scanline_intensity / 4,
                        scanline_intensity / 3,
                        scanline_intensity,
                    );
                    cell.set_fg(scanline_color);
                }

                // Add tracking lines
                if self.should_draw_tracking_line(y, progress) {
                    cell.set_char('─');
                    cell.set_style(
                        Style::default()
                            .fg(base_color)
                            .bg(Color::Rgb(10, 10, 30)),
                    );
                }

                // Add noise/static in certain areas
                if (x + y + frame_u16) % 7 == 0 && intensity > 0.3 {
                    let chars = ['░', '▒', '▓', '█'];
                    let char_idx = ((x as u32 * y as u32 + self.effect.frame) % chars.len() as u32) as usize;
                    cell.set_char(chars[char_idx]);
                    let noise_intensity = (intensity * 100.0) as u8;
                    cell.set_fg(Color::Rgb(
                        noise_intensity,
                        noise_intensity,
                        noise_intensity + 50,
                    ));
                }

                // Horizontal smearing effect (VHS tracking)
                if (y + frame_u16) % 5 == 0 && (x as f32 / area.width as f32) < progress {
                    cell.set_char('▌');
                    cell.set_fg(base_color);
                }
            }
        }

        // Draw "<<< REWIND" text in center
        if intensity > 0.5 {
            let text = "◄◄◄ REWIND";
            let text_x = area.x + (area.width.saturating_sub(text.len() as u16)) / 2;
            let text_y = area.y + area.height / 2;

            if text_y < buf.area().height {
                for (i, ch) in text.chars().enumerate() {
                    let x = text_x + i as u16;
                    if x < buf.area().width {
                        let cell = &mut buf[(x, text_y)];
                        cell.set_char(ch);
                        cell.set_style(
                            Style::default()
                                .fg(Color::Rgb(166, 227, 161))
                                .bg(Color::Rgb(30, 30, 46)),
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewind_effect() {
        let mut effect = RewindEffect::new();
        assert!(!effect.is_active());

        effect.trigger();
        assert!(effect.is_active());
        assert_eq!(effect.frame, 0);

        // Advance to completion
        for _ in 0..effect.max_frames {
            effect.tick();
        }

        assert!(!effect.is_active());
    }

    #[test]
    fn test_progress() {
        let mut effect = RewindEffect::new();
        effect.trigger();

        assert_eq!(effect.progress(), 0.0);

        effect.frame = effect.max_frames / 2;
        assert!((effect.progress() - 0.5).abs() < 0.01);

        effect.frame = effect.max_frames - 1;
        assert!(effect.progress() > 0.9);
    }
}
