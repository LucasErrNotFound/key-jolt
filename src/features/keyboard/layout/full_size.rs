use super::KeySpec;

pub(in crate::features::keyboard) const FULL_NUMPAD_ROW_1: &[KeySpec] = &[
    KeySpec::new("num_lock", "Num", 38.0),
    KeySpec::new("num_divide", "/", 38.0),
    KeySpec::new("num_multiply", "*", 38.0),
    KeySpec::new("num_minus", "-", 38.0),
];

pub(in crate::features::keyboard) const FULL_NUMPAD_ROW_2: &[KeySpec] = &[
    KeySpec::new("num_7", "7", 38.0),
    KeySpec::new("num_8", "8", 38.0),
    KeySpec::new("num_9", "9", 38.0),
    KeySpec::new("num_plus_1", "+", 38.0),
];

pub(in crate::features::keyboard) const FULL_NUMPAD_ROW_3: &[KeySpec] = &[
    KeySpec::new("num_4", "4", 38.0),
    KeySpec::new("num_5", "5", 38.0),
    KeySpec::new("num_6", "6", 38.0),
    KeySpec::new("num_plus_2", "+", 38.0),
];

pub(in crate::features::keyboard) const FULL_NUMPAD_ROW_4: &[KeySpec] = &[
    KeySpec::new("num_1", "1", 38.0),
    KeySpec::new("num_2", "2", 38.0),
    KeySpec::new("num_3", "3", 38.0),
    KeySpec::new("num_enter_1", "Enter", 38.0),
];

pub(in crate::features::keyboard) const FULL_NUMPAD_ROW_5: &[KeySpec] = &[
    KeySpec::new("num_0", "0", 78.0),
    KeySpec::new("num_decimal", ".", 38.0),
    KeySpec::new("num_enter_2", "Enter", 38.0),
];
