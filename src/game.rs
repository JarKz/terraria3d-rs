use sdl2::{event::Event, keyboard::Keycode};
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::Window;

pub mod player;
use player::Player;

pub mod world;
use world::World;

pub mod storage;

use crate::render::aim::Aim;

pub struct Game {
    player: Rc<RefCell<Player>>,
    world: World,
    aim: Aim,

    timer: sdl2::TimerSubsystem,
    last_frame: u32,

    start_performance_counter: u64,
}

//TODO:
//This is temporary values, need to change to methods from other structs!
const DEFAULT_SEED: u32 = 5;
const DEFAULT_BLOCK_SIZE: f32 = 1.;

impl Game {
    pub fn init(window: &Window) -> Result<Game, String> {
        let player = Rc::new(RefCell::new(Player::new(
            *super::window::ASPECT_RATIO.lock(),
            45f32,
            0.1f32,
            200f32,
        )));
        Ok(Game {
            player: player.clone(),
            world: World::new(DEFAULT_SEED, DEFAULT_BLOCK_SIZE, player),
            aim: Aim::new(
                window.width() as f32,
                window.height() as f32,
                nalgebra_glm::vec3(1., 1., 1.),
            ),

            timer: window.timer()?,
            last_frame: 0,
            start_performance_counter: 0,
        })
    }

    pub fn handle(&mut self, event: Event) {
        match event {
            Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                sdl2::mouse::MouseButton::Left => self.world.player_destroy_block_if_possible(),
                sdl2::mouse::MouseButton::Right => self.world.player_place_block_if_possible(),
                _ => (),
            },
            Event::KeyDown { keycode, .. } => {
                if keycode.is_none() {
                    return;
                }
                match keycode.unwrap() {
                    Keycode::W => self.player.borrow_mut().move_forward(),
                    Keycode::A => self.player.borrow_mut().move_left(),
                    Keycode::S => self.player.borrow_mut().move_backward(),
                    Keycode::D => self.player.borrow_mut().move_right(),
                    Keycode::Space => self.player.borrow_mut().move_up(),
                    Keycode::LShift => self.player.borrow_mut().move_down(),
                    Keycode::Num1 => self.player.borrow_mut().select_hotbar_cell(0),
                    Keycode::Num2 => self.player.borrow_mut().select_hotbar_cell(1),
                    Keycode::Num3 => self.player.borrow_mut().select_hotbar_cell(2),
                    Keycode::Num4 => self.player.borrow_mut().select_hotbar_cell(3),
                    Keycode::Num5 => self.player.borrow_mut().select_hotbar_cell(4),
                    Keycode::Num6 => self.player.borrow_mut().select_hotbar_cell(5),
                    Keycode::Num7 => self.player.borrow_mut().select_hotbar_cell(6),
                    Keycode::Num8 => self.player.borrow_mut().select_hotbar_cell(7),
                    Keycode::Num9 => self.player.borrow_mut().select_hotbar_cell(8),
                    Keycode::Num0 => self.player.borrow_mut().select_hotbar_cell(9),
                    _ => (),
                }
            }
            Event::KeyUp { keycode, .. } => {
                if keycode.is_none() {
                    return;
                }
                match keycode.unwrap() {
                    Keycode::W => self.player.borrow_mut().stop_move_forward(),
                    Keycode::A => self.player.borrow_mut().stop_move_left(),
                    Keycode::S => self.player.borrow_mut().stop_move_backward(),
                    Keycode::D => self.player.borrow_mut().stop_move_right(),
                    Keycode::Space => self.player.borrow_mut().stop_move_up(),
                    Keycode::LShift => self.player.borrow_mut().stop_move_down(),
                    _ => (),
                }
            }
            Event::MouseMotion { xrel, yrel, .. } => self
                .player
                .borrow_mut()
                .rotate_camera_by_offsets(xrel as f32, yrel as f32),
            _ => (),
        }
    }

    pub fn update(&mut self) {
        let current = self.timer.ticks();
        let delta_time = (current - self.last_frame) as f32 / 1000.0;

        self.world.update(delta_time);
        self.world.update_state();

        self.last_frame = current;
    }

    pub fn render(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.world.render();
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
        let normalized_fps = (Self::FPS60 - elapsed).floor();
        self.timer.delay(normalized_fps as u32);
        1000f32 / elapsed
    }
}
