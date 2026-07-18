use std::rc::Rc;

use gpui::{
    App, Bounds, ContentMask, Context, Element, ElementId, Entity, FocusHandle, Focusable,
    GlobalElementId, Hitbox, InteractiveElement, IntoElement, LayoutId, MouseDownEvent,
    ParentElement as _, Pixels, Render, Size, Style, Styled as _, Window, div,
};
use wry::{
    Rect,
    dpi::{LogicalPosition, LogicalSize, Position, Size as DpiSize},
};

pub(in crate::native) struct BrowserPane {
    focus_handle: FocusHandle,
    webview: Rc<wry::WebView>,
    visible: bool,
}

impl BrowserPane {
    pub(in crate::native) fn new(webview: wry::WebView, cx: &mut Context<Self>) -> Self {
        let _ = webview.set_bounds(Rect::default());
        Self {
            focus_handle: cx.focus_handle(),
            webview: Rc::new(webview),
            visible: true,
        }
    }

    pub(in crate::native) fn load_url(&self, url: &str) -> Result<(), String> {
        self.webview
            .load_url(url)
            .map_err(|error| error.to_string())
    }

    pub(in crate::native) fn back(&self) -> Result<(), String> {
        self.webview
            .evaluate_script("history.back();")
            .map_err(|error| error.to_string())
    }

    pub(in crate::native) fn hide(&mut self) {
        let _ = self.webview.focus_parent();
        let _ = self.webview.set_visible(false);
        self.visible = false;
    }
}

impl Drop for BrowserPane {
    fn drop(&mut self) {
        self.hide();
    }
}

impl Focusable for BrowserPane {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BrowserPane {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .child(BrowserElement {
                pane: cx.entity(),
                webview: self.webview.clone(),
            })
    }
}

struct BrowserElement {
    pane: Entity<BrowserPane>,
    webview: Rc<wry::WebView>,
}

impl IntoElement for BrowserElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for BrowserElement {
    type RequestLayoutState = ();
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            flex_shrink: 1.0,
            size: Size::full(),
            ..Style::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if !self.pane.read(cx).visible {
            return None;
        }
        let _ = self.webview.set_bounds(Rect {
            position: Position::Logical(LogicalPosition::new(
                bounds.origin.x.into(),
                bounds.origin.y.into(),
            )),
            size: DpiSize::Logical(LogicalSize::new(
                bounds.size.width.as_f32().into(),
                bounds.size.height.as_f32().into(),
            )),
        });
        Some(window.insert_hitbox(bounds, gpui::HitboxBehavior::Normal))
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        _: &mut App,
    ) {
        let bounds = hitbox.as_ref().map_or(bounds, |hitbox| hitbox.bounds);
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            let webview = self.webview.clone();
            window.on_mouse_event(move |event: &MouseDownEvent, _, _, _| {
                if !bounds.contains(&event.position) {
                    let _ = webview.focus_parent();
                }
            });
        });
    }
}
