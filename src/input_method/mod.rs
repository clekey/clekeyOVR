use crate::utils::ToTuple;
use glam::UVec2;

#[derive(Copy, Clone, Debug)]
pub enum HardKeyButton {
    CloseButton,
}

impl HardKeyButton {
    pub const VALUES: [HardKeyButton; 1] = [HardKeyButton::CloseButton];
}

impl InputNextAction {
    /// nothing to do
    pub fn nop() -> Self {
        InputNextAction::Nop
    }

    /// enter additional char
    pub fn enter_char(char: char) -> Self {
        InputNextAction::EnterChar(char)
    }
}

pub enum InputNextAction {
    Nop,
    EnterChar(char),
}

macro_rules! get_table_str {
    ($table: expr, $stick: expr) => {
        $table[stick_index($stick) as usize]
    };
}

macro_rules! get_table_char {
    ($table: expr, $stick: expr) => {
        get_table_str!($table, $stick).chars().next().unwrap()
    };
}

pub trait IInputMethod {
    #[must_use]
    fn get_table(&self) -> &[&str; 8 * 8];

    fn on_input(&mut self, stick: UVec2, buffer: &mut String) -> InputNextAction;

    fn on_hard_input(&mut self, button: HardKeyButton) -> InputNextAction;

    fn set_inputted_table(&mut self);
    fn set_inputting_table(&mut self);
}

pub fn stick_index(stick: UVec2) -> u8 {
    (stick.x * 8 + stick.y) as u8
}

const BACKSPACE_ICON: &str = "⌫";
const SPACE_ICON: &str = "␣";
const NEXT_PLANE_ICON: &str = "\u{1F310}"; // 🌐
const SIGNS_ICON: &str = "#+=";
const RETURN_ICON: &str = "⏎";

pub struct SignsInput {
    table: [&'static str; 8 * 8],
}

impl SignsInput {
    pub fn new() -> Self {
        Self {
            #[rustfmt::skip]
            table: [
                "(", ")", "[", "]", "{", "}", "<", ">",
                "/", "\\", ";",  ":", "-", "+", "_", "=",
                "\"", "'", "#", "1", "2", "3", "4", "5",
                ".", ",", "!", "6", "7", "8", "9", "0",
                "&", "*", "¥", "€", "^", "%", "!", "?",
                "~", "`", "@", "|", "", "", "Close", RETURN_ICON,
                "", "", "", "", "", "", BACKSPACE_ICON, SPACE_ICON,
                "", "", "", "", "", "", SIGNS_ICON, NEXT_PLANE_ICON,
            ],
        }
    }
}

impl IInputMethod for SignsInput {
    fn get_table(&self) -> &[&str; 8 * 8] {
        &self.table
    }

    fn on_input(&mut self, stick: UVec2, _: &mut String) -> InputNextAction {
        match stick.to_tuple() {
            (l @ (5 | 6 | 7), r @ (6 | 7)) => unreachable!("intrinsic keys: {}, {}", l, r),
            (0..=4, _) | (5, 0..=3) => {
                InputNextAction::enter_char(get_table_char!(self.get_table(), stick))
            }
            (0..=7, 0..=7) => InputNextAction::nop(),
            (8..=u32::MAX, _) | (_, 8..=u32::MAX) => {
                unreachable!("invalid keys: {}, {}", stick.x, stick.y)
            }
        }
    }

    fn on_hard_input(&mut self, button: HardKeyButton) -> InputNextAction {
        match button {
            HardKeyButton::CloseButton => unreachable!("intrinsic keys"),
        }
    }

    fn set_inputted_table(&mut self) {
        self.table[stick_index(UVec2::new(5, 6)) as usize] = "Close";
        self.table[stick_index(UVec2::new(5, 7)) as usize] = RETURN_ICON;
    }

    fn set_inputting_table(&mut self) {
        self.table[stick_index(UVec2::new(5, 6)) as usize] = "変換";
        self.table[stick_index(UVec2::new(5, 7)) as usize] = "確定";
    }
}

pub struct EnglishInput {
    table: [&'static str; 8 * 8],
}

impl EnglishInput {
    pub fn new() -> Self {
        Self {
            #[rustfmt::skip]
            table: [
                "a", "A", "b", "B", "c", "C", "d", "D",
                "e", "E", "f", "F", "g", "G", "h", "H",
                "i", "I", "j", "J", "k", "K", "l", "L",
                "m", "M", "n", "N", "o", "O", "p", "P",
                "q", "Q", "r", "R", "s", "S", "?", "!",
                "t", "T", "u", "U", "v", "V", "Close", RETURN_ICON,
                "w", "W", "x", "X", "y", "Y", BACKSPACE_ICON, SPACE_ICON,
                "z", "Z", "\"", ".", "\'", ",", SIGNS_ICON, NEXT_PLANE_ICON,
            ],
        }
    }
}

impl IInputMethod for EnglishInput {
    fn get_table(&self) -> &[&str; 8 * 8] {
        &self.table
    }

    fn on_input(&mut self, stick: UVec2, _: &mut String) -> InputNextAction {
        match stick.to_tuple() {
            (l @ (5 | 6 | 7), r @ (6 | 7)) => unreachable!("intrinsic keys: {}, {}", l, r),
            (0..=7, 0..=7) => InputNextAction::enter_char(
                self.get_table()[stick_index(stick) as usize]
                    .chars()
                    .next()
                    .unwrap(),
            ),
            (8..=u32::MAX, _) | (_, 8..=u32::MAX) => {
                unreachable!("invalid key: {}, {}", stick.x, stick.y)
            }
        }
    }

    fn on_hard_input(&mut self, button: HardKeyButton) -> InputNextAction {
        match button {
            HardKeyButton::CloseButton => unreachable!("intrinsic keys"),
        }
    }

    fn set_inputted_table(&mut self) {
        self.table[stick_index(UVec2::new(5, 6)) as usize] = "Close";
        self.table[stick_index(UVec2::new(5, 7)) as usize] = RETURN_ICON;
    }

    fn set_inputting_table(&mut self) {
        self.table[stick_index(UVec2::new(5, 6)) as usize] = "変換";
        self.table[stick_index(UVec2::new(5, 7)) as usize] = "確定";
    }
}

pub struct JapaneseInput {
    table: [&'static str; 8 * 8],
}

impl JapaneseInput {
    pub fn new() -> Self {
        Self {
            #[rustfmt::skip]
            table: [
                "あ", "い", "う", "え", "お", "よ", "ゆ", "や",
                "か", "き", "く", "け", "こ", "ん", "を", "わ",
                "さ", "し", "す", "せ", "そ", "「", "。", "?",
                "た", "ち", "つ", "て", "と", "」", "、", "!",
                "な", "に", "ぬ", "ね", "の", "小", DAKUTEN_ICON, HANDAKUTEN_ICON,
                "は", "ひ", "ふ", "へ", "ほ", "", "閉じる", RETURN_ICON,
                "ま", "み", "む", "め", "も", "ー", BACKSPACE_ICON, SPACE_ICON,
                "ら", "り", "る", "れ", "ろ", "〜", SIGNS_ICON, NEXT_PLANE_ICON,
            ],
        }
    }
}

const DAKUTEN_ICON: &'static str = "\u{2B1A}\u{3099}";
const HANDAKUTEN_ICON: &'static str = "\u{2B1A}\u{309a}";

impl IInputMethod for JapaneseInput {
    fn get_table(&self) -> &[&str; 8 * 8] {
        &self.table
    }

    fn on_input(&mut self, stick: UVec2, buffer: &mut String) -> InputNextAction {
        match stick.to_tuple() {
            (l @ (5 | 6 | 7), r @ (6 | 7)) => unreachable!("intrinsic keys: {}, {}", l, r),
            (4, 5) => {
                // small char
                if let Some(c) = buffer.pop() {
                    buffer.push(match c {
                        'あ' | 'い' | 'う' | 'え' | 'お' | 'つ' | 'や' | 'ゆ' | 'よ' | 'わ' =>
                        unsafe { char::from_u32_unchecked(c as u32 - 1) },
                        'ぁ' | 'ぃ' | 'ぅ' | 'ぇ' | 'ぉ' | 'っ' | 'ゃ' | 'ゅ' | 'ょ' | 'ゎ' =>
                        unsafe { char::from_u32_unchecked(c as u32 + 1) },
                        'か' => 'ゕ',
                        'ゕ' => 'か',
                        'け' => 'ゖ',
                        'ゖ' => 'け',
                        other => other,
                    })
                }
                InputNextAction::nop()
            }
            (4, 6) => {
                // add Dakuten
                if let Some(c) = buffer.pop() {
                    buffer.push(match c {
                        'か' | 'き' | 'く' | 'け' | 'こ' | 'さ' | 'し' | 'す' | 'せ' | 'そ'
                        | 'た' | 'ち' | 'つ' | 'て' | 'と' | 'は' | 'ひ' | 'ふ' | 'へ' | 'ほ' =>
                        unsafe { char::from_u32_unchecked(c as u32 + 1) },
                        'が' | 'ぎ' | 'ぐ' | 'げ' | 'ご' | 'ざ' | 'じ' | 'ず' | 'ぜ' | 'ぞ'
                        | 'だ' | 'ぢ' | 'づ' | 'で' | 'ど' | 'ば' | 'び' | 'ぶ' | 'べ' | 'ぼ' =>
                        unsafe { char::from_u32_unchecked(c as u32 - 1) },
                        'ぱ' | 'ぴ' | 'ぷ' | 'ぺ' | 'ぽ' => unsafe {
                            char::from_u32_unchecked(c as u32 - 1)
                        },
                        'う' => 'ゔ',
                        'ゔ' => 'う',
                        other => other,
                    })
                }
                InputNextAction::nop()
            }
            (4, 7) => {
                // add Handakuten
                if let Some(c) = buffer.pop() {
                    buffer.push(match c {
                        'は' | 'ひ' | 'ふ' | 'へ' | 'ほ' => unsafe {
                            char::from_u32_unchecked(c as u32 + 2)
                        },
                        'ぱ' | 'ぴ' | 'ぷ' | 'ぺ' | 'ぽ' => unsafe {
                            char::from_u32_unchecked(c as u32 - 2)
                        },
                        'ば' | 'び' | 'ぶ' | 'べ' | 'ぼ' => unsafe {
                            char::from_u32_unchecked(c as u32 + 1)
                        },
                        'う' => 'ゔ',
                        'ゔ' => 'う',
                        other => other,
                    })
                }
                InputNextAction::nop()
            }
            (5, 5) => InputNextAction::nop(),
            ////////////
            ////////////
            (2 | 3, 5 | 6 | 7) | (6 | 7, 5) => {
                InputNextAction::enter_char(get_table_char!(self.get_table(), stick))
            }
            (0..=7, 0..=7) => InputNextAction::enter_char(get_table_char!(self.table, stick)),
            (8..=u32::MAX, _) | (_, 8..=u32::MAX) => {
                unreachable!("invalid key: {}, {}", stick.x, stick.y)
            }
        }
    }

    fn on_hard_input(&mut self, button: HardKeyButton) -> InputNextAction {
        match button {
            HardKeyButton::CloseButton => unreachable!("intrinsic keys"),
        }
    }

    fn set_inputted_table(&mut self) {
        self.table[stick_index(UVec2::new(5, 6)) as usize] = "閉じる";
        self.table[stick_index(UVec2::new(5, 7)) as usize] = RETURN_ICON;
    }

    fn set_inputting_table(&mut self) {
        self.table[stick_index(UVec2::new(5, 6)) as usize] = "変換";
        self.table[stick_index(UVec2::new(5, 7)) as usize] = "確定";
    }
}
