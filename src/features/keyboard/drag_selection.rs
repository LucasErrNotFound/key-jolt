use super::KeyboardEditorView;
use super::selection_math::marquee_selection;
use gpui_kit::component::ActiveTheme;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
pub(super) struct CanvasGeometry {
    pub(super) viewport: Option<Bounds<Pixels>>,
    pub(super) origin: Point<Pixels>,
    pub(super) keys: HashMap<&'static str, Bounds<Pixels>>,
}

pub(super) type SharedCanvasGeometry = Rc<RefCell<CanvasGeometry>>;

pub(super) struct SelectionDrag {
    pub(super) origin: Point<Pixels>,
    pub(super) current: Point<Pixels>,
    baseline: [Vec<&'static str>; 3],
    additive: bool,
    pub(super) moved: bool,
}

impl KeyboardEditorView {
    pub(super) fn begin_selection_drag(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left {
            return;
        }
        self.suppress_key_click = false;
        let geometry = self.canvas_geometry.borrow();
        if !geometry
            .viewport
            .is_some_and(|bounds| bounds.contains(&event.position))
        {
            self.selection_drag = None;
            return;
        }
        self.canvas_focus.focus(window, cx);
        self.selection_drag = Some(SelectionDrag {
            origin: event.position,
            current: event.position,
            baseline: self.selected_keys.clone(),
            additive: event.modifiers.control,
            moved: false,
        });
        cx.notify();
    }

    pub(super) fn update_selection_drag(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let Some(drag) = self.selection_drag.as_mut() else {
            return;
        };
        if !event.dragging() {
            self.finish_selection_drag(cx);
            return;
        }
        let distance =
            (event.position.x - drag.origin.x).abs() + (event.position.y - drag.origin.y).abs();
        if !drag.moved && distance < px(5.) {
            return;
        }
        drag.moved = true;
        drag.current = event.position;
        self.suppress_key_click = true;
        let geometry = self.canvas_geometry.borrow();
        let Some(viewport) = geometry.viewport else {
            return;
        };
        let marquee = selection_bounds(drag.origin, drag.current).intersect(&viewport);
        let hits = self
            .keyboard_layout
            .key_ids()
            .into_iter()
            .filter(|key| {
                geometry.keys.get(key).is_some_and(|bounds| {
                    bounds.intersects(&viewport) && bounds.intersects(&marquee)
                })
            })
            .collect::<Vec<_>>();
        let selected = marquee_selection(
            &drag.baseline[self.keyboard_layout.index()],
            &hits,
            drag.additive,
        );
        drop(geometry);
        self.replace_editing_selection(&selected, cx);
    }

    pub(super) fn finish_selection_drag(&mut self, cx: &mut Context<Self>) {
        if self.selection_drag.take().is_some() {
            cx.notify();
        }
    }

    pub(super) fn cancel_selection_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(drag) = self.selection_drag.take() {
            self.selected_keys = drag.baseline;
            self.suppress_key_click = drag.moved;
            cx.notify();
        }
    }

    pub(super) fn render_selection_marquee(&self, cx: &App) -> impl IntoElement {
        let geometry = self.canvas_geometry.borrow();
        div().absolute().inset_0().when_some(
            self.selection_drag
                .as_ref()
                .filter(|drag| drag.moved)
                .zip(geometry.viewport),
            |overlay, (drag, viewport)| {
                let bounds = selection_bounds(drag.origin, drag.current).intersect(&viewport);
                overlay.child(
                    div()
                        .absolute()
                        .left(bounds.origin.x - geometry.origin.x)
                        .top(bounds.origin.y - geometry.origin.y)
                        .w(bounds.size.width)
                        .h(bounds.size.height)
                        .border_1()
                        .border_color(cx.theme().primary)
                        .bg(cx.theme().primary.opacity(0.08)),
                )
            },
        )
    }
}

fn selection_bounds(start: Point<Pixels>, end: Point<Pixels>) -> Bounds<Pixels> {
    Bounds::new(
        point(start.x.min(end.x), start.y.min(end.y)),
        size((start.x - end.x).abs(), (start.y - end.y).abs()),
    )
}
