use crate::utils::ToTuple;
use glam::UVec2;

#[derive(Copy, Clone, Debug)]
pub enum HardKeyButton {
    CloseButton,
}

impl HardKeyButton {
    pub const VALUES: [HardKeyButton; 1] = [HardKeyButton::CloseButton];
}

pub struct InputNextAction {
    flush: bool,
    action: InputNextMoreAction,
}

impl InputNextAction {
    fn new(flush: bool, action: InputNextMoreAction) -> Self {
        Self { flush, action }
    }

    /// nothing to do
    pub fn nop(flush: bool) -> Self {
        Self::new(flush, InputNextMoreAction::Nop)
    }

    /// move to next plane
    pub fn move_to_next_plane(flush: bool) -> Self {
        Self::new(flush, InputNextMoreAction::MoveToNextPlane)
    }

    /// move to sign plane
    /// this will be used to back to char plane in sign plane
    pub fn move_to_sign_plane(flush: bool) -> Self {
        Self::new(flush, InputNextMoreAction::MoveToSignPlane)
    }

    /// enter additional char
    pub fn enter_char(flush: bool, char: char) -> Self {
        Self::new(flush, InputNextMoreAction::EnterChar(char))
    }

    /// enter delete key
    pub fn remove_last_char(flush: bool) -> Self {
        Self::new(flush, InputNextMoreAction::RemoveLastChar)
    }

    /// close clekey
    pub fn close_keyboard(flush: bool) -> Self {
        Self::new(flush, InputNextMoreAction::CloseKeyboard)
    }

    /// enter new line
    pub fn new_line(flush: bool) -> Self {
        Self::new(flush, InputNextMoreAction::NewLine)
    }

    pub fn flush(&self) -> bool {
        self.flush
    }
    pub fn action(&self) -> &InputNextMoreAction {
        &self.action
    }
}

pub enum InputNextMoreAction {
    Nop,
    MoveToNextPlane,
    MoveToSignPlane,
    EnterChar(char),
    RemoveLastChar,
    CloseKeyboard,
    NewLine,
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

    #[must_use]
    fn buffer(&self) -> &str {
        ""
    }

    #[must_use]
    fn get_and_clear_buffer(&mut self) -> String {
        String::new()
    }

    fn on_input(&mut self, stick: UVec2) -> InputNextAction;

    fn on_hard_input(&mut self, button: HardKeyButton) -> InputNextAction;
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
    _reserved: (),
}

impl SignsInput {
    pub fn new() -> Self {
        Self { _reserved: () }
    }
}

impl IInputMethod for SignsInput {
    #[rustfmt::skip]
    fn get_table(&self) -> &[&str; 8 * 8] {
        return &[
            "(", ")", "[", "]", "{", "}", "<", ">",
            "/", "\\", ";",  ":", "-", "+", "_", "=",
            "\"", "'", "#", "1", "2", "3", "4", "5",
            ".", ",", "!", "6", "7", "8", "9", "0",
            "&", "*", "¥", "€", "^", "%", "!", "?",
            "~", "`", "@", "|", "", "", "Close", RETURN_ICON,
            "", "", "", "", "", "", BACKSPACE_ICON, SPACE_ICON,
            "", "", "", "", "", "", SIGNS_ICON, NEXT_PLANE_ICON,
        ];
    }

    fn on_input(&mut self, stick: UVec2) -> InputNextAction {
        match stick.to_tuple() {
            (5, 6) => InputNextAction::close_keyboard(false),
            (5, 7) => InputNextAction::new_line(false),
            (6, 6) => InputNextAction::remove_last_char(false),
            (6, 7) => InputNextAction::enter_char(false, ' '),
            (7, 6) => InputNextAction::move_to_sign_plane(false),
            (7, 7) => InputNextAction::move_to_next_plane(false),
            (0..=4, _) | (5, 0..=3) => {
                InputNextAction::enter_char(false, get_table_char!(self.get_table(), stick))
            }
            (0..=7, 0..=7) => InputNextAction::nop(false),
            (8..=u32::MAX, _) | (_, 8..=u32::MAX) => unreachable!(),
        }
    }

    fn on_hard_input(&mut self, button: HardKeyButton) -> InputNextAction {
        match button {
            HardKeyButton::CloseButton => InputNextAction::close_keyboard(false),
        }
    }
}

pub struct EnglishInput {
    _reserved: (),
}

impl EnglishInput {
    pub fn new() -> Self {
        Self { _reserved: () }
    }
}

impl IInputMethod for EnglishInput {
    #[rustfmt::skip]
    fn get_table(&self) -> &[&str; 8 * 8] {
        return &[
            "a", "A", "b", "B", "c", "C", "d", "D",
            "e", "E", "f", "F", "g", "G", "h", "H",
            "i", "I", "j", "J", "k", "K", "l", "L",
            "m", "M", "n", "N", "o", "O", "p", "P",
            "q", "Q", "r", "R", "s", "S", "?", "!",
            "t", "T", "u", "U", "v", "V", "Close", RETURN_ICON,
            "w", "W", "x", "X", "y", "Y", BACKSPACE_ICON, SPACE_ICON,
            "z", "Z", "\"", ".", "\'", ",", SIGNS_ICON, NEXT_PLANE_ICON,
        ];
    }

    fn on_input(&mut self, stick: UVec2) -> InputNextAction {
        match stick.to_tuple() {
            (5, 6) => InputNextAction::close_keyboard(false),
            (5, 7) => InputNextAction::new_line(false),
            (6, 6) => InputNextAction::remove_last_char(false),
            (6, 7) => InputNextAction::enter_char(false, ' '),
            (7, 6) => InputNextAction::move_to_sign_plane(false),
            (7, 7) => InputNextAction::move_to_next_plane(false),
            (0..=7, 0..=7) => InputNextAction::enter_char(
                false,
                self.get_table()[stick_index(stick) as usize]
                    .chars()
                    .next()
                    .unwrap(),
            ),
            (8..=u32::MAX, _) | (_, 8..=u32::MAX) => unreachable!(),
        }
    }

    fn on_hard_input(&mut self, button: HardKeyButton) -> InputNextAction {
        match button {
            HardKeyButton::CloseButton => InputNextAction::close_keyboard(false),
        }
    }
}

pub struct JapaneseInput {
    buffer: String,
    table: [&'static str; 8 * 8],
}

impl JapaneseInput {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
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

    fn set_inputted_table(&mut self) {
        self.table[stick_index(UVec2::new(5, 6)) as usize] = "閉じる";
        self.table[stick_index(UVec2::new(5, 7)) as usize] = RETURN_ICON;
    }

    fn set_inputting_table(&mut self) {
        self.table[stick_index(UVec2::new(5, 6)) as usize] = "変換";
        self.table[stick_index(UVec2::new(5, 7)) as usize] = "確定";
    }
}

const DAKUTEN_ICON: &'static str = "\u{2B1A}\u{3099}";
const HANDAKUTEN_ICON: &'static str = "\u{2B1A}\u{309a}";

impl IInputMethod for JapaneseInput {
    fn get_table(&self) -> &[&str; 8 * 8] {
        &self.table
    }

    fn buffer(&self) -> &str {
        &self.buffer
    }

    fn get_and_clear_buffer(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }

    fn on_input(&mut self, stick: UVec2) -> InputNextAction {
        match stick.to_tuple() {
            (4, 5) => {
                // small char
                if let Some(c) = self.buffer.pop() {
                    self.buffer.push(match c {
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
                InputNextAction::nop(false)
            }
            (4, 6) => {
                // add Dakuten
                if let Some(c) = self.buffer.pop() {
                    self.buffer.push(match c {
                        'か' | 'き' | 'く' | 'け' | 'こ' | 'さ' | 'し' | 'す' | 'せ' | 'そ'
                        | 'た' | 'ち' | 'つ' | 'て' | 'と' | 'は' | 'ひ' | 'ふ' | 'へ' | 'ほ' =>
                        unsafe { char::from_u32_unchecked(c as u32 + 1) },
                        'が' | 'ぎ' | 'ぐ' | 'げ' | 'ご' | 'ざ' | 'じ' | 'ず' | 'ぜ' | 'ぞ'
                        | 'だ' | 'ぢ' | 'づ' | 'で' | 'ど' | 'ば' | 'び' | 'ぶ' | 'べ' | 'ぼ' =>
                        unsafe { char::from_u32_unchecked(c as u32 - 1) },
                        'う' => 'ゔ',
                        'ゔ' => 'う',
                        other => other,
                    })
                }
                InputNextAction::nop(false)
            }
            (4, 7) => {
                // add Handakuten
                if let Some(c) = self.buffer.pop() {
                    self.buffer.push(match c {
                        'は' | 'ひ' | 'ふ' | 'へ' | 'ほ' => unsafe {
                            char::from_u32_unchecked(c as u32 + 2)
                        },
                        'ぱ' | 'ぴ' | 'ぷ' | 'ぺ' | 'ぽ' => unsafe {
                            char::from_u32_unchecked(c as u32 - 2)
                        },
                        'う' => 'ゔ',
                        'ゔ' => 'う',
                        other => other,
                    })
                }
                InputNextAction::nop(false)
            }
            (5, 5) => InputNextAction::nop(false),
            ////////////
            (5, 6) => {
                if self.buffer.is_empty() {
                    InputNextAction::close_keyboard(false)
                } else {
                    InputNextAction::nop(false)
                }
            }
            (5, 7) => {
                if self.buffer.is_empty() {
                    InputNextAction::new_line(true)
                } else {
                    InputNextAction::nop(true)
                }
            }
            (6, 6) => {
                if let Some(_) = self.buffer.pop() {
                    if self.buffer.is_empty() {
                        self.set_inputted_table();
                    }
                    InputNextAction::nop(false)
                } else {
                    InputNextAction::remove_last_char(false)
                }
            }
            (6, 7) => {
                if self.buffer.is_empty() {
                    InputNextAction::enter_char(true, ' ')
                } else {
                    self.buffer.push(' ');
                    InputNextAction::nop(false)
                }
            }
            (7, 6) => InputNextAction::move_to_sign_plane(true),
            (7, 7) => InputNextAction::move_to_next_plane(true),
            ////////////
            (2 | 3, 5 | 6 | 7) | (6 | 7, 5) => {
                if self.buffer.is_empty() {
                    InputNextAction::enter_char(false, get_table_char!(self.get_table(), stick))
                } else {
                    self.buffer.push_str(&get_table_str!(self.table, stick));
                    self.set_inputting_table();
                    InputNextAction::nop(false)
                }
            }
            (0..=7, 0..=7) => {
                self.buffer.push_str(&get_table_str!(self.table, stick));
                self.set_inputting_table();
                InputNextAction::nop(false)
            }
            (8..=u32::MAX, _) | (_, 8..=u32::MAX) => unreachable!(),
        }
    }

    fn on_hard_input(&mut self, button: HardKeyButton) -> InputNextAction {
        match button {
            HardKeyButton::CloseButton => InputNextAction::close_keyboard(false),
        }
    }
}
