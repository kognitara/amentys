#![cfg_attr(not(test), no_std)]
use core::cmp;

/// The visual detector that projects the 1D data onto a 2D grid
///
/// # Fields
/// - `buffer` The raw data slice (up to 4096 bytes)
/// - `cursor_x` Visual column (intention)
/// - `cursor_y` Visual line on the screen
/// - `scroll_y` Vertical scroll (how many lines we've passed)
pub struct Section<'a> {
    pub buffer: &'a [u8],
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub scroll_y: u16,
}

impl Section<'_> {
    /// Creates a new Section with the given buffer.
    ///
    /// # Arg
    /// - `buffer`: The raw data slice (up to 4096 bytes)
    #[must_use]
    pub const fn new(buffer: &[u8]) -> Section<'_> {
        Section {
            buffer,
            cursor_x: 0,
            cursor_y: 0,
            scroll_y: 0,
        }
    }

    /// Calculates the length of the line at the given target line.
    /// # Arg
    /// - `target_line`: The target line number
    /// # Return
    /// - The length of the line at the given target line
    #[must_use]
    pub fn line_length(&self, target_line: usize) -> usize {
        let mut current_line = 0;
        let mut length = 0;

        for &byte in self.buffer {
            if current_line == target_line {
                if byte == b'\n' {
                    break;
                }
                length += 1;
            } else if byte == b'\n' {
                current_line += 1;
            }
        }
        length
    }
    /// Move cursor
    /// # Args
    /// - `dx`: The horizontal movement
    /// - `dy`: The vertical movement
    pub fn move_cursor(&mut self, dx: i16, dy: i16) {
        let absolute_y = i32::from(self.scroll_y) + i32::from(self.cursor_y) + i32::from(dy);

        if absolute_y < 0 {
            self.cursor_y = 0;
            self.scroll_y = 0;
        } else {
            let new_y = u16::try_from(absolute_y).unwrap_or(0);
            if new_y < self.scroll_y {
                self.scroll_y = new_y;
                self.cursor_y = 0;
            } else if new_y >= self.scroll_y + 40 {
                self.scroll_y = new_y - 40 + 1;
                self.cursor_y = 40 - 1;
            } else {
                self.cursor_y = new_y - self.scroll_y;
            }
        }
        let target_line = usize::from(self.scroll_y + self.cursor_y);
        let max_x_usize = self.line_length(target_line);
        let max_x = u16::try_from(max_x_usize).unwrap_or(u16::MAX);

        let absolute_x = i32::from(self.cursor_x) + i32::from(dx);
        let safe_x_i32 = cmp::max(absolute_x, 0);
        let safe_x = u16::try_from(safe_x_i32).unwrap_or(0);

        self.cursor_x = cmp::min(safe_x, max_x);
    }

    /// Convert cursor position to byte offset
    #[must_use]
    pub fn cursor_to_byte_offset(&self) -> usize {
        let target_line = usize::from(self.scroll_y + self.cursor_y);
        let target_col = usize::from(self.cursor_x);

        let mut current_line = 0;
        let mut byte_offset = 0;

        for (i, &byte) in self.buffer.iter().enumerate() {
            if current_line == target_line {
                let remaining = self.buffer[i..].iter().take_while(|&&b| b != b'\n').count();
                let actual_col = cmp::min(target_col, remaining);
                return i + actual_col;
            }
            if byte == b'\n' {
                current_line += 1;
            }
            byte_offset += 1;
        }
        byte_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_line_length() {
        let text = b"Amentys\nCubeFS\nZen";
        let section = Section::new(text);

        assert_eq!(section.line_length(0), 7); // "Amentys"
        assert_eq!(section.line_length(1), 6); // "CubeFS"
        assert_eq!(section.line_length(2), 3); // "Zen"
        assert_eq!(section.line_length(99), 0);
    }

    #[test]
    fn test_cursor_to_byte_offset() {
        let text = b"Amentys\nCubeFS\nZen";
        let mut section = Section::new(text);

        // Position (0, 0) -> Pointe sur 'A'
        assert_eq!(section.cursor_to_byte_offset(), 0);

        // Mouvement vers la ligne 1, colonne 0 -> Pointe sur 'C'
        section.cursor_y = 1;
        assert_eq!(section.cursor_to_byte_offset(), 8); // 7 + 1 (pour le \n)

        // Mouvement vers la ligne 1, colonne 4 -> Pointe sur 'F' dans CubeFS
        section.cursor_x = 4;
        assert_eq!(section.cursor_to_byte_offset(), 12);
    }
    #[test]
    fn test_move_cursor_clamping() {
        let text = b"Amentys\nCubeFS\nZen";
        let mut section = Section::new(text);

        // Intention : aller très loin à droite sur la première ligne
        section.move_cursor(50, 0);
        // Le curseur doit être bridé à la fin du mot "Amentys" (longueur 7)
        assert_eq!(section.cursor_x, 7);

        // Intention : descendre sur la ligne "Zen" (qui est plus courte)
        section.move_cursor(0, 2);
        // Le curseur X était à 7, il doit glisser à 3 (la fin de "Zen")
        // Note: l'implémentation de `move_cursor` doit garantir ce glissement (bridage sur max_x)
        assert_eq!(section.cursor_y, 2);
        assert_eq!(section.cursor_x, 3);
    }
}
