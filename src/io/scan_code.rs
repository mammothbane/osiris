#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ScanCode {
    Escape = 0x1,
    One, Two, Three, Four, Five, Six, Seven, Eight, Nine, Zero, Dash, Equals, Backspace,
    Tab, Q, W, E, R, T, Y, U, I, O, P, LeftBracket, RightBracket,
    Enter,
    LeftCtl,
    A, S, D, F, G, H, J, K, L, Semicolon, SingleQuote,
    BackTick,
    LeftShift,
    Backslash,
    Z, X, C, V, B, N, M, Comma, Period, Slash, RightShift,
    NumStar,
    LeftAlt,
    Space,
    CapsLock,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10,
    NumLock, ScrollLock,
    Num7, Num8, Num9, NumMinus,
    Num4, Num5, Num6, NumPlus,
    Num1, Num2, Num3, Num0, NumDot,

    F11 = 0x57,
    F12,

    Extended = 0xE0,
}

impl ScanCode {
    pub fn ascii(&self) -> Option<u8> {
        use self::ScanCode::*;

        match self {
            A => Some(b'a'),
            B => Some(b'b'),
            C => Some(b'c'),
            D => Some(b'd'),
            E => Some(b'e'),
            F => Some(b'f'),
            G => Some(b'g'),
            H => Some(b'h'),
            I => Some(b'i'),
            J => Some(b'j'),
            K => Some(b'k'),
            L => Some(b'l'),
            M => Some(b'm'),
            N => Some(b'n'),
            O => Some(b'o'),
            P => Some(b'p'),
            Q => Some(b'q'),
            R => Some(b'r'),
            S => Some(b's'),
            T => Some(b't'),
            U => Some(b'u'),
            V => Some(b'v'),
            W => Some(b'w'),
            X => Some(b'x'),
            Y => Some(b'y'),
            Z => Some(b'z'),

            One | Num1 => Some(b'1'),
            Two | Num2 => Some(b'2'),
            Three | Num3 => Some(b'3'),
            Four | Num4 => Some(b'4'),
            Five | Num5 => Some(b'5'),
            Six | Num6 => Some(b'6'),
            Seven | Num7 => Some(b'7'),
            Eight | Num8 => Some(b'8'),
            Nine | Num9 => Some(b'9'),
            Zero | Num0 => Some(b'0'),

            NumStar => Some(b'*'),
            NumPlus => Some(b'+'),
            NumMinus | Dash => Some(b'-'),
            Equals => Some(b'='),

            BackTick => Some(b'`'),
            Slash => Some(b'/'),
            Backslash => Some(b'\\'),
            LeftBracket => Some(b'['),
            RightBracket => Some(b']'),
            Period | NumDot => Some(b'.'),
            Comma => Some(b','),
            Semicolon => Some(b';'),
            SingleQuote => Some(b'\''),

            Space => Some(b' '),
            Enter => Some(b'\n'),
            Tab => Some(b'\t'),
            Backspace => Some(0x08),

            _ => None,
        }
    }
}
