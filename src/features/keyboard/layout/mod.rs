mod compact;
mod full_size;
mod tkl;

pub(super) use compact::*;
pub(super) use full_size::*;
pub(super) use tkl::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyboardLayout {
    FullSize,
    Tkl,
    Compact,
}

impl KeyboardLayout {
    pub(super) const ALL: [Self; 3] = [Self::FullSize, Self::Tkl, Self::Compact];

    pub(super) fn index(self) -> usize {
        match self {
            Self::FullSize => 0,
            Self::Tkl => 1,
            Self::Compact => 2,
        }
    }

    pub(super) fn width(self) -> f32 {
        match self {
            Self::Compact => 520.0,
            Self::Tkl => 770.0,
            Self::FullSize => 950.0,
        }
    }

    pub(super) fn key_ids(self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        let mut add_row = |row: &[KeySpec]| {
            keys.extend(
                row.iter()
                    .filter(|key| !key.id.starts_with("nav_up_spacer"))
                    .map(|key| key.id),
            );
        };

        match self {
            Self::Compact => {
                add_row(&ROW_1);
                add_row(&ROW_2);
                add_row(&ROW_3);
                add_row(&ROW_4);
                add_row(&ROW_5);
                add_row(&ROW_6);
            }
            Self::Tkl | Self::FullSize => {
                add_row(&TKL_FUNCTION_ROW);
                add_row(TKL_SYSTEM_ROW);
                add_row(TKL_MAIN_ROW_1);
                add_row(TKL_MAIN_ROW_2);
                add_row(TKL_MAIN_ROW_3);
                add_row(TKL_MAIN_ROW_4);
                add_row(TKL_MAIN_ROW_5);
                add_row(TKL_NAV_ROW_1);
                add_row(TKL_NAV_ROW_2);
                add_row(TKL_NAV_ROW_3);
                add_row(TKL_NAV_ROW_4);

                if self == Self::FullSize {
                    add_row(FULL_NUMPAD_ROW_1);
                    add_row(FULL_NUMPAD_ROW_2);
                    add_row(FULL_NUMPAD_ROW_3);
                    add_row(FULL_NUMPAD_ROW_4);
                    add_row(FULL_NUMPAD_ROW_5);
                }
            }
        }

        keys
    }
}

#[derive(Clone, Copy)]
pub(super) struct KeySpec {
    pub(super) id: &'static str,
    pub(super) label: &'static str,
    pub(super) width: f32,
}

impl KeySpec {
    const fn new(id: &'static str, label: &'static str, width: f32) -> Self {
        Self { id, label, width }
    }
}
