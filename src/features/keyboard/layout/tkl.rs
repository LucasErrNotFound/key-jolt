use super::KeySpec;

pub(in crate::features::keyboard) const TKL_MAIN_ROW_1: &[KeySpec] = &[
    KeySpec::new("grave", "`", 38.0),
    KeySpec::new("1", "1", 38.0),
    KeySpec::new("2", "2", 38.0),
    KeySpec::new("3", "3", 38.0),
    KeySpec::new("4", "4", 38.0),
    KeySpec::new("5", "5", 38.0),
    KeySpec::new("6", "6", 38.0),
    KeySpec::new("7", "7", 38.0),
    KeySpec::new("8", "8", 38.0),
    KeySpec::new("9", "9", 38.0),
    KeySpec::new("0", "0", 38.0),
    KeySpec::new("minus", "-", 38.0),
    KeySpec::new("equal", "=", 38.0),
    KeySpec::new("backspace", "Backspace", 72.0),
];

pub(in crate::features::keyboard) const TKL_MAIN_ROW_2: &[KeySpec] = &[
    KeySpec::new("tab", "Tab", 58.0),
    KeySpec::new("q", "Q", 38.0),
    KeySpec::new("w", "W", 38.0),
    KeySpec::new("e", "E", 38.0),
    KeySpec::new("r", "R", 38.0),
    KeySpec::new("t", "T", 38.0),
    KeySpec::new("y", "Y", 38.0),
    KeySpec::new("u", "U", 38.0),
    KeySpec::new("i", "I", 38.0),
    KeySpec::new("o", "O", 38.0),
    KeySpec::new("p", "P", 38.0),
    KeySpec::new("left_bracket", "[", 38.0),
    KeySpec::new("right_bracket", "]", 38.0),
    KeySpec::new("backslash", "\\", 58.0),
];

pub(in crate::features::keyboard) const TKL_MAIN_ROW_3: &[KeySpec] = &[
    KeySpec::new("caps_lock", "Caps Lock", 72.0),
    KeySpec::new("a", "A", 38.0),
    KeySpec::new("s", "S", 38.0),
    KeySpec::new("d", "D", 38.0),
    KeySpec::new("f", "F", 38.0),
    KeySpec::new("g", "G", 38.0),
    KeySpec::new("h", "H", 38.0),
    KeySpec::new("j", "J", 38.0),
    KeySpec::new("k", "K", 38.0),
    KeySpec::new("l", "L", 38.0),
    KeySpec::new("semicolon", ";", 38.0),
    KeySpec::new("quote", "'", 38.0),
    KeySpec::new("enter", "Enter", 78.0),
];

pub(in crate::features::keyboard) const TKL_MAIN_ROW_4: &[KeySpec] = &[
    KeySpec::new("left_shift", "Shift", 92.0),
    KeySpec::new("z", "Z", 38.0),
    KeySpec::new("x", "X", 38.0),
    KeySpec::new("c", "C", 38.0),
    KeySpec::new("v", "V", 38.0),
    KeySpec::new("b", "B", 38.0),
    KeySpec::new("n", "N", 38.0),
    KeySpec::new("m", "M", 38.0),
    KeySpec::new("comma", ",", 38.0),
    KeySpec::new("period", ".", 38.0),
    KeySpec::new("slash", "/", 38.0),
    KeySpec::new("right_shift", "Shift", 96.0),
];

pub(in crate::features::keyboard) const TKL_MAIN_ROW_5: &[KeySpec] = &[
    KeySpec::new("left_ctrl", "Ctrl", 58.0),
    KeySpec::new("left_meta", "Win", 58.0),
    KeySpec::new("left_alt", "Alt", 58.0),
    KeySpec::new("space", "", 190.0),
    KeySpec::new("right_alt", "Alt", 58.0),
    KeySpec::new("right_meta", "Win", 58.0),
    KeySpec::new("menu", "Menu", 58.0),
    KeySpec::new("right_ctrl", "Ctrl", 58.0),
];

pub(in crate::features::keyboard) const TKL_NAV_ROW_1: &[KeySpec] = &[
    KeySpec::new("insert", "Ins", 38.0),
    KeySpec::new("home", "Home", 38.0),
    KeySpec::new("page_up", "PgUp", 38.0),
];

pub(in crate::features::keyboard) const TKL_NAV_ROW_2: &[KeySpec] = &[
    KeySpec::new("delete", "Del", 38.0),
    KeySpec::new("end", "End", 38.0),
    KeySpec::new("page_down", "PgDn", 38.0),
];

pub(in crate::features::keyboard) const TKL_NAV_ROW_3: &[KeySpec] = &[
    KeySpec::new("nav_up_spacer", "", 38.0),
    KeySpec::new("arrow_up", "↑", 38.0),
    KeySpec::new("nav_up_spacer_2", "", 38.0),
];

pub(in crate::features::keyboard) const TKL_NAV_ROW_4: &[KeySpec] = &[
    KeySpec::new("arrow_left", "←", 38.0),
    KeySpec::new("arrow_down", "↓", 38.0),
    KeySpec::new("arrow_right", "→", 38.0),
];

pub(in crate::features::keyboard) const TKL_SYSTEM_ROW: &[KeySpec] =
    &[KeySpec::new("print_screen", "PrtSc", 122.0)];

pub(in crate::features::keyboard) const TKL_FUNCTION_ROW: [KeySpec; 13] = [
    KeySpec::new("esc", "Esc", 44.0),
    KeySpec::new("f1", "F1", 44.0),
    KeySpec::new("f2", "F2", 44.0),
    KeySpec::new("f3", "F3", 44.0),
    KeySpec::new("f4", "F4", 44.0),
    KeySpec::new("f5", "F5", 44.0),
    KeySpec::new("f6", "F6", 44.0),
    KeySpec::new("f7", "F7", 44.0),
    KeySpec::new("f8", "F8", 44.0),
    KeySpec::new("f9", "F9", 44.0),
    KeySpec::new("f10", "F10", 44.0),
    KeySpec::new("f11", "F11", 44.0),
    KeySpec::new("f12", "F12", 44.0),
];
