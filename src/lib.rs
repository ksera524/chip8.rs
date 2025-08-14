mod chip8;
mod display;
mod keyboard;
mod web_display;
mod web_keyboard;

use wasm_bindgen::prelude::*;
use web_sys::console;
use std::cell::RefCell;

use chip8::Cpu;
use display::Draw;
use web_display::WebDraw;
use web_keyboard::WebKeyboard;

// ログ出力のマクロ
macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!( $( $t )* ).into());
    }
}

// グローバル状態を管理する構造体
pub struct GameState {
    cpu: Cpu<WebKeyboard>,
    drawer: WebDraw,
    last_cpu_time: f64,
    last_timer_time: f64,
    current_rom: Vec<u8>, // 現在のROMデータを保持
}

thread_local! {
    static GAME_STATE: RefCell<Option<GameState>> = RefCell::new(None);
}

// ログ初期化の状態を管理
thread_local! {
    static LOGGER_INITIALIZED: RefCell<bool> = RefCell::new(false);
}

#[wasm_bindgen]
pub fn init_wasm() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    
    // ロガーを一度だけ初期化
    LOGGER_INITIALIZED.with(|initialized| {
        if !*initialized.borrow() {
            console_log::init_with_level(log::Level::Debug)
                .map_err(|e| JsValue::from_str(&format!("Failed to init logger: {:?}", e)))?;
            *initialized.borrow_mut() = true;
        }
        Ok(())
    })
}

#[wasm_bindgen]
pub fn init_game(canvas_id: &str, rom_data: &[u8]) -> Result<(), JsValue> {
    log!("Initializing CHIP-8 emulator");
    
    let keyboard = WebKeyboard::new();
    let drawer = WebDraw::new(canvas_id)?;
    
    // CPUを初期化（ROM データを直接渡す）
    let cpu = Cpu::from_bytes(rom_data, keyboard);
    
    let now = js_sys::Date::now();
    let game_state = GameState {
        cpu,
        drawer,
        last_cpu_time: now,
        last_timer_time: now,
        current_rom: rom_data.to_vec(), // ROMデータを保存
    };
    
    GAME_STATE.with(|state| {
        *state.borrow_mut() = Some(game_state);
    });
    
    log!("CHIP-8 emulator initialized successfully");
    Ok(())
}

#[wasm_bindgen]
pub fn game_loop() {
    GAME_STATE.with(|state_cell| {
        if let Some(ref mut state) = *state_cell.borrow_mut() {
            const CPU_FREQUENCY: f64 = 600.0; // 600命令/秒
            const TIMER_FREQUENCY: f64 = 60.0; // 60Hz固定
            
            let cpu_interval = 1000.0 / CPU_FREQUENCY; // ミリ秒
            let timer_interval = 1000.0 / TIMER_FREQUENCY; // ミリ秒
            
            let now = js_sys::Date::now();
            
            // CPU命令実行（600Hz）
            if now - state.last_cpu_time >= cpu_interval {
                state.cpu.update();
                state.drawer.draw(state.cpu.get_display());
                state.last_cpu_time = now;
            }
            
            // タイマー減算（60Hz）
            if now - state.last_timer_time >= timer_interval {
                state.cpu.decrement_timers();
                state.last_timer_time = now;
            }
        }
    });
}

#[wasm_bindgen]
pub fn reset_current_game() -> Result<(), JsValue> {
    GAME_STATE.with(|state_cell| {
        if let Some(ref mut state) = *state_cell.borrow_mut() {
            // 現在のROMデータを使ってCPUを再初期化
            let keyboard = WebKeyboard::new();
            let rom_data = state.current_rom.clone();
            
            // CPUを完全にリセット
            state.cpu = Cpu::from_bytes(&rom_data, keyboard);
            state.last_cpu_time = js_sys::Date::now();
            state.last_timer_time = js_sys::Date::now();
            
            // 画面をクリア
            state.drawer.draw(state.cpu.get_display());
            
            log!("Game reset successfully");
            Ok(())
        } else {
            Err(JsValue::from_str("No game is currently loaded"))
        }
    })
}

#[wasm_bindgen]
pub fn stop_game() {
    GAME_STATE.with(|state_cell| {
        *state_cell.borrow_mut() = None;
    });
    log!("Game stopped");
}

#[wasm_bindgen]
pub fn is_game_running() -> bool {
    GAME_STATE.with(|state_cell| {
        state_cell.borrow().is_some()
    })
}

// ROMファイルのデータを組み込み
const BRIX_ROM: &[u8] = include_bytes!("../rom/BRIX");
const INVADERS_ROM: &[u8] = include_bytes!("../rom/INVADERS");
const GUESS_ROM: &[u8] = include_bytes!("../rom/GUESS");

#[wasm_bindgen]
pub fn load_brix(canvas_id: &str) -> Result<(), JsValue> {
    init_game(canvas_id, BRIX_ROM)
}

#[wasm_bindgen]
pub fn load_invaders(canvas_id: &str) -> Result<(), JsValue> {
    init_game(canvas_id, INVADERS_ROM)
}

#[wasm_bindgen]
pub fn load_guess(canvas_id: &str) -> Result<(), JsValue> {
    init_game(canvas_id, GUESS_ROM)
}