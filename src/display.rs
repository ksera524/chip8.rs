use std::io::Write;

use crate::chip8::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub trait Draw {
    fn draw(&self, display: &[[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT]);
}

pub struct CUIDraw;

impl Draw for CUIDraw {
    fn draw(&self, display: &[[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT]) {
        // カーソルを非表示にし、画面の一番上に移動
        print!("\x1b[?25l\x1b[H");

        // 描画用のバッファを準備
        let mut buffer = String::with_capacity(DISPLAY_HEIGHT * (DISPLAY_WIDTH + 1));

        for row in display {
            for &pixel in row {
                buffer.push(if pixel { '#' } else { ' ' });
            }
            buffer.push('\n');
        }

        // バッファの内容を一度に出力
        print!("{}", buffer);

        // カーソルを表示し、画面をフラッシュ
        print!("\x1b[?25h");
        std::io::stdout().flush().unwrap();
    }
}
