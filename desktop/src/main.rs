use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().map_err(|e| format!("SDL2 init failed: {}", e))?;
    let video_subsystem = sdl_context
        .video()
        .map_err(|e| format!("Video subsystem init failed: {}", e))?;

    const SCALE: u32 = 10;
    const WINDOW_HEIGHT: u32 = 32 * SCALE;
    const WINDOW_WIDTH: u32 = 64 * SCALE;

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
                    println!("Key down: {:?}", key);
                }
                Event::Window { win_event, .. } => {
                    if let sdl2::event::WindowEvent::Resized(w, h) = win_event {
                        println!("Window resized to {}x{}", w, h);
                    }
                }
                _ => {}
            }
        }

        // Refresh at 60 Hz
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    println!("Chips&Rust emulator closed gracefully.");
    Ok(())
}
