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

// ── Public API ──────────────────────────────────────────────────────────────

/// Return the braille string for the current pet state.
pub fn get_sprite(state: &PetState) -> String {
    let sprite = select_sprite(state);
    sprite_to_braille(sprite)
}

fn select_sprite(state: &PetState) -> &'static Sprite {
    if !state.is_alive { return &DEAD; }
    if state.stage == LifeStage::Egg { return &EGG; }
    if state.is_sleeping { return &SLEEPING; }
    if state.is_sick { return &SICK; }
    idle_sprite(&state.character)
}

fn idle_sprite(ch: &Character) -> &'static Sprite {
    match ch {
        Character::Babytchi      => &BABYTCHI,
        Character::Marutchi      => &MARUTCHI,
        Character::Tamatchi      => &TAMATCHI,
        Character::Kuchitamatchi => &KUCHITAMATCHI,
        Character::Mametchi      => &MAMETCHI,
        Character::Ginjirotchi   => &GINJIROTCHI,
        Character::Maskutchi     => &MASKUTCHI,
        Character::Kuchipatchi   => &KUCHIPATCHI,
        Character::Nyorotchi     => &NYOROTCHI,
        Character::Tarakotchi    => &TARAKOTCHI,
        Character::Oyajitchi     => &OYAJITCHI,
    }
}

// Egg — oval with diagonal stripe pattern (like the real P1 egg)
#[rustfmt::skip]
const EGG: Sprite = [
    0b0000001111000000,
    0b0000111111110000,
    0b0001110110111000,
    0b0011111011011100,
    0b0011101101111100,
    0b0011110110111100,
    0b0011101101111100,
    0b0011111011011100,
    0b0011110110111100,
    0b0011101101111100,
    0b0011111011011100,
    0b0001110110111000,
    0b0000111111110000,
    0b0000001111000000,
    0b0000000000000000,
    0b0000000000000000,
];

// Babytchi — tiny round body, tuft on top, dot eyes, small mouth, stubby feet
#[rustfmt::skip]
const BABYTCHI: Sprite = [
    0b0000000000000000,
    0b0000000110000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001111001111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000001111000000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
];

// Marutchi — round body, big oval eyes, wide smile, small feet
#[rustfmt::skip]
const MARUTCHI: Sprite = [
    0b0000000000000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011100110011100,
    0b0011100110011100,
    0b0011111111111100,
    0b0011100000011100,
    0b0011110000111100,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000011001100000,
    0b0000011001100000,
    0b0000000000000000,
    0b0000000000000000,
];

// Tamatchi — tall oval body, pointy ear tufts, round eyes, smile
#[rustfmt::skip]
const TAMATCHI: Sprite = [
    0b0001100000011000,
    0b0000110000110000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001110000111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000001111000000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
];

// Kuchitamatchi — round body, beak/bill protruding right, dot eyes
#[rustfmt::skip]
const KUCHITAMATCHI: Sprite = [
    0b0000000000000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001111111111000,
    0b0001111111111110,
    0b0001111111111110,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
];

// Mametchi — ear-cap on top (wider than head), round eyes, happy mouth
#[rustfmt::skip]
const MAMETCHI: Sprite = [
    0b0000000000000000,
    0b0011111111111100,
    0b0011111111111100,
    0b0001111111111000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001110000111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
];

// Ginjirotchi — round body, small horns/bumps on top, gentle face
#[rustfmt::skip]
const GINJIROTCHI: Sprite = [
    0b0000000000000000,
    0b0000010000100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011111111111100,
    0b0011011111101100,
    0b0011011111101100,
    0b0011111111111100,
    0b0011111001111100,
    0b0011111111111100,
    0b0001111111111000,
    0b0000111111110000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
];

// Maskutchi — angular head, rectangular eyes (mask-like), stern mouth
#[rustfmt::skip]
const MASKUTCHI: Sprite = [
    0b0000000000000000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011111111111100,
    0b0011100011100100,
    0b0011100011100100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011110000111100,
    0b0011111111111100,
    0b0001111111111000,
    0b0000111111110000,
    0b0000010000100000,
    0b0000110000110000,
    0b0000000000000000,
    0b0000000000000000,
];

// Kuchipatchi — chubby round body, duck bill, happy eyes
#[rustfmt::skip]
const KUCHIPATCHI: Sprite = [
    0b0000000000000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011111111111100,
    0b0011011111101100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111110,
    0b0011111111111110,
    0b0001111111111000,
    0b0000111111110000,
    0b0000010000100000,
    0b0000111001110000,
    0b0000000000000000,
    0b0000000000000000,
];

// Nyorotchi — snake/worm, long wavy body, small head with eyes
#[rustfmt::skip]
const NYOROTCHI: Sprite = [
    0b0000000000000000,
    0b0000000000000000,
    0b0000111110000000,
    0b0001111111000000,
    0b0001101101000000,
    0b0001111111000000,
    0b0001110011000000,
    0b0000111110000000,
    0b0000011100000000,
    0b0000001110000000,
    0b0000011111000000,
    0b0000111111100000,
    0b0001111111110000,
    0b0000111111100000,
    0b0000011111000000,
    0b0000000000000000,
];

// Tarakotchi — antennae on top, round body, frowning mouth, wide feet
#[rustfmt::skip]
const TARAKOTCHI: Sprite = [
    0b0000100000010000,
    0b0000010000100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011111111111100,
    0b0011011111101100,
    0b0011111111111100,
    0b0011110000111100,
    0b0011111111111100,
    0b0001111111111000,
    0b0000111111110000,
    0b0000010000100000,
    0b0000111001110000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
];

// Oyajitchi — bald head, moustache, old man face
#[rustfmt::skip]
const OYAJITCHI: Sprite = [
    0b0000001111000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001100110011000,
    0b0001111001111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
];

// Sleeping — generic sleeping pose, closed eyes, z's
#[rustfmt::skip]
const SLEEPING: Sprite = [
    0b0000000000000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001110110111000,
    0b0001111111111000,
    0b0001111111111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000000000000000,
    0b0000000000111000,
    0b0000000001000000,
    0b0000000000011000,
    0b0000000000000000,
    0b0000000000000000,
];

// Sick — sweat drop, X eyes, wavy mouth
#[rustfmt::skip]
const SICK: Sprite = [
    0b0000000000010000,
    0b0000011111101000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001010110101000,
    0b0001111111111000,
    0b0001111001111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000001111000000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
];

// Dead — ghost/angel with halo, X eyes, wavy bottom
#[rustfmt::skip]
const DEAD: Sprite = [
    0b0000001111000000,
    0b0000010000100000,
    0b0000001111000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001010110101000,
    0b0001111111111000,
    0b0001111001111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000010101010000,
    0b0000001010100000,
    0b0000000000000000,
    0b0000000000000000,
];

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tama_core::state::PetState;

    #[test]
    fn braille_output_dimensions() {
        let s = sprite_to_braille(&EGG);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 4, "expected 4 lines of braille");
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(), 8,
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
        let s = get_sprite(&state);
        assert!(!s.is_empty());
        let expected = sprite_to_braille(&EGG);
        assert_eq!(s, expected);
    }

    #[test]
    fn get_sprite_returns_dead_for_dead_pet() {
        let mut state = PetState::new_egg(Utc::now());
        state.is_alive = false;
        state.stage = LifeStage::Dead;
        let s = get_sprite(&state);
        let expected = sprite_to_braille(&DEAD);
        assert_eq!(s, expected);
    }

    #[test]
    fn all_characters_have_sprites() {
        let chars = [
            Character::Babytchi, Character::Marutchi, Character::Tamatchi,
            Character::Kuchitamatchi, Character::Mametchi, Character::Ginjirotchi,
            Character::Maskutchi, Character::Kuchipatchi, Character::Nyorotchi,
            Character::Tarakotchi, Character::Oyajitchi,
        ];
        for ch in &chars {
            let sprite = idle_sprite(ch);
            let s = sprite_to_braille(sprite);
            assert!(!s.is_empty(), "sprite for {:?} should not be empty", ch);
        }
    }
}
