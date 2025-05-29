mod godot_window;
mod protocols;

use godot::global::MouseButtonMask;
use godot::init::*;
use godot::prelude::*;
use godot::classes::{Control, DisplayServer, IControl, InputEventMouseButton, InputEventMouseMotion, InputEventKey};
use godot::global::{Key, MouseButton};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use wry::{WebViewBuilder, Rect, WebViewAttributes};
use wry::dpi::{PhysicalPosition, PhysicalSize};
use wry::http::Request;
use crate::godot_window::GodotWindow;
use crate::protocols::get_res_response;
use serde_json;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::collections::HashMap;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrA, SetWindowLongPtrA, GWL_STYLE};

#[cfg(target_os = "windows")]
#[link(name = "wevtapi")]
extern "system" {}

struct GodotWRY;

#[gdextension]
unsafe impl ExtensionLibrary for GodotWRY {}

#[derive(GodotClass)]
#[class(base=Control)]
struct WebView {
    base: Base<Control>,
    webview: Option<wry::WebView>,
    previous_screen_position: Vector2,
    previous_viewport_size: Vector2i,
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
    forward_input_events: bool,
}

#[godot_api]
impl IControl for WebView {
    fn init(base: Base<Control>) -> Self {
        Self {
            base,
            webview: None,
            previous_screen_position: Vector2::default(),
            previous_viewport_size: Vector2i::default(),
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
            forward_input_events: true,
        }
    }

    fn ready(&mut self) {
        self.create_webview();
    }

    fn process(&mut self, _delta: f64) {
        self.update_webview();
    }
}

#[godot_api]
impl WebView {
    #[signal]
    fn ipc_message(message: GString);

    #[func]
    fn update_webview(&mut self) {
        if let Some(_) = &self.webview {
            let viewport_size = self.base().get_tree().expect("Could not get tree").get_root().expect("Could not get viewport").get_size();

            if self.base().get_screen_position() != self.previous_screen_position || viewport_size != self.previous_viewport_size {
                self.previous_screen_position = self.base().get_screen_position();
                self.previous_viewport_size = viewport_size;
                self.resize();
            }

            #[cfg(target_os = "linux")]
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }
    }

    #[func]
    fn create_webview(&mut self) {
        let display_server = DisplayServer::singleton();
        if display_server.get_name() == "headless".into()
        {
            godot_warn!("Godot WRY: Headless mode detected. webview will not be created.");
            return;
        }

        let window = GodotWindow;

        // remove WS_CLIPCHILDREN from the window style
        // otherwise, transparent on windows won't work
        #[cfg(target_os = "windows")]
        {
            let handle = window.window_handle().unwrap().as_raw();
            let raw_handle: HWND = match handle {
                RawWindowHandle::Win32(win32) => HWND(win32.hwnd.get() as _),
                _ => {
                    panic!("Unsupported window handle type");
                }
            };

            unsafe {
                let current_style = GetWindowLongPtrA(raw_handle, GWL_STYLE);
                // remove WS_CLIPCHILDREN
                SetWindowLongPtrA(raw_handle, GWL_STYLE, current_style & !0x02000000);
            };
        }

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
                                    
                                    let godot_key = GODOT_KEYS.get(key_str).copied().unwrap_or(Key::NONE);
                                    
                                    event.set_keycode(godot_key);
                                    event.set_pressed(event_type == "_key_down");
                                    
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

    static ref GODOT_KEYS: HashMap<&'static str, Key> = HashMap::from([
        // https://docs.godotengine.org/en/stable/classes/class_%40globalscope.html#enum-globalscope-key

        ("a", Key::A),
        ("A", Key::A),
        ("b", Key::B),
        ("B", Key::B),
        ("c", Key::C),
        ("C", Key::C),
        ("d", Key::D),
        ("D", Key::D),
        ("e", Key::E),
        ("E", Key::E),
        ("f", Key::F),
        ("F", Key::F),
        ("g", Key::G),
        ("G", Key::G),
        ("h", Key::H),
        ("H", Key::H),
        ("i", Key::I),
        ("I", Key::I),
        ("j", Key::J),
        ("J", Key::J),
        ("k", Key::K),
        ("K", Key::K),
        ("l", Key::L),
        ("L", Key::L),
        ("m", Key::M),
        ("M", Key::M),
        ("n", Key::N),
        ("N", Key::N),
        ("o", Key::O),
        ("O", Key::O),
        ("p", Key::P),
        ("P", Key::P),
        ("q", Key::Q),
        ("Q", Key::Q),
        ("r", Key::R),
        ("R", Key::R),
        ("s", Key::S),
        ("S", Key::S),
        ("t", Key::T),
        ("T", Key::T),
        ("u", Key::U),
        ("U", Key::U),
        ("v", Key::V),
        ("V", Key::V),
        ("w", Key::W),
        ("W", Key::W),
        ("x", Key::X),
        ("X", Key::X),
        ("y", Key::Y),
        ("Y", Key::Y),
        ("z", Key::Z),
        ("Z", Key::Z),
        
        ("0", Key::KEY_0),
        ("1", Key::KEY_1),
        ("2", Key::KEY_2),
        ("3", Key::KEY_3),
        ("4", Key::KEY_4),
        ("5", Key::KEY_5),
        ("6", Key::KEY_6),
        ("7", Key::KEY_7),
        ("8", Key::KEY_8),
        ("9", Key::KEY_9),
        ("Numpad0", Key::KP_0),
        ("Numpad1", Key::KP_1),
        ("Numpad2", Key::KP_2),
        ("Numpad3", Key::KP_3),
        ("Numpad4", Key::KP_4),
        ("Numpad5", Key::KP_5),
        ("Numpad6", Key::KP_6),
        ("Numpad7", Key::KP_7),
        ("Numpad8", Key::KP_8),
        ("Numpad9", Key::KP_9),
        
        ("F1", Key::F1),
        ("F2", Key::F2),
        ("F3", Key::F3),
        ("F4", Key::F4),
        ("F5", Key::F5),
        ("F6", Key::F6),
        ("F7", Key::F7),
        ("F8", Key::F8),
        ("F9", Key::F9),
        ("F10", Key::F10),
        ("F11", Key::F11),
        ("F12", Key::F12),
        ("F13", Key::F13),
        ("F14", Key::F14),
        ("F15", Key::F15),
        ("F16", Key::F16),
        ("F17", Key::F17),
        ("F18", Key::F18),
        ("F19", Key::F19),
        ("F20", Key::F20),
        ("F21", Key::F21),
        ("F22", Key::F22),
        ("F23", Key::F23),
        ("F24", Key::F24),
        
        ("ArrowUp", Key::UP),
        ("ArrowDown", Key::DOWN),
        ("ArrowLeft", Key::LEFT),
        ("ArrowRight", Key::RIGHT),
        
        ("Enter", Key::ENTER),
        ("NumpadEnter", Key::KP_ENTER),
        ("Tab", Key::TAB),
        ("Space", Key::SPACE),
        (" ", Key::SPACE),
        ("Backspace", Key::BACKSPACE),
        ("Escape", Key::ESCAPE),
        ("CapsLock", Key::CAPSLOCK),
        ("ScrollLock", Key::SCROLLLOCK),
        ("NumLock", Key::NUMLOCK),
        ("PrintScreen", Key::PRINT),
        ("Pause", Key::PAUSE),
        ("Insert", Key::INSERT),
        ("Home", Key::HOME),
        ("PageUp", Key::PAGEUP),
        ("Delete", Key::DELETE),
        ("End", Key::END),
        ("PageDown", Key::PAGEDOWN),
        
        ("Shift", Key::SHIFT),
        ("Control", Key::CTRL),
        ("Alt", Key::ALT),
        ("AltGraph", Key::ALT),
        ("Meta", Key::META),
        ("ContextMenu", Key::MENU),
        
        ("NumpadMultiply", Key::KP_MULTIPLY),
        ("NumpadDivide", Key::KP_DIVIDE),
        ("NumpadAdd", Key::KP_ADD),
        ("NumpadSubtract", Key::KP_SUBTRACT),
        ("NumpadDecimal", Key::KP_PERIOD),
        
        ("MediaPlayPause", Key::MEDIAPLAY),
        ("MediaStop", Key::MEDIASTOP),
        ("MediaTrackNext", Key::MEDIANEXT),
        ("MediaTrackPrevious", Key::MEDIAPREVIOUS),
        ("VolumeDown", Key::VOLUMEDOWN),
        ("VolumeUp", Key::VOLUMEUP),
        ("VolumeMute", Key::VOLUMEMUTE),
        
        ("BrowserBack", Key::BACK),
        ("BrowserForward", Key::FORWARD),
        ("BrowserRefresh", Key::REFRESH),
        ("BrowserStop", Key::STOP),
        ("BrowserSearch", Key::SEARCH),
        ("BrowserHome", Key::HOMEPAGE),
        
        ("`", Key::QUOTELEFT),
        ("~", Key::ASCIITILDE),
        ("!", Key::EXCLAM),
        ("@", Key::AT),
        ("#", Key::NUMBERSIGN),
        ("$", Key::DOLLAR),
        ("%", Key::PERCENT),
        ("^", Key::ASCIICIRCUM),
        ("&", Key::AMPERSAND),
        ("*", Key::ASTERISK),
        ("(", Key::PARENLEFT),
        (")", Key::PARENRIGHT),
        ("-", Key::MINUS),
        ("_", Key::UNDERSCORE),
        ("=", Key::EQUAL),
        ("+", Key::PLUS),
        ("[", Key::BRACKETLEFT),
        ("{", Key::BRACELEFT),
        ("]", Key::BRACKETRIGHT),
        ("}", Key::BRACERIGHT),
        ("\\", Key::BACKSLASH),
        ("|", Key::BAR),
        (";", Key::SEMICOLON),
        (":", Key::COLON),
        ("'", Key::APOSTROPHE),
        ("\"", Key::QUOTEDBL),
        (",", Key::COMMA),
        ("<", Key::LESS),
        (".", Key::PERIOD),
        (">", Key::GREATER),
        ("/", Key::SLASH),
        ("?", Key::QUESTION),
    ]);
}
