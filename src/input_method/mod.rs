use crate::{Application, KeyboardStatus};

#[derive(Copy, Clone, Debug)]
pub enum HardKeyButton {
    CloseButton,
}

impl HardKeyButton {
    pub const VALUES: [HardKeyButton; 1] = [HardKeyButton::CloseButton];
}

pub(crate) enum InputNextAction {
    EnterChar(char),
    Extra(fn(&mut KeyboardStatus)),
    Intrinsic(fn(&mut Application)),
}

#[derive(Clone)]
pub(crate) struct CleKeyInputTable<'a> {
    pub starts_ime: bool,
    pub table: [CleKeyButton<'a>; 8 * 8],
}

#[derive(Copy, Clone)]
pub(crate) struct CleKeyButton<'a>(pub &'a [CleKeyButtonAction<'a>]);

pub(crate) struct CleKeyButtonAction<'a> {
    pub shows: &'a str,
    pub action: InputNextAction,
}

impl<'a> CleKeyButton<'a> {
    #[allow(dead_code)]
    pub(crate) const fn empty() -> CleKeyButton<'a> {
        CleKeyButton(&[])
    }

    #[allow(dead_code)]
    pub(crate) const fn todo() -> CleKeyButton<'a> {
        CleKeyButton(&[])
    }

    pub(crate) const fn builtin() -> CleKeyButton<'a> {
        CleKeyButton(&[])
    }
}

macro_rules! char_button {
    (
        $($char: expr),+ $(,)*
    ) => {
        CleKeyButton(&[$(CleKeyButtonAction{
            shows: $crate::char_to_str!($char),
            action: InputNextAction::EnterChar($char),
        },)*])
    };
}

macro_rules! single_extra_action {
    ($shows: expr => $action: expr) => {
        CleKeyButton(&[CleKeyButtonAction {
            shows: $shows,
            action: InputNextAction::Extra($action),
        }])
    };
}

macro_rules! replace_last_char {
    ($vis: vis fn $name: ident { $($tt:tt)* }) => {
        $vis fn $name(status: &mut KeyboardStatus) {
            if let Some(c) = status.buffer.pop() {
                status.buffer.push({
                    static MAPPING: [char; 6 * 16] = {
                        let mut init = ['\0'; 6 * 16];
                        replace_last_char!(@first_init_0 init; 0, 1, 2, 3, 4, 5);
                        replace_last_char!(@init init; $($tt)*);
                        init
                    };

                    if matches!(c as u32, 0x3040..=0x309F) {
                        MAPPING[(c as u32 - 0x3040) as usize]
                    } else {
                        c
                    }
                })
            }
        }
    };

    (@first_init_0 $init: expr; $($e: expr),*) => {
        $( replace_last_char!(@first_init_1 $init; $e; 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15); )*
    };
    (@first_init_1 $init: expr; $b: expr; $($e: expr),*) => {
        $( replace_last_char!(@first_init_set $init; $b * 16 + $e); )*
    };
    (@first_init_set $init: expr; $v: expr) => {
        // unstable
        //$init[$v] = unsafe { std::char::from_u32_unchecked($v + 0x3040) };
        $init[$v] = unsafe { std::mem::transmute::<u32, char>($v + 0x3040) };
    };

    (@init $init: expr; $a: literal <=> $b: literal , $($tt:tt)*) => {
        $init[($a as u32 - 0x3040) as usize] = $b;
        $init[($b as u32 - 0x3040) as usize] = $a;
        replace_last_char!(@init $init; $($tt)*);
    };
    (@init $init: expr; $a: literal => $b: literal , $($tt:tt)*) => {
        $init[($a as u32 - 0x3040) as usize] = $b;
        replace_last_char!(@init $init; $($tt)*);
    };
    (@init $init: expr;) => {};
}

pub(crate) static SIGNS_TABLE: &CleKeyInputTable = &CleKeyInputTable {
    starts_ime: false,
    table: [
        char_button!('('),
        char_button!(')'),
        char_button!('['),
        char_button!(']'),
        char_button!('{'),
        char_button!('}'),
        char_button!('<'),
        char_button!('>'),
        char_button!('/'),
        char_button!('\\'),
        char_button!(';'),
        char_button!(':'),
        char_button!('-'),
        char_button!('+'),
        char_button!('_'),
        char_button!('='),
        char_button!('"'),
        char_button!('\''),
        char_button!('#'),
        char_button!('1'),
        char_button!('2'),
        char_button!('3'),
        char_button!('4'),
        char_button!('5'),
        char_button!('.'),
        char_button!(','),
        char_button!('!'),
        char_button!('6'),
        char_button!('7'),
        char_button!('8'),
        char_button!('9'),
        char_button!('0'),
        char_button!('&'),
        char_button!('*'),
        char_button!('¥'),
        char_button!('€'),
        char_button!('^'),
        char_button!('%'),
        char_button!('!'),
        char_button!('?'),
        char_button!('~'),
        char_button!('`'),
        char_button!('@'),
        char_button!('|'),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::builtin(),
        CleKeyButton::builtin(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::builtin(),
        CleKeyButton::builtin(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::empty(),
        CleKeyButton::builtin(),
        CleKeyButton::builtin(),
    ],
};

pub(crate) static ENGLISH_TABLE: &CleKeyInputTable = &CleKeyInputTable {
    starts_ime: false,
    table: [
        char_button!('a'),
        char_button!('A'),
        char_button!('b'),
        char_button!('B'),
        char_button!('c'),
        char_button!('C'),
        char_button!('d'),
        char_button!('D'),
        char_button!('e'),
        char_button!('E'),
        char_button!('f'),
        char_button!('F'),
        char_button!('g'),
        char_button!('G'),
        char_button!('h'),
        char_button!('H'),
        char_button!('i'),
        char_button!('I'),
        char_button!('j'),
        char_button!('J'),
        char_button!('k'),
        char_button!('K'),
        char_button!('l'),
        char_button!('L'),
        char_button!('m'),
        char_button!('M'),
        char_button!('n'),
        char_button!('N'),
        char_button!('o'),
        char_button!('O'),
        char_button!('p'),
        char_button!('P'),
        char_button!('q'),
        char_button!('Q'),
        char_button!('r'),
        char_button!('R'),
        char_button!('s'),
        char_button!('S'),
        char_button!('?'),
        char_button!('!'),
        char_button!('t'),
        char_button!('T'),
        char_button!('u'),
        char_button!('U'),
        char_button!('v'),
        char_button!('V'),
        CleKeyButton::builtin(),
        CleKeyButton::builtin(),
        char_button!('w'),
        char_button!('W'),
        char_button!('x'),
        char_button!('X'),
        char_button!('y'),
        char_button!('Y'),
        CleKeyButton::builtin(),
        CleKeyButton::builtin(),
        char_button!('z'),
        char_button!('Z'),
        char_button!('"'),
        char_button!('.'),
        char_button!('\''),
        char_button!(','),
        CleKeyButton::builtin(),
        CleKeyButton::builtin(),
    ],
};

pub(crate) static JAPANESE_INPUT: &CleKeyInputTable = &CleKeyInputTable {
    starts_ime: true,
    table: [
        char_button!('あ', 'ぁ'),
        char_button!('い', 'ぃ'),
        char_button!('う', 'ぅ', 'ゔ'),
        char_button!('え', 'ぇ'),
        char_button!('お', 'ぉ'),
        char_button!('よ', 'ょ'),
        char_button!('ゆ', 'ゅ'),
        char_button!('や', 'ゃ'),
        char_button!('か', 'が'),
        char_button!('き', 'ぎ'),
        char_button!('く', 'ぐ'),
        char_button!('け', 'げ'),
        char_button!('こ', 'ご'),
        char_button!('ん'),
        char_button!('を'),
        char_button!('わ'),
        char_button!('さ', 'ざ'),
        char_button!('し', 'じ'),
        char_button!('す', 'ず'),
        char_button!('せ', 'ぜ'),
        char_button!('そ', 'ぞ'),
        char_button!('「'),
        char_button!('。'),
        char_button!('?'),
        char_button!('た', 'だ'),
        char_button!('ち', 'ぢ'),
        char_button!('つ', 'っ', 'づ'),
        char_button!('て', 'で'),
        char_button!('と', 'ど'),
        char_button!('」'),
        char_button!('、'),
        char_button!('!'),
        char_button!('な'),
        char_button!('に'),
        char_button!('ぬ'),
        char_button!('ね'),
        char_button!('の'),
        single_extra_action!("小" => jp_small),
        single_extra_action!("\u{2B1A}\u{3099}" => jp_dakuten),
        single_extra_action!("\u{2B1A}\u{309a}" => jp_handakuten),
        char_button!('は', 'ば', 'ぱ'),
        char_button!('ひ', 'び', 'ぴ'),
        char_button!('ふ', 'ぶ', 'ぷ'),
        char_button!('へ', 'べ', 'ぺ'),
        char_button!('ほ', 'ぼ', 'ぽ'),
        CleKeyButton::empty(),
        CleKeyButton::builtin(),
        CleKeyButton::builtin(),
        char_button!('ま'),
        char_button!('み'),
        char_button!('む'),
        char_button!('め'),
        char_button!('も'),
        char_button!('ー'),
        CleKeyButton::builtin(),
        CleKeyButton::builtin(),
        char_button!('ら'),
        char_button!('り'),
        char_button!('る'),
        char_button!('れ'),
        char_button!('ろ'),
        char_button!('〜'),
        CleKeyButton::builtin(),
        CleKeyButton::builtin(),
    ],
};

replace_last_char!(
    fn jp_small {
        'あ' <=> 'ぁ', 'い' <=> 'ぃ', 'う' <=> 'ぅ', 'え' <=> 'ぇ', 'お' <=> 'ぉ',
        'や' <=> 'ゃ', 'ゆ' <=> 'ゅ', 'よ' <=> 'ょ',
        'つ' <=> 'っ', 'わ' <=> 'ゎ', 'か' <=> 'ゕ', 'け' <=> 'ゖ',
    }
);

replace_last_char!(
    fn jp_dakuten {
        'か' <=> 'が', 'き' <=> 'ぎ', 'く' <=> 'ぐ', 'け' <=> 'げ', 'こ' <=> 'ご',
        'さ' <=> 'ざ', 'し' <=> 'じ', 'す' <=> 'ず', 'せ' <=> 'ぜ', 'そ' <=> 'ぞ',
        'た' <=> 'だ', 'ち' <=> 'ぢ', 'つ' <=> 'づ', 'て' <=> 'で', 'と' <=> 'ど',
        'は' <=> 'ば', 'ひ' <=> 'び', 'ふ' <=> 'ぶ', 'へ' <=> 'べ', 'ほ' <=> 'ぼ',
        'う' <=> 'ゔ',
        'ぱ' => 'ば', 'ぴ' => 'び', 'ぷ' => 'ぶ', 'ぺ' => 'べ', 'ぽ' => 'ぼ',
    }
);

replace_last_char!(
    fn jp_handakuten {
        'は' <=> 'ぱ', 'ひ' <=> 'ぴ', 'ふ' <=> 'ぷ', 'へ' <=> 'ぺ', 'ほ' <=> 'ぽ',
        'ば' => 'ぱ', 'び' => 'ぴ', 'ぶ' => 'ぷ', 'べ' => 'ぺ', 'ぼ' => 'ぽ',
    }
);
