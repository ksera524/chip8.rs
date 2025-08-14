use crate::keyboard::KeyboardInput;
use std::sync::mpsc;
use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent;
use std::cell::RefCell;
use std::rc::Rc;

pub struct WebKeyboard {
    current_key: Rc<RefCell<Option<u8>>>,
}

impl WebKeyboard {
    pub fn new() -> Self {
        let current_key = Rc::new(RefCell::new(None));
        
        // キーボードイベントリスナーを設定
        let current_key_clone = current_key.clone();
        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let key_code = match event.key().as_str() {
                "1" => Some(0x1),
                "2" => Some(0x2),
                "3" => Some(0x3),
                "4" => Some(0xC),
                "q" | "Q" => Some(0x4),
                "w" | "W" => Some(0x5),
                "e" | "E" => Some(0x6),
                "r" | "R" => Some(0xD),
                "a" | "A" => Some(0x7),
                "s" | "S" => Some(0x8),
                "d" | "D" => Some(0x9),
                "f" | "F" => Some(0xE),
                "z" | "Z" => Some(0xA),
                "x" | "X" => Some(0x0),
                "c" | "C" => Some(0xB),
                "v" | "V" => Some(0xF),
                _ => None,
            };
            
            if let Some(key) = key_code {
                *current_key_clone.borrow_mut() = Some(key);
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);

        let window = web_sys::window().expect("should have a window");
        let document = window.document().expect("should have a document");
        
        document
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .expect("should register keydown handler");
        
        closure.forget(); // メモリリークを防ぐため、クロージャを忘れる
        
        // キーリリース時にキーをクリア
        let current_key_clone2 = current_key.clone();
        let keyup_closure = Closure::wrap(Box::new(move |_event: KeyboardEvent| {
            *current_key_clone2.borrow_mut() = None;
        }) as Box<dyn FnMut(KeyboardEvent)>);

        document
            .add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())
            .expect("should register keyup handler");
        
        keyup_closure.forget();

        WebKeyboard { current_key }
    }
}

impl KeyboardInput for WebKeyboard {
    fn start_keyboard_thread(_sender: mpsc::Sender<u8>) {
        // Webでは不要
    }

    fn get_key(&self) -> Option<u8> {
        *self.current_key.borrow()
    }
}