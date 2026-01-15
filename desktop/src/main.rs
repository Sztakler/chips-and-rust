use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Duration;

use std::fs::File;
use std::io::Read;

use chip8_core::{Emu, SCREEN_HEIGHT, SCREEN_WIDTH};

use clap::Parser;
use serde::Deserialize;

mod audio;

#[derive(Parser, Debug)]
#[command(author, version, about = "CHIP-8 Emulator in Rust")]
struct Args {
    #[arg(long)]
    // Pixel color in RGB format (e.g. 255, 0, 0)
    pixel_color: Option<String>,

    #[arg(long)]
    // Background color in RGB format (e.g. 255, 0, 0)
    background_color: Option<String>,

    rom_path: String,
}

#[derive(Deserialize)]
struct Config {
    pixel_color: (u8, u8, u8),
    background_color: (u8, u8, u8),
}

impl Config {
    fn load() -> Self {
        let config_str = std::fs::read_to_string("config.toml").unwrap_or_else(|_| {
            "pixel_color = [0, 255, 0]\nbackground_color = [0, 0, 0]".to_string()
        });

        toml::from_str(&config_str).expect("Error in formatting of the config.toml file")
    }
}

fn map_keycode(keycode: Keycode) -> Option<usize> {
    match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),

        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),

        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),

        Keycode::Z => Some(0x8),
        Keycode::X => Some(0x9),
        Keycode::C => Some(0x0),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

struct Display {
    background_color: Color,
    pixel_color: Color,
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    pub fn new() -> Self {
        Self {
            pixel_color: Color::RGB(0, 255, 0),
            background_color: Color::RGB(0, 0, 0),
        }
    }

    pub fn draw(
        &self,
        canvas: &mut WindowCanvas,
        screen_data: &[bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    ) -> Result<(), String> {
        canvas.set_draw_color(self.background_color);
        canvas.clear();

        canvas.set_draw_color(self.pixel_color);

        let range = 0..(SCREEN_WIDTH * SCREEN_HEIGHT);
        for i in range {
            if screen_data[i] {
                let x = (i % SCREEN_WIDTH) as i32;
                let y = (i / SCREEN_WIDTH) as i32;

                canvas.fill_rect(Rect::new(x, y, 1, 1))?;
            }
        }

        Ok(())
    }
}

fn parse_color(color_str: Option<String>, default: (u8, u8, u8)) -> (u8, u8, u8) {
    color_str
        .and_then(|s| {
            let parts: Vec<u8> = s.split(',').filter_map(|s| s.parse().ok()).collect();
            if parts.len() == 3 {
                Some((parts[0], parts[1], parts[2]))
            } else {
                None
            }
        })
        .unwrap_or(default)
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let mut config = Config::load();
    config.pixel_color = parse_color(args.pixel_color, config.pixel_color);
    config.background_color = parse_color(args.background_color, config.background_color);

    let mut emu = Emu::new();

    let mut rom_file = File::open(&args.rom_path)
        .map_err(|e| format!("Error while opening file ROM ({}): {}", &args.rom_path, e))?;

    let mut rom_buffer = Vec::new();
    rom_file
        .read_to_end(&mut rom_buffer)
        .map_err(|e| format!("Error while loading ROM: {}", e))?;

    emu.load_rom(&rom_buffer);
    println!("Loaded ROM: {} ({}) bytes", args.rom_path, rom_buffer.len());

    let display = Display {
        pixel_color: Color::RGB(
            config.pixel_color.0,
            config.pixel_color.1,
            config.pixel_color.2,
        ),
        background_color: Color::RGB(
            config.background_color.0,
            config.background_color.1,
            config.background_color.2,
        ),
    };

    let sdl_context = sdl2::init().map_err(|e| format!("SDL2 init failed: {}", e))?;
    let video_subsystem = sdl_context
        .video()
        .map_err(|e| format!("Video subsystem init failed: {}", e))?;

    const WINDOW_HEIGHT: u32 = 32;
    const WINDOW_WIDTH: u32 = 64;

    let window = video_subsystem
        .window("Chips&Rust", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .allow_highdpi()
        .build()
        .map_err(|e| format!("Window creation failed: {}", e))?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .map_err(|e| format!("Canvas creation failed: {}", e))?;

    let _ = canvas.set_logical_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "nearest");

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|e| format!("Event pump failed: {}", e))?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                // Window closed (cross)
                Event::Quit { .. } => break 'running,
                // Escape key -- exit
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(index) = map_keycode(key) {
                        emu.set_key(index, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(index) = map_keycode(key) {
                        emu.set_key(index, false);
                    }
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        emu.fill_screen_random();
        display.draw(&mut canvas, emu.get_screen())?;

        canvas.present();

        canvas.set_viewport(None);

        // Refresh at 60 Hz
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    println!("Chips&Rust emulator closed gracefully.");
    Ok(())
}
