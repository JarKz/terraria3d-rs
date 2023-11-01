extern crate gl;
extern crate sdl2;

pub mod window;
use window::*;

pub mod game;
use game::*;

mod render;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = Window::from("Terraria 3D", Window::WIDTH, Window::HEIGHT)?;
    let mut events = window.event_pump()?;

    let mut game = Game::init(&window)?;

    'mainloop: loop {
        game.new_loop();

        while let Some(event) = events.poll_event() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'mainloop,
                other => game.handle(other),
            }
        }

        game.update();
        game.render();
        game.fps();
        // println!("Current FPS: {}", game.fps());

        window.update();
    }

    Ok(())
}
