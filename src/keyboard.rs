use {derive_more::Display, midly::num::u7, std::ops::RangeInclusive};

const KEY_RANGE_88: RangeInclusive<u8> = 21..=108;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Key out of range")]
    KeyOutOfRange,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum KeyPos {
    C = 0,
    CSharp = 1,
    D = 2,
    DSharp = 3,
    E = 4,
    F = 5,
    FSharp = 6,
    G = 7,
    GSharp = 8,
    A = 9,
    ASharp = 10,
    B = 11,
}

impl KeyPos {
    pub const fn oct(self, oct: u8) -> Key {
        Key::new(self, oct)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::C => "C",
            Self::CSharp => "C#",
            Self::D => "D",
            Self::DSharp => "D#",
            Self::E => "E",
            Self::F => "F",
            Self::FSharp => "F#",
            Self::G => "G",
            Self::GSharp => "G#",
            Self::A => "A",
            Self::ASharp => "A#",
            Self::B => "B",
        }
    }

    /// Position within the octave.
    pub fn oct_pos(&self) -> usize {
        match self {
            Self::C => 0,
            Self::CSharp => 0,
            Self::D => 1,
            Self::DSharp => 1,
            Self::E => 2,
            Self::F => 3,
            Self::FSharp => 2,
            Self::G => 4,
            Self::GSharp => 3,
            Self::A => 5,
            Self::ASharp => 4,
            Self::B => 6,
        }
    }

    pub fn is_natural(&self) -> bool {
        !self.is_sharp()
    }

    pub fn is_sharp(&self) -> bool {
        match self {
            Self::CSharp | Self::DSharp | Self::FSharp | Self::GSharp | Self::ASharp => true,
            _ => false,
        }
    }

    pub fn pitch_name(&self) -> &'static str {
        match self {
            Self::C | Self::CSharp => "c",
            Self::D | Self::DSharp => "d",
            Self::E => "e",
            Self::F | Self::FSharp => "f",
            Self::G | Self::GSharp => "g",
            Self::A | Self::ASharp => "a",
            Self::B => "b",
        }
    }

    fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::C,
            1 => Self::CSharp,
            2 => Self::D,
            3 => Self::DSharp,
            4 => Self::E,
            5 => Self::F,
            6 => Self::FSharp,
            7 => Self::G,
            8 => Self::GSharp,
            9 => Self::A,
            10 => Self::ASharp,
            11 => Self::B,
            _ => panic!("invalid value: {}", val),
        }
    }
}

#[derive(Display, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[display("{}/{}", pos.as_str(), oct)]
pub struct Key {
    pub pos: KeyPos,
    pub oct: u8,
}

impl Key {
    // Position of `C0`, as we're not interested in `-1` octave.
    const OFFSET: u8 = 12;

    pub const fn new(pos: KeyPos, oct: u8) -> Self {
        Self { pos, oct }
    }

    pub fn is_natural(&self) -> bool {
        self.pos.is_natural()
    }

    pub fn is_sharp(&self) -> bool {
        self.pos.is_sharp()
    }

    pub fn prev(&self) -> Option<Key> {
        Self::try_from_midi(self.to_midi() - 1.into()).ok()
    }

    pub fn next(&self) -> Option<Key> {
        Self::try_from_midi(self.to_midi() + 1.into()).ok()
    }

    pub fn try_from_midi(key: u7) -> Result<Self, Error> {
        if is_valid_key(key) {
            let key = key.as_int() - Self::OFFSET;

            Ok(Self {
                pos: KeyPos::from_u8(key % 12),
                oct: key / 12,
            })
        } else {
            Err(Error::KeyOutOfRange)
        }
    }

    pub const fn to_midi(&self) -> u7 {
        u7::from_int_lossy(self.oct * 12 + self.pos as u8 + Self::OFFSET)
    }
}

#[derive(Debug, Clone)]
pub struct Keyboard {
    range: RangeInclusive<u8>,
}

impl Keyboard {
    fn new(range: RangeInclusive<u8>) -> Self {
        Self { range }
    }

    pub fn standard_88_key() -> Self {
        Self::new(KEY_RANGE_88)
    }

    pub fn first(&self) -> Key {
        Key::try_from_midi((*self.range.start()).into()).unwrap()
    }

    pub fn last(&self) -> Key {
        Key::try_from_midi((*self.range.end()).into()).unwrap()
    }

    pub fn num_keys(&self) -> usize {
        (self.range.end() - self.range.start()) as usize + 1
    }

    pub fn num_natural_keys(&self) -> usize {
        self.iter_natural_keys().count()
    }

    pub fn num_sharp_keys(&self) -> usize {
        self.iter_sharp_keys().count()
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = Key> {
        range(&self.first(), &self.last())
    }

    pub fn iter_natural_keys(&self) -> impl Iterator<Item = Key> {
        self.iter_keys().filter(Key::is_natural)
    }

    pub fn iter_sharp_keys(&self) -> impl Iterator<Item = Key> {
        self.iter_keys().filter(Key::is_sharp)
    }

    pub fn offset(&self, key: &Key) -> usize {
        let midi_code = key.to_midi().as_int();
        (midi_code - *self.range.start() as u8) as usize
    }

    pub fn natural_index(&self, key: &Key) -> Option<usize> {
        let first_key = self.first();

        if !key.is_natural() || *key < first_key {
            return None;
        }

        let oct_diff = (key.oct - first_key.oct) as usize;

        Some(if oct_diff == 0 {
            key.pos.oct_pos() - first_key.pos.oct_pos()
        } else {
            key.pos.oct_pos() + oct_diff * 7 - first_key.pos.oct_pos()
        })
    }
}

impl PartialOrd<Key> for Key {
    fn partial_cmp(&self, other: &Key) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_midi().cmp(&other.to_midi())
    }
}

/// Returns an iterator for range `start..=end`.
pub fn range(start: &Key, end: &Key) -> impl Iterator<Item = Key> + use<> {
    let start = start.to_midi().as_int();
    let end = end.to_midi().as_int();
    (start..=end).map(|key| Key::try_from_midi(key.into()).unwrap())
}

/// Checks whether the key code is in the valid range for a stadnard 88-key
/// piano (`21..=108`).
pub fn is_valid_key(key: u7) -> bool {
    KEY_RANGE_88.contains(&key.as_int())
}

#[cfg(test)]
mod test {
    use {super::*, wasm_bindgen_test::*};

    #[wasm_bindgen_test]
    fn key_codes() {
        assert_eq!(Key::try_from_midi(21.into()).unwrap(), Key {
            pos: KeyPos::A,
            oct: 0
        });
        assert_eq!(Key::try_from_midi(22.into()).unwrap(), Key {
            pos: KeyPos::ASharp,
            oct: 0
        });
        assert_eq!(Key::try_from_midi(23.into()).unwrap(), Key {
            pos: KeyPos::B,
            oct: 0
        });
        assert_eq!(Key::try_from_midi(24.into()).unwrap(), Key {
            pos: KeyPos::C,
            oct: 1
        });

        assert!(!is_valid_key(20.into()));
        assert!(!is_valid_key(109.into()));
        assert!(is_valid_key(21.into()));
        assert!(is_valid_key(108.into()));

        for key in 0..127u8 {
            if is_valid_key(key.into()) {
                let parsed = Key::try_from_midi(key.into()).unwrap();
                assert_eq!(key, parsed.to_midi());
            } else {
                assert!(Key::try_from_midi(key.into()).is_err())
            }
        }

        assert_eq!(KeyPos::ASharp.oct(0).prev(), Some(KeyPos::A.oct(0)));
        assert_eq!(KeyPos::ASharp.oct(0).next(), Some(KeyPos::B.oct(0)));
        assert_eq!(KeyPos::B.oct(0).next(), Some(KeyPos::C.oct(1)));
    }

    #[wasm_bindgen_test]
    fn keyboard() {
        let kbd = Keyboard::standard_88_key();
        assert_eq!(kbd.num_keys(), 88);
        assert_eq!(kbd.num_sharp_keys(), 36);
        assert_eq!(kbd.num_natural_keys(), 52);
        assert_eq!(kbd.offset(&KeyPos::A.oct(0)), 0);
        assert_eq!(kbd.offset(&KeyPos::C.oct(8)), 87);
        assert_eq!(kbd.natural_index(&KeyPos::A.oct(0)), Some(0));
        assert_eq!(kbd.natural_index(&KeyPos::C.oct(1)), Some(2));
        assert_eq!(kbd.natural_index(&KeyPos::C.oct(4)), Some(23));
        assert_eq!(kbd.natural_index(&KeyPos::C.oct(8)), Some(51));
    }
}
