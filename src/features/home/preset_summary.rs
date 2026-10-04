use gpui_kit::assets::IconName;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants};
use gpui_kit::component::*;
use gpui_kit::*;

#[derive(IntoElement)]
pub(super) struct PresetSummary {
    icon: IconName,
    title: SharedString,
    subtitle: SharedString,
    action: Button,
}

impl PresetSummary {
    pub(super) fn new(
        icon: IconName,
        title: impl Into<SharedString>,
        subtitle: impl Into<SharedString>,
        action: Button,
    ) -> Self {
        Self {
            icon,
            title: title.into(),
            subtitle: subtitle.into(),
            action,
        }
    }
}

impl RenderOnce for PresetSummary {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let action_variant = ButtonCustomVariant::new(cx)
            .color(cx.theme().muted_foreground.opacity(0.25))
            .foreground(cx.theme().primary)
            .hover(cx.theme().muted.opacity(0.3))
            .active(cx.theme().muted.opacity(0.5));

        GroupBox::new()
            .fill()
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.3))
            .rounded(cx.theme().radius)
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(
                        h_flex()
                            .flex_shrink_0()
                            .w(rems(2.5))
                            .h(rems(2.5))
                            .items_center()
                            .justify_center()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().primary.opacity(0.15))
                            .child(
                                Icon::new(self.icon)
                                    .with_size(cx.theme().font_size * 1.875)
                                    .text_color(cx.theme().primary),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().font_semibold().child(self.title))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(self.subtitle),
                            ),
                    )
                    .child(self.action.border_1().custom(action_variant)),
            )
    }
}
