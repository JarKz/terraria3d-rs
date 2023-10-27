use crate::window::Window;
use sdl2::{event::Event, keyboard::Keycode};

pub mod player;
use player::Player;

pub mod world;
use world::World;

pub struct Game {
    player: Player,
    world: World,

    timer: sdl2::TimerSubsystem,
    last_frame: u32,

    start_performance_counter: u64,
}

//TODO:
//This is temporary values, need to change to methods from other structs!
const DEFAULT_SEED: u32 = 1;
const DEFAULT_BLOCK_SIZE: f32 = 0.4;

impl Game {
    pub fn init(window: &Window) -> Result<Game, String> {
        Ok(Game {
            player: Player::new(window.width() as f32, window.height() as f32, 0.1f32, 30f32),
            world: World::new(DEFAULT_SEED, DEFAULT_BLOCK_SIZE),

            timer: window.timer()?,
            last_frame: 0,
            start_performance_counter: 0,
        })
    }

    pub fn handle(&mut self, event: Event) {
        match event {
            Event::KeyDown { keycode, .. } => {
                if keycode.is_none() {
                    return;
                }
                match keycode.unwrap() {
                    Keycode::W => self.player.move_forward(),
                    Keycode::A => self.player.move_left(),
                    Keycode::S => self.player.move_backward(),
                    Keycode::D => self.player.move_right(),
                    Keycode::Space => self.player.move_up(),
                    Keycode::LShift => self.player.move_down(),
                    _ => (),
                }
            }
            Event::KeyUp { keycode, .. } => {
                if keycode.is_none() {
                    return;
                }
                match keycode.unwrap() {
                    Keycode::W => self.player.stop_move_forward(),
                    Keycode::A => self.player.stop_move_left(),
                    Keycode::S => self.player.stop_move_backward(),
                    Keycode::D => self.player.stop_move_right(),
                    Keycode::Space => self.player.stop_move_up(),
                    Keycode::LShift => self.player.stop_move_down(),
                    _ => (),
                }
            }
            Event::MouseMotion { xrel, yrel, .. } => {
                self.player
                    .rotate_camera_by_offsets(xrel as f32, yrel as f32);
            }
            _ => (),
        }
    }

    pub fn update(&mut self) {
        let current = self.timer.ticks();
        let delta_timer = (self.last_frame - self.timer.ticks()) as f32;

        self.player.process_move(delta_timer);

        //TODO: this function call have no affect!
        self.world.update_shaders(self.player.position());

        self.last_frame = current;
    }

    pub fn render(&mut self) {}

    pub fn new_loop(&mut self) {
        self.start_performance_counter = self.timer.performance_counter();
    }

    const FPS60: f32 = 16.66668;

    pub fn fps(&mut self) -> f32 {
        let elapsed = (self.timer.performance_counter() - self.start_performance_counter) as f32
            / self.timer.performance_frequency() as f32
            * 1000.0;
        let normalized_fps = (Self::FPS60 - elapsed).floor();
        self.timer.delay(normalized_fps as u32);
        1000f32 / normalized_fps
    }
}
