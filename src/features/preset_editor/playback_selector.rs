use super::PlaybackMode;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

type ChangeHandler = Box<dyn Fn(&usize, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub(crate) struct PlaybackSelector {
    id: ElementId,
    mode: PlaybackMode,
    random_available: bool,
    on_change: Option<ChangeHandler>,
}

impl PlaybackSelector {
    pub(crate) fn new(
        id: impl Into<ElementId>,
        mode: PlaybackMode,
        random_available: bool,
    ) -> Self {
        Self {
            id: id.into(),
            mode,
            random_available,
            on_change: None,
        }
    }

    pub(crate) fn on_change(
        mut self,
        callback: impl Fn(&usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
}

impl RenderOnce for PlaybackSelector {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .flex_shrink_0()
            .p(px(2.))
            .rounded_full()
            .bg(cx.theme().muted)
            .child(
                TabBar::new(self.id)
                    .pill()
                    .small()
                    .selected_index(usize::from(self.mode == PlaybackMode::Random))
                    .child(Tab::new().label("Sequential"))
                    .child(Tab::new().label("Random").disabled(!self.random_available))
                    .when_some(self.on_change, |bar, callback| bar.on_click(callback)),
            )
    }
}
