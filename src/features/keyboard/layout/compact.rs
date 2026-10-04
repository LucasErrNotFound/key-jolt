use super::KeySpec;

pub(in crate::features::keyboard) const ROW_1: [KeySpec; 14] = [
    KeySpec::new("esc", "Esc", 32.0),
    KeySpec::new("f1", "F1", 32.0),
    KeySpec::new("f2", "F2", 32.0),
    KeySpec::new("f3", "F3", 32.0),
    KeySpec::new("f4", "F4", 32.0),
    KeySpec::new("f5", "F5", 32.0),
    KeySpec::new("f6", "F6", 32.0),
    KeySpec::new("f7", "F7", 32.0),
    KeySpec::new("f8", "F8", 32.0),
    KeySpec::new("f9", "F9", 32.0),
    KeySpec::new("f10", "F10", 32.0),
    KeySpec::new("f11", "F11", 32.0),
    KeySpec::new("f12", "F12", 32.0),
    KeySpec::new("print_screen", "PrtSc", 32.0),
];

pub(in crate::features::keyboard) const ROW_2: [KeySpec; 14] = [
    KeySpec::new("grave", "~", 32.0),
    KeySpec::new("1", "1", 32.0),
    KeySpec::new("2", "2", 32.0),
    KeySpec::new("3", "3", 32.0),
    KeySpec::new("4", "4", 32.0),
    KeySpec::new("5", "5", 32.0),
    KeySpec::new("6", "6", 32.0),
    KeySpec::new("7", "7", 32.0),
    KeySpec::new("8", "8", 32.0),
    KeySpec::new("9", "9", 32.0),
    KeySpec::new("0", "0", 32.0),
    KeySpec::new("minus", "-", 32.0),
    KeySpec::new("equal", "=", 32.0),
    KeySpec::new("backspace", "Backspace", 91.0),
];

pub(in crate::features::keyboard) const ROW_3: [KeySpec; 14] = [
    KeySpec::new("tab", "Tab", 65.0),
    KeySpec::new("q", "Q", 30.0),
    KeySpec::new("w", "W", 30.0),
    KeySpec::new("e", "E", 30.0),
    KeySpec::new("r", "R", 30.0),
    KeySpec::new("t", "T", 30.0),
    KeySpec::new("y", "Y", 30.0),
    KeySpec::new("u", "U", 30.0),
    KeySpec::new("i", "I", 30.0),
    KeySpec::new("o", "O", 30.0),
    KeySpec::new("p", "P", 30.0),
    KeySpec::new("left_bracket", "[", 30.0),
    KeySpec::new("right_bracket", "]", 30.0),
    KeySpec::new("backslash", "| \\", 81.0),
];

pub(in crate::features::keyboard) const ROW_4: [KeySpec; 12] = [
    KeySpec::new("caps_lock", "Caps Lock", 85.0),
    KeySpec::new("a", "A", 30.0),
    KeySpec::new("s", "S", 30.0),
    KeySpec::new("d", "D", 30.0),
    KeySpec::new("f", "F", 30.0),
    KeySpec::new("g", "G", 30.0),
    KeySpec::new("h", "H", 30.0),
    KeySpec::new("j", "J", 30.0),
    KeySpec::new("k", "K", 30.0),
    KeySpec::new("l", "L", 30.0),
    KeySpec::new("semicolon", ";", 30.0),
    KeySpec::new("enter", "Enter", 130.0),
];

pub(in crate::features::keyboard) const ROW_5: [KeySpec; 12] = [
    KeySpec::new("shift_left", "Shift", 95.0),
    KeySpec::new("z", "Z", 30.0),
    KeySpec::new("x", "X", 30.0),
    KeySpec::new("c", "C", 30.0),
    KeySpec::new("v", "V", 30.0),
    KeySpec::new("b", "B", 30.0),
    KeySpec::new("n", "N", 30.0),
    KeySpec::new("m", "M", 30.0),
    KeySpec::new("comma", ",", 30.0),
    KeySpec::new("period", ".", 30.0),
    KeySpec::new("slash", "/", 30.0),
    KeySpec::new("shift_right", "Shift", 120.0),
];

pub(in crate::features::keyboard) const ROW_6: [KeySpec; 8] = [
    KeySpec::new("ctrl_left", "Ctrl", 50.0),
    KeySpec::new("win_left", "Win", 50.0),
    KeySpec::new("alt_left", "Alt", 50.0),
    KeySpec::new("space", "", 135.0),
    KeySpec::new("alt_right", "Alt", 50.0),
    KeySpec::new("win_right", "Win", 50.0),
    KeySpec::new("menu", "Menu", 50.0),
    KeySpec::new("ctrl_right", "Ctrl", 50.0),
];
