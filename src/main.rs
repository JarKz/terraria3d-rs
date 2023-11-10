extern crate gl;
extern crate sdl2;

pub mod window;
use window::*;

pub mod game;
use game::*;

mod render;

fn run() {
    let window = Window::from("Terraria 3D", Window::WIDTH, Window::HEIGHT)
        .expect("Window cannot be initialized!");
    let mut events = window
        .event_pump()
        .expect("Must be initialized sdl for pumping event!");

    let mut game = Game::init(&window).expect("Game cannot be initialized!");

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
        println!("Current FPS: {}", game.fps());

        window.update();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "macos") {
        run();
    } else {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .name(String::from("Main"))
            .spawn(run)
            .unwrap()
            .join()
            .unwrap();
    }
    Ok(())
}
