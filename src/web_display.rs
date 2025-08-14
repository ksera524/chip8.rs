use crate::chip8::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::display::Draw;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct WebDraw {
    context: CanvasRenderingContext2d,
    pixel_size: f64,
}

impl WebDraw {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("Failed to get window")?;
        let document = window.document().ok_or("Failed to get document")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("Failed to get canvas element")?
            .dyn_into::<HtmlCanvasElement>()?;
        
        // Canvasのサイズを設定（64x32の10倍）
        let pixel_size = 10.0;
        canvas.set_width((DISPLAY_WIDTH as f64 * pixel_size) as u32);
        canvas.set_height((DISPLAY_HEIGHT as f64 * pixel_size) as u32);
        
        let context = canvas
            .get_context("2d")?
            .ok_or("Failed to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        // 背景を黒に設定
        context.set_fill_style(&"#000000".into());
        context.fill_rect(0.0, 0.0, 
            DISPLAY_WIDTH as f64 * pixel_size,
            DISPLAY_HEIGHT as f64 * pixel_size);
        
        Ok(WebDraw { context, pixel_size })
    }
}

impl Draw for WebDraw {
    fn draw(&self, display: &[[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT]) {
        // 画面全体をクリア（黒で塗りつぶし）
        self.context.set_fill_style(&"#000000".into());
        self.context.fill_rect(0.0, 0.0,
            DISPLAY_WIDTH as f64 * self.pixel_size,
            DISPLAY_HEIGHT as f64 * self.pixel_size);
        
        // ピクセルを描画（白で塗りつぶし）
        self.context.set_fill_style(&"#ffffff".into());
        
        for (y, row) in display.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                if pixel {
                    self.context.fill_rect(
                        x as f64 * self.pixel_size,
                        y as f64 * self.pixel_size,
                        self.pixel_size,
                        self.pixel_size,
                    );
                }
            }
        }
    }
}