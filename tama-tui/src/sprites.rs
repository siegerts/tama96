use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock, time::Duration};
use tama_core::state::{Character, LifeStage, PetState};

/// Width and height of each sprite in pixels.
pub const SPRITE_W: usize = 16;
pub const SPRITE_H: usize = 16;

/// A sprite is a 16-row array of 16-bit bitmasks (1 bit per pixel, MSB = leftmost).
type Sprite = [u16; SPRITE_H];

// ── Braille encoding ────────────────────────────────────────────────────────
// Unicode braille block U+2800–U+28FF. Each braille cell is 2 columns x 4 rows.
// Bit mapping within a cell (col, row) -> bit:
//   (0,0)=0x01  (1,0)=0x08
//   (0,1)=0x02  (1,1)=0x10
//   (0,2)=0x04  (1,2)=0x20
//   (0,3)=0x40  (1,3)=0x80

/// Convert a 16x16 pixel sprite into a multi-line string of braille characters.
/// Result is 4 lines of 8 braille chars each (16/2 cols, 16/4 rows).
pub fn sprite_to_braille(sprite: &Sprite) -> String {
    let mut out = String::new();
    for cell_row in 0..4 {
        for cell_col in 0..8 {
            let mut code: u8 = 0;
            let px = cell_col * 2;
            let py = cell_row * 4;
            if pixel(sprite, px, py)     { code |= 0x01; }
            if pixel(sprite, px, py + 1) { code |= 0x02; }
            if pixel(sprite, px, py + 2) { code |= 0x04; }
            if pixel(sprite, px + 1, py)     { code |= 0x08; }
            if pixel(sprite, px + 1, py + 1) { code |= 0x10; }
            if pixel(sprite, px + 1, py + 2) { code |= 0x20; }
            if pixel(sprite, px, py + 3) { code |= 0x40; }
            if pixel(sprite, px + 1, py + 3) { code |= 0x80; }
            out.push(char::from_u32(0x2800 + code as u32).unwrap());
        }
        if cell_row < 3 {
            out.push('\n');
        }
    }
    out
}

/// Read a single pixel from a sprite bitmap.
#[inline]
fn pixel(sprite: &Sprite, x: usize, y: usize) -> bool {
    if x >= SPRITE_W || y >= SPRITE_H {
        return false;
    }
    (sprite[y] >> (SPRITE_W - 1 - x)) & 1 == 1
}

#[derive(Deserialize)]
#[serde(try_from = "[String; SPRITE_H]")]
struct Bitmap(Sprite);

impl TryFrom<[String; SPRITE_H]> for Bitmap {
    type Error = &'static str;

    fn try_from(rows: [String; SPRITE_H]) -> Result<Self, Self::Error> {
        let mut bits = [0; SPRITE_H];
        for (row, cells) in bits.iter_mut().zip(rows) {
            if cells.len() != SPRITE_W {
                return Err("expected a 16-pixel row");
            }
            for pixel in cells.bytes() {
                *row = (*row << 1)
                    | match pixel {
                        b'.' => 0,
                        b'#' => 1,
                        _ => return Err("unknown sprite pixel"),
                    };
            }
        }
        Ok(Self(bits))
    }
}

#[derive(Deserialize)]
struct Poses {
    rest: Bitmap,
    gesture: Option<Bitmap>,
    sleeping: Option<Bitmap>,
    sick: Option<Bitmap>,
}

static ART: LazyLock<HashMap<String, Poses>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/pet-frames.json"))
        .expect("invalid built-in pet artwork")
});

pub struct PetFrame {
    bitmap: &'static Bitmap,
    padding: usize,
    pub redraw_after: Duration,
}

impl PetFrame {
    fn still(bitmap: &'static Bitmap) -> Self {
        Self {
            bitmap,
            padding: 0,
            redraw_after: Duration::from_secs(1),
        }
    }

    pub fn render(&self, large: bool, right: bool) -> String {
        let pixels = if right { self.bitmap.0.map(u16::reverse_bits) } else { self.bitmap.0 };
        if large { self.blocks(&pixels) } else { self.braille_pixels(&pixels) }
    }

    /// One terminal column per pixel, with two pixel rows in each character.
    fn blocks(&self, pixels: &Sprite) -> String {
        (0..SPRITE_H).step_by(2).map(|y| {
            let mut row = " ".repeat(self.padding * 2);
            for x in 0..SPRITE_W {
                row.push(match (pixel(pixels, x, y), pixel(pixels, x, y + 1)) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                });
            }
            row
        }).collect::<Vec<_>>().join("\n")
    }

    fn braille_pixels(&self, pixels: &Sprite) -> String {
        let sprite = sprite_to_braille(pixels);
        if self.padding == 0 {
            return sprite;
        }
        let spaces = " ".repeat(self.padding);
        sprite
            .lines()
            .map(|line| format!("{spaces}{line}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn pet_frame(state: &PetState, elapsed: Duration, bite_left: Duration) -> PetFrame {
    if !state.is_alive {
        return PetFrame::still(&ART["dead"].rest);
    }
    let egg = state.stage == LifeStage::Egg;
    let poses = &ART[if egg {
        "Egg"
    } else {
        crate::character_name(state)
    }];
    if state.is_sleeping || state.is_sick {
        let bitmap = if egg {
            &poses.rest
        } else if state.is_sleeping {
            poses.sleeping.as_ref().unwrap_or(&ART["sleeping"].rest)
        } else {
            poses.sick.as_ref().unwrap_or(&ART["sick"].rest)
        };
        return PetFrame::still(bitmap);
    }

    let ms = elapsed.as_millis() as u64;
    if let Some(gesture) = &poses.gesture {
        if state.character == Character::Kuchipatchi {
            if bite_left.is_zero() {
                return PetFrame::still(&poses.rest);
            }
            return PetFrame {
                bitmap: gesture,
                padding: 0,
                redraw_after: bite_left,
            };
        }
        if state.character == Character::Marutchi && state.happiness < 3 {
            return PetFrame::still(&poses.rest);
        }
        let beat = ms % 1500;
        let (bitmap, next) = if beat < 1200 {
            (&poses.rest, 1200)
        } else {
            (gesture, 1500)
        };
        return PetFrame {
            bitmap,
            padding: 0,
            redraw_after: Duration::from_millis(next - beat),
        };
    }

    PetFrame {
        bitmap: &poses.rest,
        padding: [0, 1, 0, 2][(elapsed.as_secs() % 4) as usize],
        redraw_after: Duration::from_millis(1000 - ms % 1000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tama_core::state::PetState;

    #[test]
    fn braille_output_dimensions() {
        let s = sprite_to_braille(&ART["Egg"].rest.0);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 4, "expected 4 lines of braille");
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                8,
                "line {i} should have 8 braille chars, got {}",
                line.chars().count()
            );
        }
    }

    #[test]
    fn blank_sprite_is_all_empty_braille() {
        let blank: Sprite = [0u16; 16];
        let s = sprite_to_braille(&blank);
        for ch in s.chars() {
            if ch != '\n' {
                assert_eq!(ch, '\u{2800}', "blank sprite should produce empty braille");
            }
        }
    }

    #[test]
    fn get_sprite_returns_egg_for_new_egg() {
        let state = PetState::new_egg(Utc::now());
        let s = pet_frame(&state, Duration::ZERO, Duration::ZERO).render(false, false);
        assert!(!s.is_empty());
        let expected = sprite_to_braille(&ART["Egg"].rest.0);
        assert_eq!(s, expected);
    }

    #[test]
    fn get_sprite_returns_dead_for_dead_pet() {
        let mut state = PetState::new_egg(Utc::now());
        state.is_alive = false;
        state.stage = LifeStage::Dead;
        let s = pet_frame(&state, Duration::ZERO, Duration::ZERO).render(false, false);
        let expected = sprite_to_braille(&ART["dead"].rest.0);
        assert_eq!(s, expected);
    }

    #[test]
    fn all_characters_have_sprites() {
        let chars = [
            Character::Babytchi,
            Character::Marutchi,
            Character::Tamatchi,
            Character::Kuchitamatchi,
            Character::Mametchi,
            Character::Ginjirotchi,
            Character::Maskutchi,
            Character::Kuchipatchi,
            Character::Nyorotchi,
            Character::Tarakotchi,
            Character::Oyajitchi,
        ];
        let mut state = PetState::new_egg(Utc::now());
        state.stage = LifeStage::Adult;
        for ch in chars {
            state.character = ch;
            let frame = pet_frame(&state, Duration::ZERO, Duration::ZERO);
            assert!(
                frame.bitmap.0.iter().any(|&row| row != 0),
                "blank sprite for {:?}",
                state.character
            );
        }
    }

    #[test]
    fn gestures_follow_mood_and_feeding_and_yield_to_status() {
        let mut state = PetState::new_egg(Utc::now());
        state.stage = LifeStage::Adult;
        let pulse = Duration::from_millis(1200);
        let bite = Duration::from_millis(450);
        let zero = Duration::ZERO;
        for character in [
            Character::Babytchi,
            Character::Marutchi,
            Character::Tamatchi,
            Character::Kuchipatchi,
        ] {
            state.character = character;
            state.happiness = 4;
            state.is_sleeping = false;
            state.is_sick = false;
            let poses = &ART[crate::character_name(&state)];
            let frame = |s: &PetState, elapsed, feeding| pet_frame(s, elapsed, feeding).bitmap.0;
            assert_eq!(frame(&state, zero, zero), poses.rest.0);
            if state.character == Character::Kuchipatchi {
                assert_eq!(frame(&state, pulse, zero), poses.rest.0);
                assert_eq!(frame(&state, zero, bite), poses.gesture.as_ref().unwrap().0);
            } else {
                assert_eq!(
                    frame(&state, pulse, zero),
                    poses.gesture.as_ref().unwrap().0
                );
                assert_eq!(
                    frame(&state, Duration::from_millis(1500), zero),
                    poses.rest.0
                );
            }
            if state.character == Character::Marutchi {
                state.happiness = 2;
                assert_eq!(frame(&state, pulse, zero), poses.rest.0);
                state.happiness = 3;
                assert_eq!(
                    frame(&state, pulse, zero),
                    poses.gesture.as_ref().unwrap().0
                );
            }
            state.is_sick = true;
            assert_eq!(frame(&state, pulse, bite), poses.sick.as_ref().unwrap().0);
            state.is_sleeping = true;
            assert_eq!(
                frame(&state, pulse, bite),
                poses.sleeping.as_ref().unwrap().0
            );
        }
    }
}
