#![cfg_attr(target_family = "wasm", no_main)]

mod text_input;

use anyhow::{Result, ensure};
use futures::AsyncReadExt;
use gpui::{
    App, Bounds, Context, Entity, FocusHandle, Focusable, KeyBinding, SharedString, Task, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use std::borrow::Cow;
use text_input::{
    Backspace, Copy, Cut, Delete, End, Home, Left, Paste, Right, SelectAll, SelectLeft,
    SelectRight, TextInput,
};

/// The browser has no system fonts to fall back to, so Hangul needs a font of its own. It's
/// fetched rather than embedded to keep it out of the wasm binary, where it'd be downloaded again
/// with every build.
const KOREAN_FONT_PATH: &str = "/fonts/NanumGothicCoding-Regular.ttf";
const CODE_FONT: &str = "Lilex";

const SAMPLE_CODE: &[&str] = &[
    "// 브라우저에서 돌아가는 Luna 렌더링 테스트",
    "fn greet(name: &str) -> String {",
    "    // 한글 주석과 문자열도 같은 폰트 경로로 그려집니다",
    "    format!(\"안녕하세요, {name}님!\")",
    "}",
    "",
    "fn main() {",
    "    println!(\"{}\", greet(\"루나\"));",
    "}",
];

enum FontState {
    Loading,
    Loaded,
    Failed(SharedString),
}

struct LunaWebPoc {
    text_input: Entity<TextInput>,
    font_state: FontState,
    _font_download: Task<()>,
    focus_handle: FocusHandle,
}

impl LunaWebPoc {
    fn new(cx: &mut Context<Self>) -> Self {
        let text_input = cx.new(|cx| TextInput::new("여기에 한글을 입력해 보세요", cx));
        cx.observe(&text_input, |_, _, cx| cx.notify()).detach();
        Self {
            text_input,
            font_state: FontState::Loading,
            _font_download: Self::load_korean_font(cx),
            focus_handle: cx.focus_handle(),
        }
    }

    fn load_korean_font(cx: &mut Context<Self>) -> Task<()> {
        let http_client = cx.http_client();
        cx.spawn(async move |this, cx| {
            let download: Result<Vec<u8>> = async {
                let origin = web_sys::window()
                    .and_then(|window| window.location().origin().ok())
                    .unwrap_or_default();
                let mut response = http_client
                    .get(&format!("{origin}{KOREAN_FONT_PATH}"), Default::default(), true)
                    .await?;
                ensure!(
                    response.status().is_success(),
                    "HTTP {}",
                    response.status()
                );
                let mut bytes = Vec::new();
                response.body_mut().read_to_end(&mut bytes).await?;
                Ok(bytes)
            }
            .await;
            this.update(cx, |this, cx| {
                let added =
                    download.and_then(|bytes| cx.text_system().add_fonts(vec![Cow::Owned(bytes)]));
                this.font_state = match added {
                    Ok(()) => FontState::Loaded,
                    Err(error) => FontState::Failed(format!("{error:#}").into()),
                };
                // Text shaped before the font arrived was shaped with missing glyphs.
                cx.refresh_windows();
                cx.notify();
            })
            .ok();
        })
    }

    fn font_status(&self) -> (SharedString, u32) {
        match &self.font_state {
            FontState::Loading => ("Korean font: loading…".into(), 0xe5c07b),
            FontState::Loaded => ("Korean font: loaded".into(), 0x98c379),
            FontState::Failed(error) => (format!("Korean font: {error}").into(), 0xe06c75),
        }
    }
}

impl Focusable for LunaWebPoc {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LunaWebPoc {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (font_status, font_status_color) = self.font_status();
        let typed = self.text_input.read(cx).content().clone();
        div()
            .track_focus(&self.focus_handle(cx))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e2127))
            .text_color(rgb(0xabb2bf))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x21252b))
                    .border_b_1()
                    .border_color(rgb(0x181a1f))
                    .child(div().text_color(rgb(0xdcdfe4)).child("Luna Web PoC"))
                    .child(div().text_color(rgb(font_status_color)).child(font_status)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .font_family(CODE_FONT)
                    .text_size(px(15.))
                    .children(SAMPLE_CODE.iter().enumerate().map(|(index, line)| {
                        div()
                            .flex()
                            .flex_row()
                            .gap_4()
                            .child(
                                div()
                                    .w(px(24.))
                                    .text_color(rgb(0x495162))
                                    .child(format!("{}", index + 1)),
                            )
                            .child(div().child(*line))
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .border_t_1()
                    .border_color(rgb(0x181a1f))
                    .child("Input test (Korean IME)")
                    .child(div().text_color(rgb(0x1e2127)).child(self.text_input.clone()))
                    .child(format!("Typed: {typed}")),
            )
    }
}

fn load_embedded_fonts(cx: &App) -> Result<()> {
    cx.text_system().add_fonts(vec![
        Cow::Borrowed(include_bytes!("../../assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../../assets/fonts/lilex/Lilex-Regular.ttf").as_slice()),
    ])
}

fn run() {
    application().run(|cx: &mut App| {
        if let Err(error) = load_embedded_fonts(cx) {
            log::error!("failed to load fonts: {error:#}");
            return;
        }
        let mut bindings = Vec::new();
        for modifier in ["cmd", "ctrl"] {
            bindings.extend([
                KeyBinding::new(&format!("{modifier}-a"), SelectAll, None),
                KeyBinding::new(&format!("{modifier}-v"), Paste, None),
                KeyBinding::new(&format!("{modifier}-c"), Copy, None),
                KeyBinding::new(&format!("{modifier}-x"), Cut, None),
            ]);
        }
        bindings.extend([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
        ]);
        cx.bind_keys(bindings);

        let bounds = Bounds::centered(None, size(px(900.), px(600.)), cx);
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(LunaWebPoc::new),
        );
        let Ok(window) = window else {
            log::error!("failed to open the window");
            return;
        };
        window
            .update(cx, |view, window, cx| {
                window.focus(&view.text_input.focus_handle(cx), cx);
                cx.activate(true);
            })
            .ok();
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run();
}
