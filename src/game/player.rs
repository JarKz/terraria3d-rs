use nalgebra_glm::*;

pub mod inventory;
use inventory::*;

struct PlayerMove {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

pub struct Player {
    projection: Mat4,

    position: Vec3,
    target: Vec3,
    up: Vec3,
    norm_up: Vec3,

    hitbox: Hitbox,

    pitch: f32,
    yaw: f32,

    sensitivity: f32,
    velocity: f32,

    move_direction: PlayerMove,

    inventory: Inventory,
    cell_in_hotbar: usize,
}

impl Player {
    const DEFAULT_PITCH: f32 = 0.0;
    const DEFAULT_YAW: f32 = -90.0;
    const DEFAULT_VELOCITY: f32 = 5.;
    const DEFAULT_SENSITIVITY: f32 = 0.1;

    const DEFAULT_MAX_UP_ROTATION: f32 = 89.0;
    const DEFAULT_MAX_DOWN_ROTATION: f32 = -89.0;

    const DEFAULT_HITBOX: Hitbox = Hitbox {
        data: [
            Vec3::new(0.4, 0.4, -0.4),
            Vec3::new(-0.4, 0.4, -0.4),
            Vec3::new(0.4, 0.4, 0.4),
            Vec3::new(-0.4, 0.4, 0.4),
            Vec3::new(0.4, -0.5, -0.4),
            Vec3::new(-0.4, -0.5, -0.4),
            Vec3::new(0.4, -0.5, 0.4),
            Vec3::new(-0.4, -0.5, 0.4),
            Vec3::new(0.4, -1.5, -0.4),
            Vec3::new(-0.4, -1.5, -0.4),
            Vec3::new(0.4, -1.5, 0.4),
            Vec3::new(-0.4, -1.5, 0.4),
        ],
    };

    pub fn new(aspect: f32, fovy: f32, near: f32, far: f32) -> Self {
        Player {
            projection: perspective(aspect, fovy, near, far),
            position: vec3(0.0, 0.0, 0.0),
            target: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
            norm_up: vec3(0.0, 1.0, 0.0),

            hitbox: Self::DEFAULT_HITBOX.clone(),

            pitch: Self::DEFAULT_PITCH,
            yaw: Self::DEFAULT_YAW,
            sensitivity: Self::DEFAULT_SENSITIVITY,
            velocity: Self::DEFAULT_VELOCITY,

            move_direction: PlayerMove {
                forward: false,
                backward: false,
                left: false,
                right: false,
                up: false,
                down: false,
            },

            inventory: Inventory::new(),
            cell_in_hotbar: 0,
        }
    }

    pub fn set_sensitivity(&mut self, new_sensitivity: f32) {
        self.sensitivity = new_sensitivity;
    }

    pub fn set_velocity(&mut self, new_velocity: f32) {
        self.velocity = new_velocity;
    }

    pub fn update_vision(&mut self, fovy: f32, near: f32, far: f32) {
        self.projection = perspective(*crate::window::ASPECT_RATIO.lock(), fovy, near, far);
    }

    pub fn move_forward(&mut self) {
        self.move_direction.forward = true;
    }

    pub fn move_backward(&mut self) {
        self.move_direction.backward = true;
    }

    pub fn move_left(&mut self) {
        self.move_direction.left = true;
    }

    pub fn move_right(&mut self) {
        self.move_direction.right = true;
    }

    pub fn move_up(&mut self) {
        self.move_direction.up = true;
    }

    pub fn move_down(&mut self) {
        self.move_direction.down = true;
    }

    pub fn stop_move_forward(&mut self) {
        self.move_direction.forward = false;
    }

    pub fn stop_move_backward(&mut self) {
        self.move_direction.backward = false;
    }

    pub fn stop_move_left(&mut self) {
        self.move_direction.left = false;
    }

    pub fn stop_move_right(&mut self) {
        self.move_direction.right = false;
    }

    pub fn stop_move_up(&mut self) {
        self.move_direction.up = false;
    }

    pub fn stop_move_down(&mut self) {
        self.move_direction.down = false;
    }

    pub fn process_move(&mut self, new_position: Vec3) {
        self.position = new_position;
    }

    pub fn get_new_position(&self, delta_time: f32) -> Vec3 {
        let direction = &self.move_direction;
        let offset = delta_time * self.velocity;
        let mut new_position = self.position.clone();
        if direction.forward {
            new_position += offset * normalize(&vec3(self.target.x, 0.0, self.target.z));
        }
        if direction.backward {
            new_position -= offset * normalize(&vec3(self.target.x, 0.0, self.target.z));
        }
        if direction.left {
            new_position -= offset * normalize(&cross(&self.target, &self.norm_up));
        }
        if direction.right {
            new_position += offset * normalize(&cross(&self.target, &self.norm_up));
        }
        if direction.up {
            new_position += offset * self.norm_up;
        }
        if direction.down {
            new_position -= offset * self.norm_up;
        }

        new_position
    }

    pub fn rotate_camera_by_offsets(&mut self, xoffset: f32, yoffset: f32) {
        self.pitch = (self.pitch - yoffset * self.sensitivity).clamp(
            Self::DEFAULT_MAX_DOWN_ROTATION,
            Self::DEFAULT_MAX_UP_ROTATION,
        );
        self.yaw += xoffset * self.sensitivity;

        let pitch_angle = radians(&vec1(self.pitch));
        let yaw_angle = radians(&vec1(self.yaw));

        let direction = vec3(
            cos(&yaw_angle).x * cos(&pitch_angle).x,
            sin(&pitch_angle).x,
            sin(&yaw_angle).x * cos(&pitch_angle).x,
        );
        self.target = normalize(&direction);
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn projection(&self) -> &Mat4 {
        &self.projection
    }

    pub fn look_at(&self) -> Mat4 {
        look_at(
            &self.position,
            &(self.position.clone() + self.target),
            &self.up,
        )
    }

    pub fn view_ray(&self) -> Vec3 {
        self.target
    }

    pub fn get_hitbox(&self, blocksize: f32) -> Hitbox {
        let mut hitbox = self.hitbox.clone();
        hitbox *= blocksize;
        hitbox
    }

    pub fn get_block_in_hand(&mut self) -> Option<Item> {
        self.inventory.get_item_from_hotbar(self.cell_in_hotbar)
    }

    pub fn pick_block(&mut self, block: u64, total: Count) {
        self.inventory.pick_item(Item::from_block(block, total));
    }

    pub fn select_hotbar_cell(&mut self, cell_position: usize) {
        let cell_position = cell_position.min(HOTBAR_SIZE - 1);
        self.cell_in_hotbar = cell_position;
    }
}

#[derive(Clone)]
pub struct Hitbox {
    pub data: [Vec3; 12],
}

impl std::ops::MulAssign<f32> for Hitbox {
    fn mul_assign(&mut self, rhs: f32) {
        self.data.iter_mut().for_each(|vec| *vec *= rhs);
    }
}

impl std::ops::AddAssign<Vec3> for Hitbox {
    fn add_assign(&mut self, rhs: Vec3) {
        self.data.iter_mut().for_each(|vec| *vec += rhs);
    }
}
