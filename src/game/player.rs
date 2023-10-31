use nalgebra_glm::*;

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

    pitch: f32,
    yaw: f32,

    sensitivity: f32,
    velocity: f32,

    move_direction: PlayerMove,
}

impl Player {
    const DEFAULT_PITCH: f32 = 0.0;
    const DEFAULT_YAW: f32 = -90.0;
    const DEFAULT_VELOCITY: f32 = 2.5;
    const DEFAULT_SENSITIVITY: f32 = 0.1;

    const DEFAULT_MAX_UP_ROTATION: f32 = 89.0;
    const DEFAULT_MAX_DOWN_ROTATION: f32 = -89.0;

    pub fn new(aspect: f32, fovy: f32, near: f32, far: f32) -> Self {
        Player {
            projection: perspective(aspect, fovy, near, far),
            //TODO:
            //CHANGE IT IN FUTURE TO NORMAL POSITION
            position: vec3(10.0, 0.0, 10.0),
            target: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
            pitch: Self::DEFAULT_PITCH,
            yaw: Self::DEFAULT_YAW,
            sensitivity: Self::DEFAULT_SENSITIVITY,
            velocity: Self::DEFAULT_VELOCITY,

            norm_up: vec3(0.0, 1.0, 0.0),
            move_direction: PlayerMove {
                forward: false,
                backward: false,
                left: false,
                right: false,
                up: false,
                down: false,
            },
        }
    }

    pub fn set_sensitivity(&mut self, new_sensitivity: f32) {
        self.sensitivity = new_sensitivity;
    }

    pub fn set_velocity(&mut self, new_velocity: f32) {
        self.velocity = new_velocity;
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

    pub fn process_move(&mut self, delta_time: f32) {
        let direction = &self.move_direction;
        let offset = delta_time * self.velocity;
        if direction.forward {
            self.position += offset * normalize(&vec3(self.target.x, 0.0, self.target.z));
        }
        if direction.backward {
            self.position -= offset * normalize(&vec3(self.target.x, 0.0, self.target.z));
        }
        if direction.left {
            self.position -= offset * normalize(&cross(&self.target, &self.norm_up));
        }
        if direction.right {
            self.position += offset * normalize(&cross(&self.target, &self.norm_up));
        }
        if direction.up {
            self.position += offset * self.norm_up;
        }
        if direction.down {
            self.position -= offset * self.norm_up;
        }
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

    pub fn position(&self) -> &Vec3 {
        &self.position
    }

    pub fn projection(&self) -> &Mat4 {
        &self.projection
    }

    pub fn look_at(&self) -> Mat4 {
        look_at(&self.position, &self.target, &self.up)
    }
}
