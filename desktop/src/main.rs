use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Duration;

fn draw_chessboard(canvas: &mut WindowCanvas) {
    const CELL_SIZE: i32 = 1;

    for y in 0..(32 / CELL_SIZE) {
        for x in 0..(64 / CELL_SIZE) {
            let color = if (x + y) % 2 == 0 {
                Color::RGB(40, 40, 40)
            } else {
                Color::RGB(90, 90, 90)
            };

            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(
                x * CELL_SIZE,
                y * CELL_SIZE,
                CELL_SIZE as u32,
                CELL_SIZE as u32,
            ));
        }
    }
}

fn main() -> Result<(), String> {
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
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "nearest"); // sharp pixels

    canvas.set_draw_color(Color::RGB(0, 255, 0));
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
                Event::Window { win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::Resized(..)
                    | sdl2::event::WindowEvent::SizeChanged(..) => {
                        println!("Resised window");
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        draw_chessboard(&mut canvas);

        canvas.set_draw_color(Color::RGB(180, 40, 40));
        canvas.draw_rect(Rect::new(0, 0, 64, 32))?;

        canvas.present();

        canvas.set_viewport(None);

        // Refresh at 60 Hz
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    println!("Chips&Rust emulator closed gracefully.");
    Ok(())
}
