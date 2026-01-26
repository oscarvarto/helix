use std::sync::{Arc, Mutex};

use arc_swap::access::Map;
use helix_core::Position;
use helix_view::{
    graphics::{CursorKind, Rect},
    Editor,
};
use steel::{
    rvals::{AsRefMutSteelVal, AsRefSteelVal, Custom},
    SteelVal,
};

use crate::{
    commands::{
        engine::steel::{BoxDynComponent, HelixConfiguration},
        Context,
    },
    compositor::Component,
    config::Config,
    job::Jobs,
    keymap::Keymaps,
    ui::EditorView,
};

use super::WrappedDynComponent;

pub struct SteelEditor {
    editor: Option<Editor>,
    editor_view: EditorView,
}

unsafe impl Send for EditorView {}
unsafe impl Sync for EditorView {}

impl std::fmt::Debug for SteelEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#<SteelEditor>")
    }
}

impl SteelEditor {
    pub fn get_text(&self) -> Option<SteelVal> {
        if let Some(editor) = &self.editor {
            editor
                .documents
                .first_key_value()
                .map(|x| SteelVal::StringV(x.1.text().to_string().into()))
        } else {
            None
        }
    }
}

impl Custom for SteelEditor {}

struct SteelEditorComponent {
    component: SteelVal,
    area: Rect,
}

impl Component for SteelEditorComponent {
    fn render(
        &mut self,
        _area: Rect,
        frame: &mut tui::buffer::Buffer,
        _ctx: &mut crate::compositor::Context,
    ) {
        if let Ok(mut component) = SteelEditor::as_mut_ref(&mut self.component) {
            let mut editor = component.editor.take().unwrap();
            let area = self.area; // .clip_bottom(1);
                                  // editor.resize(area);

            let mut ctx = crate::compositor::Context {
                editor: &mut editor,
                scroll: None,
                jobs: &mut Jobs::new(),
            };

            component.editor_view.render(area, frame, &mut ctx);
            component.editor = Some(editor);
        }
    }

    fn handle_event(
        &mut self,
        event: &helix_view::input::Event,
        _ctx: &mut crate::compositor::Context,
    ) -> crate::compositor::EventResult {
        if let Ok(mut component) = SteelEditor::as_mut_ref(&mut self.component) {
            let mut editor = component.editor.take().unwrap();

            let mut ctx = crate::compositor::Context {
                editor: &mut editor,
                scroll: None,
                jobs: &mut Jobs::new(),
            };
            let res = component.editor_view.handle_event(event, &mut ctx);

            component.editor = Some(editor);

            res
        } else {
            crate::compositor::EventResult::Ignored(None)
        }
    }

    fn cursor(&mut self, area: Rect, _: &mut Editor) -> (Option<Position>, CursorKind) {
        if let Ok(mut component) = SteelEditor::as_mut_ref(&self.component) {
            let mut editor = component.editor.take().unwrap();

            let res = component.editor_view.cursor(area, &mut editor);
            component.editor = Some(editor);

            res
        } else {
            (None, CursorKind::Block)
        }
    }
}

impl Custom for SteelEditorComponent {}

impl SteelEditorComponent {}

impl SteelEditor {
    pub fn new(value: SteelVal, area: Rect) -> WrappedDynComponent {
        if SteelEditor::as_ref(&value).is_ok() {
            WrappedDynComponent {
                inner: Some(Box::new(SteelEditorComponent {
                    component: value,
                    area,
                })),
            }
        } else {
            panic!()
        }
    }
}

pub fn make_editor(
    ctx: &mut Context,
    config: SteelVal,
    area: Rect,
) -> steel::rvals::Result<SteelEditor> {
    let config = HelixConfiguration::as_ref(&config)?;

    let handlers = crate::handlers::setup_fake(config.configuration.clone());

    let mut editor = Editor::new(
        area,
        ctx.editor.theme_loader.clone(),
        ctx.editor.syn_loader.clone(),
        ctx.editor.config.clone(),
        handlers,
    );

    editor.new_file(helix_view::editor::Action::VerticalSplit);

    let keys = Box::new(Map::new(
        Arc::clone(&config.configuration),
        |config: &Config| &config.keys,
    ));

    let editor_view = EditorView::new(Keymaps::new(keys));

    Ok(SteelEditor {
        editor: Some(editor),
        editor_view,
    })
}
