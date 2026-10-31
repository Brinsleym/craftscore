//! The browser platform: canvas start-up, the file picker, dropped files and local storage.

use std::cell::RefCell;
use std::rc::Rc;

use eframe::wasm_bindgen::JsCast;
use eframe::wasm_bindgen::closure::Closure;
use scorecraft_ui_egui::{App, Platform, Settings};

/// Files read by the browser, waiting for the next frame: name and bytes.
type Inbox = Rc<RefCell<Vec<(String, Vec<u8>)>>>;

const SETTINGS_KEY: &str = "scorecraft.settings";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Reads `file` and leaves it in `inbox`.
fn read_into(file: web_sys::File, inbox: Inbox, ctx: eframe::egui::Context) {
    wasm_bindgen_futures::spawn_local(async move {
        let name = file.name();
        let Ok(buf) = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await else {
            return;
        };
        let bytes = js_sys::Uint8Array::new(&buf).to_vec();
        inbox.borrow_mut().push((name, bytes));
        ctx.request_repaint();
    });
}

struct Web {
    inbox: Inbox,
    ctx: Option<eframe::egui::Context>,
}

impl Platform for Web {
    /// Opens the browser's file picker; the file arrives in a later frame.
    fn pick_score(&mut self) -> Option<String> {
        let document = web_sys::window()?.document()?;
        let input: web_sys::HtmlInputElement =
            document.create_element("input").ok()?.dyn_into().ok()?;
        input.set_type("file");
        input.set_accept(".pdf,application/pdf");
        let (inbox, ctx) = (self.inbox.clone(), self.ctx.clone()?);
        let picked = input.clone();
        let on_change = Closure::<dyn FnMut()>::new(move || {
            if let Some(file) = picked.files().and_then(|f| f.get(0)) {
                read_into(file, inbox.clone(), ctx.clone());
            }
        });
        input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        // The closure lives as long as the page: one per pick.
        on_change.forget();
        input.click();
        None
    }

    fn set_keep_awake(&mut self, on: bool) -> Result<(), String> {
        if on {
            Err("not in the browser yet".into())
        } else {
            Ok(())
        }
    }

    fn save_settings(&mut self, settings: &Settings) {
        if let Some(s) = storage() {
            let _ = s.set_item(SETTINGS_KEY, &settings.to_json_string());
        }
    }
}

struct Shell {
    app: App,
    inbox: Inbox,
}

impl eframe::App for Shell {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Dropped files: the browser reads them asynchronously.
        let dropped: Vec<_> = ctx.input(|i| i.raw.dropped_files.clone());
        for f in dropped {
            let (inbox, ctx2) = (self.inbox.clone(), ctx.clone());
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(bytes) = f.bytes_async().await {
                    let name = f
                        .path()
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "score.pdf".into());
                    inbox.borrow_mut().push((name, bytes));
                    ctx2.request_repaint();
                }
            });
        }
        let ready: Vec<_> = self.inbox.borrow_mut().drain(..).collect();
        for (name, bytes) in ready {
            if let Err(e) = self.app.open_bytes(&name, bytes) {
                self.app.state.message = Some(e.to_string());
            }
        }
        self.app.ui(ui);
    }
}

pub fn start() {
    let settings = storage()
        .and_then(|s| s.get_item(SETTINGS_KEY).ok().flatten())
        .map(|t| Settings::parse(&t))
        .unwrap_or_default();
    wasm_bindgen_futures::spawn_local(async move {
        let Some(canvas) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("scorecraft_canvas"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        else {
            return;
        };
        let inbox: Inbox = Rc::default();
        let started = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |cc| {
                    let platform = Web {
                        inbox: inbox.clone(),
                        ctx: Some(cc.egui_ctx.clone()),
                    };
                    let app = App::new(Box::new(platform), settings);
                    Ok(Box::new(Shell { app, inbox }))
                }),
            )
            .await;
        if let Err(e) = started {
            web_sys::console::error_1(&e);
        }
    });
}
