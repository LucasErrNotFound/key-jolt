use crate::presets::PresetData;
use gpui_kit::component::searchable_list::{SearchableListItem, SearchableVec};

use gpui_kit::component::*;
use gpui_kit::*;

#[derive(Clone)]
pub(crate) struct PresetChoice {
    pub id: SharedString,
    pub name: SharedString,
    pub summary: SharedString,
    pub data: Option<PresetData>,
    pub is_builtin: bool,
}

impl PresetChoice {
    pub(crate) fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        summary: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            summary: summary.into(),
            data: None,
            is_builtin: true,
        }
    }
}

impl SearchableListItem for PresetChoice {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }

    fn render(&self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex().child(div().child(self.name.clone())).child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(self.summary.clone()),
        )
    }
}

pub(super) type PresetItems = SearchableVec<PresetChoice>;
