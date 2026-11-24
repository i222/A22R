use gpui_kit::{
    div, px, App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window,
};

use crate::{ActionIcon, IconName, Size};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Reusable native file actions for import/export toolbars.
///
/// The two child controls keep stable element IDs so application and native
/// interaction tests can target the same controls without depending on layout.
#[derive(IntoElement)]
pub struct ImportExportActions {
    id: ElementId,
    import_label: SharedString,
    export_label: SharedString,
    disabled: bool,
    on_import: Option<ClickHandler>,
    on_export: Option<ClickHandler>,
}

impl ImportExportActions {
    pub fn new() -> Self {
        Self {
            id: "import-export-actions".into(),
            import_label: "Import".into(),
            export_label: "Export".into(),
            disabled: false,
            on_import: None,
            on_export: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn import_label(mut self, label: impl Into<SharedString>) -> Self {
        self.import_label = label.into();
        self
    }

    pub fn export_label(mut self, label: impl Into<SharedString>) -> Self {
        self.export_label = label.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_import(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_import = Some(Box::new(handler));
        self
    }

    pub fn on_export(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_export = Some(Box::new(handler));
        self
    }
}

impl Default for ImportExportActions {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ImportExportActions {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut import = ActionIcon::new("import-items", IconName::Upload)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(self.import_label)
            .disabled(self.disabled);
        if let Some(handler) = self.on_import {
            import = import.on_click(handler);
        }

        let mut export = ActionIcon::new("export-items", IconName::Download)
            .size(Size::Sm)
            .pixel_size(24.0)
            .label(self.export_label)
            .disabled(self.disabled);
        if let Some(handler) = self.on_export {
            export = export.on_click(handler);
        }

        let import = div().debug_selector(|| "import-items".into()).child(import);
        let export = div().debug_selector(|| "export-items".into()).child(export);

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(import)
            .child(export)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{div, point, prelude::*, Context, Modifiers, Render, TestAppContext, Window};
    use gpui_kit::component::init;

    use super::ImportExportActions;

    struct Harness {
        events: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Render for Harness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let imports = self.events.clone();
            let exports = self.events.clone();
            div().child(
                ImportExportActions::new()
                    .import_label("Import items")
                    .export_label("Export items")
                    .on_import(move |_, _, _| imports.borrow_mut().push("import"))
                    .on_export(move |_, _, _| exports.borrow_mut().push("export")),
            )
        }
    }

    #[gpui::test]
    fn import_and_export_actions_emit_distinct_events(cx: &mut TestAppContext) {
        cx.update(init);
        let events = Rc::new(RefCell::new(Vec::new()));
        let (view, visual) = cx.add_window_view({
            let events = events.clone();
            move |_, _| Harness { events }
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));

        let import = visual
            .debug_bounds("import-items")
            .expect("import action should be registered");
        visual.simulate_click(
            point(import.center().x, import.center().y),
            Modifiers::default(),
        );
        let export = visual
            .debug_bounds("export-items")
            .expect("export action should be registered");
        visual.simulate_click(
            point(export.center().x, export.center().y),
            Modifiers::default(),
        );

        view.update(cx, |_view, _cx| {});
        assert_eq!(&*events.borrow(), &["import", "export"]);
    }
}
