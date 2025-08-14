#[cfg(not(target_arch = "wasm32"))]
use getch_rs::{Getch, Key};
use std::{sync::mpsc, thread};

pub trait KeyboardInput {
    fn start_keyboard_thread(sender: mpsc::Sender<u8>);
    fn get_key(&self) -> Option<u8>;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct GetchKeyboard {
    receiver: mpsc::Receiver<u8>,
}

#[cfg(not(target_arch = "wasm32"))]
impl GetchKeyboard {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<u8>();
        Self::start_keyboard_thread(sender);
        GetchKeyboard { receiver }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl KeyboardInput for GetchKeyboard {
    fn start_keyboard_thread(sender: mpsc::Sender<u8>) {
        thread::spawn(move || {
            let g = Getch::new();
            loop {
                match g.getch() {
                    Ok(Key::Char('1')) => sender.send(0x1).unwrap(),
                    Ok(Key::Char('2')) => sender.send(0x2).unwrap(),
                    Ok(Key::Char('3')) => sender.send(0x3).unwrap(),
                    Ok(Key::Char('4')) => sender.send(0xC).unwrap(),
                    Ok(Key::Char('q')) => sender.send(0x4).unwrap(),
                    Ok(Key::Char('w')) => sender.send(0x5).unwrap(),
                    Ok(Key::Char('e')) => sender.send(0x6).unwrap(),
                    Ok(Key::Char('r')) => sender.send(0xD).unwrap(),
                    Ok(Key::Char('a')) => sender.send(0x7).unwrap(),
                    Ok(Key::Char('s')) => sender.send(0x8).unwrap(),
                    Ok(Key::Char('d')) => sender.send(0x9).unwrap(),
                    Ok(Key::Char('f')) => sender.send(0xE).unwrap(),
                    Ok(Key::Char('z')) => sender.send(0xA).unwrap(),
                    Ok(Key::Char('x')) => sender.send(0x0).unwrap(),
                    Ok(Key::Char('c')) => sender.send(0xB).unwrap(),
                    Ok(Key::Char('v')) => sender.send(0xF).unwrap(),
                    Ok(Key::Esc) => std::process::exit(0),
                    _ => {}
                }
            }
        });
    }

    fn get_key(&self) -> Option<u8> {
        self.receiver.try_recv().ok()
    }
}
