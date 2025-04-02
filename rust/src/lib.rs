mod godot_window;
mod protocols;

use godot::global::MouseButtonMask;
use godot::init::*;
use godot::prelude::*;
use godot::classes::{Control, IControl, InputEventMouseButton, InputEventMouseMotion, InputEventKey};
use godot::global::{Key, MouseButton};
use wry::{WebViewBuilder, Rect, WebViewAttributes};
use wry::dpi::{PhysicalPosition, PhysicalSize};
use wry::http::Request;
use crate::godot_window::GodotWindow;
use crate::protocols::get_res_response;
use serde_json;
use std::sync::Mutex;
use lazy_static::lazy_static;

struct GodotWRY;

#[gdextension]
unsafe impl ExtensionLibrary for GodotWRY {}

#[derive(GodotClass)]
#[class(base=Control)]
struct WebView {
    base: Base<Control>,
    webview: Option<wry::WebView>,
    previous_screen_position: Vector2,
    #[export]
    full_window_size: bool,
    #[export]
    url: GString,
    #[export]
    html: GString,
    #[export]
    transparent: bool,
    #[export]
    background_color: Color,
    #[export]
    devtools: bool,
    #[export]
    headers: Dictionary,
    #[export]
    user_agent: GString,
    #[export]
    zoom_hotkeys: bool,
    #[export]
    clipboard: bool,
    #[export]
    incognito: bool,
    #[export]
    focused_when_created: bool,
    #[export]
    allow_interactions_without_focus: bool,
    #[export]
    forward_input_events: bool,
}

#[godot_api]
impl IControl for WebView {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            webview: None,
            previous_screen_position: Vector2::default(),
            full_window_size: true,
            url: "https://github.com/doceazedo/godot_wry".into(),
            html: "".into(),
            transparent: false,
            background_color: Color::from_rgb(1.0, 1.0, 1.0),
            devtools: true,
            headers: Dictionary::new(),
            user_agent: "".into(),
            zoom_hotkeys: false,
            clipboard: true,
            incognito: false,
            focused_when_created: true,
            allow_interactions_without_focus: true,
            forward_input_events: true,
        }
    }

    fn process(&mut self, _delta: f64) {
        if self.webview.is_none() { return }

        if self.allow_interactions_without_focus {
            let webview = self.webview.as_ref().unwrap();
            webview.focus_parent().unwrap();
        }

        if self.base().get_screen_position() != self.previous_screen_position {
            self.previous_screen_position = self.base().get_screen_position();
            self.resize();
        }

        #[cfg(target_os = "linux")]
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }

    fn ready(&mut self) {
        let window = GodotWindow;
        let base = self.base().clone();
        let webview_builder = WebViewBuilder::with_attributes(WebViewAttributes {
            url: if self.html.is_empty() { Some(String::from(&self.url)) } else { None },
            html: if self.url.is_empty() { Some(String::from(&self.html)) } else { None },
            transparent: self.transparent,
            devtools: self.devtools,
            // headers: Some(HeaderMap::try_from(self.headers.iter_shared().typed::<GString, Variant>()).unwrap_or_default()),
            user_agent: Some(String::from(&self.user_agent)),
            zoom_hotkeys_enabled: self.zoom_hotkeys,
            clipboard: self.clipboard,
            incognito: self.incognito,
            focused: self.focused_when_created,
            accept_first_mouse: true,
            ..Default::default()
        })
            .with_ipc_handler(move |req: Request<String>| {
                let body = req.body().as_str();
                
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body) {
                    if let Some(event_type) = json_value.get("type").and_then(|t| t.as_str()) {
                        if let Some(viewport) = base.clone().get_viewport() {
                            match event_type {
                                "_mouse_move" => {
                                    let x = json_value.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                    let y = json_value.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                    
                                    let movement_x = json_value.get("movementX").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                    let movement_y = json_value.get("movementY").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                    
                                    let mut event = InputEventMouseMotion::new_gd();
                                    event.set_position(Vector2::new(x, y));
                                    event.set_global_position(Vector2::new(x, y));
                                    
                                    let button_mask = CURRENT_BUTTON_MASK.lock().unwrap();
                                    event.set_button_mask(*button_mask);

                                    event.set_relative(Vector2::new(movement_x, movement_y));
                                    
                                    let mut viewport = viewport.clone();
                                    viewport.push_input(&event);
                                    return;
                                },
                                
                                "_mouse_down" | "_mouse_up" => {
                                    let x = json_value.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                    let y = json_value.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                    let button = json_value.get("button").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                    
                                    let godot_button = match button {
                                        0 => MouseButton::LEFT,
                                        1 => MouseButton::MIDDLE,
                                        2 => MouseButton::RIGHT,
                                        3 => MouseButton::WHEEL_UP,
                                        4 => MouseButton::WHEEL_DOWN,
                                        _ => MouseButton::LEFT, // default to left button
                                    };
                                    
                                    let pressed = event_type == "_mouse_down";
                                    let mask = match godot_button {
                                        MouseButton::LEFT => MouseButtonMask::LEFT,
                                        MouseButton::RIGHT => MouseButtonMask::RIGHT,
                                        MouseButton::MIDDLE => MouseButtonMask::MIDDLE,
                                        _ => MouseButtonMask::default(),
                                    };
                                    
                                    if godot_button != MouseButton::WHEEL_UP && godot_button != MouseButton::WHEEL_DOWN {
                                        let mut button_mask = CURRENT_BUTTON_MASK.lock().unwrap();
                                        if pressed {
                                            *button_mask = *button_mask | mask;
                                        } else {
                                            match godot_button {
                                                MouseButton::LEFT => {
                                                    if button_mask.is_set(MouseButtonMask::LEFT) {
                                                        *button_mask = MouseButtonMask::from_ord(button_mask.ord() & !MouseButtonMask::LEFT.ord());
                                                    }
                                                },
                                                MouseButton::RIGHT => {
                                                    if button_mask.is_set(MouseButtonMask::RIGHT) {
                                                        *button_mask = MouseButtonMask::from_ord(button_mask.ord() & !MouseButtonMask::RIGHT.ord());
                                                    }
                                                },
                                                MouseButton::MIDDLE => {
                                                    if button_mask.is_set(MouseButtonMask::MIDDLE) {
                                                        *button_mask = MouseButtonMask::from_ord(button_mask.ord() & !MouseButtonMask::MIDDLE.ord());
                                                    }
                                                },
                                                _ => {}
                                            }
                                        }
                                    }
                                    
                                    let mut event = InputEventMouseButton::new_gd();
                                    event.set_button_index(godot_button);
                                    event.set_position(Vector2::new(x, y));
                                    event.set_global_position(Vector2::new(x, y));
                                    event.set_pressed(pressed);
                                    
                                    let button_mask = CURRENT_BUTTON_MASK.lock().unwrap();
                                    event.set_button_mask(*button_mask);
                                    
                                    let mut viewport = viewport.clone();
                                    viewport.push_input(&event);
                                    return;
                                },
                                
                                "_key_down" | "_key_up" => {
                                    let key_str = json_value.get("key").and_then(|v| v.as_str()).unwrap_or("");
                                    let key_code = json_value.get("keyCode").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                    
                                    let mut event = InputEventKey::new_gd();
                                    
                                    // TODO: this is a very simplistic implementation and should be improved
                                    let godot_key = match key_str {
                                        "a" | "A" => Key::A,
                                        "b" | "B" => Key::B,
                                        "c" | "C" => Key::C,
                                        "d" | "D" => Key::D,
                                        "e" | "E" => Key::E,
                                        "f" | "F" => Key::F,
                                        "g" | "G" => Key::G,
                                        "h" | "H" => Key::H,
                                        "i" | "I" => Key::I,
                                        "j" | "J" => Key::J,
                                        "k" | "K" => Key::K,
                                        "l" | "L" => Key::L,
                                        "m" | "M" => Key::M,
                                        "n" | "N" => Key::N,
                                        "o" | "O" => Key::O,
                                        "p" | "P" => Key::P,
                                        "q" | "Q" => Key::Q,
                                        "r" | "R" => Key::R,
                                        "s" | "S" => Key::S,
                                        "t" | "T" => Key::T,
                                        "u" | "U" => Key::U,
                                        "v" | "V" => Key::V,
                                        "w" | "W" => Key::W,
                                        "x" | "X" => Key::X,
                                        "y" | "Y" => Key::Y,
                                        "z" | "Z" => Key::Z,
                                        "ArrowUp" => Key::UP,
                                        "ArrowDown" => Key::DOWN,
                                        "ArrowLeft" => Key::LEFT,
                                        "ArrowRight" => Key::RIGHT,
                                        "Enter" => Key::ENTER,
                                        "Escape" => Key::ESCAPE,
                                        "Backspace" => Key::BACKSPACE,
                                        "Tab" => Key::TAB,
                                        "Space" | " " => Key::SPACE,
                                        "0" => Key::KEY_0,
                                        "1" => Key::KEY_1,
                                        "2" => Key::KEY_2,
                                        "3" => Key::KEY_3,
                                        "4" => Key::KEY_4,
                                        "5" => Key::KEY_5,
                                        "6" => Key::KEY_6,
                                        "7" => Key::KEY_7,
                                        "8" => Key::KEY_8,
                                        "9" => Key::KEY_9,
                                        "Shift" => Key::SHIFT,
                                        "Control" => Key::CTRL,
                                        "Alt" => Key::ALT,
                                        _ => Key::NONE,
                                    };
                                    
                                    event.set_keycode(godot_key);
                                    event.set_pressed(event_type == "_key_down");
                                    godot_print!("godot_key: {:?}", godot_key);
                                    godot_print!("event_type: {:?}", event_type);
                                    
                                    let mut input = Input::singleton();
                                    input.parse_input_event(&event);
                                    return;
                                },
                                
                                _ => {}
                            }
                        }
                    }
                }
                
                // if we get here, this is a regular IPC message
                base.clone().emit_signal("ipc_message", &[body.to_variant()]);
            })
            .with_custom_protocol(
                "res".into(), move |_webview_id, request| get_res_response(request),
            );

        if !self.url.is_empty() && !self.html.is_empty() {
            godot_error!("[Godot WRY] You have entered both a URL and HTML code. You may only enter one at a time.")
        }

        if self.allow_interactions_without_focus {
            godot_print_rich!("[color=cornflowerblue][Godot WRY] The property \"Allow interactions without focus\" is enabled. This forwards input events to the engine by preventing the webview from retaining focus, but may break focus-dependent HTML elements like <input> or <textarea>. Disable if persistent focus is needed.[/color]")
        }

        let webview = webview_builder.build_as_child(&window).unwrap();
        self.webview.replace(webview);

        let mut viewport = self.base().get_tree().expect("Could not get tree").get_root().expect("Could not get viewport");
        viewport.connect("size_changed", &Callable::from_object_method(&*self.base(), "resize"));

        self.base().clone().connect("resized", &Callable::from_object_method(&*self.base(), "resize"));
        self.base().clone().connect("visibility_changed", &Callable::from_object_method(&*self.base(), "update_visibility"));

        if self.forward_input_events {
            let forward_script = r#"
                document.addEventListener('mousemove', (e) => {
                    if (!document.hasFocus()) return;
                    window.ipc.postMessage(JSON.stringify({
                        type: '_mouse_move',
                        x: e.clientX,
                        y: e.clientY,
                        movementX: e.movementX,
                        movementY: e.movementY,
                        button: e.button
                    }));
                });
                document.addEventListener('mousedown', (e) => {
                    if (!document.hasFocus()) return;
                    window.ipc.postMessage(JSON.stringify({
                        type: '_mouse_down',
                        x: e.clientX,
                        y: e.clientY,
                        button: e.button
                    }));
                });
                document.addEventListener('mouseup', (e) => {
                    if (!document.hasFocus()) return;
                    window.ipc.postMessage(JSON.stringify({
                        type: '_mouse_up', 
                        x: e.clientX,
                        y: e.clientY,
                        button: e.button
                    }));
                });
                document.addEventListener('wheel', (e) => {
                    if (!document.hasFocus()) return;
                    const button = e.deltaY < 0 ? 3 : 4; // 3 = WHEEL_UP, 4 = WHEEL_DOWN
                    
                    window.ipc.postMessage(JSON.stringify({
                        type: '_mouse_down',
                        x: e.clientX,
                        y: e.clientY,
                        button: button
                    }));
                    
                    window.ipc.postMessage(JSON.stringify({
                        type: '_mouse_up',
                        x: e.clientX,
                        y: e.clientY,
                        button: button
                    }));
                });
                document.addEventListener('keydown', (e) => {
                    if (!document.hasFocus()) return;
                    window.ipc.postMessage(JSON.stringify({
                        type: '_key_down',
                        key: e.key,
                        code: e.code,
                        keyCode: e.keyCode
                    }));
                });
                document.addEventListener('keyup', (e) => {
                    if (!document.hasFocus()) return;
                    window.ipc.postMessage(JSON.stringify({
                        type: '_key_up',
                        key: e.key,
                        code: e.code,
                        keyCode: e.keyCode
                    }));
                });
            "#;
            
            if let Some(ref webview) = self.webview {
                let _ = webview.evaluate_script(forward_script);
            }
        }

        self.resize()
    }
}

#[godot_api]
impl WebView {
    #[signal]
    fn ipc_message(message: GString);

    #[func]
    fn post_message(&self, message: GString) {
        if let Some(webview) = &self.webview {
            let message = str::replace(&*String::from(message), "'", "\\'");
            let script = str::replace("document.dispatchEvent(new CustomEvent('message', { detail: '{}' }))", "{}", &*message);
            let _ = webview.evaluate_script(&*script);
        }
    }

    #[func]
    fn resize(&self) {
        if let Some(webview) = &self.webview {
            let rect = if self.full_window_size {
                let viewport_size = self.base().get_tree().expect("Could not get tree").get_root().expect("Could not get viewport").get_size();
                Rect {
                    position: PhysicalPosition::new(0, 0).into(),
                    size: PhysicalSize::new(viewport_size.x, viewport_size.y).into(),
                }
            } else {
                let pos = self.base().get_screen_position();
                let size = self.base().get_size();
                Rect {
                    position: PhysicalPosition::new(pos.x, pos.y).into(),
                    size: PhysicalSize::new(size.x, size.y).into(),
                }
            };
            let _ = webview.set_bounds(rect);
        }
    }

    #[func]
    fn eval(&self, script: GString) {
        if let Some(webview) = &self.webview {
            let _ = webview.evaluate_script(&*String::from(script));
        }
    }

    #[func]
    fn update_visibility(&self) {
        if let Some(webview) = &self.webview {
            let visibility = self.base().is_visible_in_tree();
            webview.set_visible(visibility).expect("Could not set visibility");
            self.resize()
        }
    }

    #[func]
    fn set_visible(&self, visibility: bool) {
        if let Some(webview) = &self.webview {
            let _ = webview.set_visible(visibility);
        }
    }

    #[func]
    fn load_html(&self, html: GString) {
        if let Some(webview) = &self.webview {
            let _ = webview.load_html(&*String::from(html));
        }
    }

    #[func]
    fn load_url(&self, url: GString) {
        if let Some(webview) = &self.webview {
            let _ = webview.load_url(&*String::from(url));
        }
    }

    #[func]
    fn clear_all_browsing_data(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.clear_all_browsing_data();
        }
    }

    #[func]
    fn close_devtools(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.close_devtools();
        }
    }

    #[func]
    fn open_devtools(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.open_devtools();
        }
    }

    #[func]
    fn is_devtools_open(&self) -> bool {
        if let Some(webview) = &self.webview {
            return webview.is_devtools_open();
        }
        false
    }

    #[func]
    fn focus(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.focus();
        }
    }

    #[func]
    fn focus_parent(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.focus_parent();
        }
    }

    #[func]
    fn print(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.print();
        }
    }

    #[func]
    fn reload(&self) {
        if let Some(webview) = &self.webview {
            let _ = webview.reload();
        }
    }
}

lazy_static! {
    static ref CURRENT_BUTTON_MASK: Mutex<MouseButtonMask> = Mutex::new(MouseButtonMask::default());
}
