use crate::window::Window;
use sdl2::{event::Event, keyboard::Keycode};

pub mod player;
use player::Player;

pub mod world;
use world::World;

pub mod storage;

use crate::render::aim::Aim;

pub struct Game {
    player: Player,
    world: World,
    aim: Aim,

    timer: sdl2::TimerSubsystem,
    last_frame: u32,

    start_performance_counter: u64,
}

//TODO:
//This is temporary values, need to change to methods from other structs!
const DEFAULT_SEED: u32 = 2;
const DEFAULT_BLOCK_SIZE: f32 = 1.;

impl Game {
    pub fn init(window: &Window) -> Result<Game, String> {
        let aspect_ratio = window.width() as f32 / window.height() as f32;
        Ok(Game {
            player: Player::new(aspect_ratio, 45f32, 0.1f32, 200f32),
            world: World::new(DEFAULT_SEED, DEFAULT_BLOCK_SIZE),
            aim: Aim::new(window.width() as f32, window.height() as f32, nalgebra_glm::vec3(1., 1., 1.)),

            timer: window.timer()?,
            last_frame: 0,
            start_performance_counter: 0,
        })
    }

    pub fn handle(&mut self, event: Event) {
        match event {
            Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                sdl2::mouse::MouseButton::Left => {
                    self.world
                        .destroy_block_if_possible(self.player.position(), self.player.view_ray());
                }
                sdl2::mouse::MouseButton::Right => (),
                _ => (),
            },
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
        let delta_timer = (current - self.last_frame) as f32 / 1000.0;

        self.player.process_move(delta_timer);
        self.world.update_player_position(self.player.position());
        self.world.update_state();

        self.last_frame = current;
    }

    pub fn render(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.world.render(&self.player);
        self.aim.render();
    }

    pub fn new_loop(&mut self) {
        self.start_performance_counter = self.timer.performance_counter();
    }

    const FPS60: f32 = 16.66668;

    pub fn fps(&mut self) -> f32 {
        let elapsed = (self.timer.performance_counter() - self.start_performance_counter) as f32
            / self.timer.performance_frequency() as f32
            * 1000.0;
        //TODO: Fix. Need to show normal FPS
        let normalized_fps = (Self::FPS60 - elapsed).floor();
        self.timer.delay(normalized_fps as u32);
        1000f32 / normalized_fps
    }
}
