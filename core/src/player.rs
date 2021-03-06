use std::ops::{ Add, Sub };

use glm:: { Vector3, GenNum, normalize, cross, length };

use crate::{ Float, Model, Camera, UpdateError, graphics::create_direction};
use crate::traits::{ Translatable, Rotatable, Updatable };

pub struct Player {
    model: Model,
    momentum: Vector3<Float>,
    forward: Vector3<Float>,
    speed: f32,
    jumping: bool
}

impl Player {
    pub fn align_camera(&self, camera: &mut Camera) {
        let mut pos = self.get_translation();
        pos.z += 3.;
        camera.set_translation(pos);
        camera.set_rotation(self.get_rotation());
    }

    pub fn is_jumping(&self) -> bool {
        self.jumping
    }

    pub fn toggle_jump(&mut self) {
        self.jumping = !self.jumping;
    }

    pub fn jump(&mut self, force: Float) {
        self.jumping = true;
        let momentum_xy = self.momentum.truncate(2);
        let jump_momentum = match length(momentum_xy) {
            mom_xy if mom_xy < 1e-3 => {
                Vector3::new(0., 0., 1.) * force
            },
            _ => {
                (normalize(momentum_xy) * self.speed).extend(force)
            }
        };
        self.push(jump_momentum);
    }

    pub fn land(&mut self) {
        self.jumping = false;
    }

    pub fn add_move_momentum(&mut self, directions: &[bool]) {
        debug_assert!(directions.len() >= 4);
        let mut move_offset: Vector3<Float> = Vector3::from_s(0.);
        if directions[0] {
            move_offset = move_offset.add(self.forward);
        }
        if directions[1] {
            let right = cross(self.forward, Vector3::new(0., 0., 1.));
            move_offset = move_offset.sub(right);
        }
        if directions[2] {
            move_offset = move_offset.sub(self.forward);
        }
        if directions[3] {
            let right = cross(self.forward, Vector3::new(0., 0., 1.));
            move_offset = move_offset.add(right);
        }
        if length(move_offset) > 1e-3 {
            let normalized_offset = normalize(move_offset);
            self.push(normalized_offset * self.speed);
        }
    }

    pub fn apply_momentum(&mut self) {
        self.mod_translation(self.momentum);
    }

    pub fn update_forward(&mut self, forward: Vector3<Float>) {
        self.forward = forward;
    }

    pub fn move_z(&mut self, offset: Float) {
        self.mod_translation(Vector3::new(0., 0., offset));
    }

	pub fn set_z(&mut self, pos_z: Float) {
		let mut pos = self.get_translation();
		pos.z = pos_z;
		self.set_translation(pos);
	}
    
    pub fn get_z(&self) -> Float {
        self.model.get_translation().z
    }

    pub fn mod_speed(&mut self, amount: f32) {
        self.speed = f32::max(self.speed + amount, 1e-3);
    }

    pub fn get_direction(&self) -> Vector3<Float> {
        create_direction(self.get_rotation())
    }

    pub fn push(&mut self, additional_momentum: Vector3<Float>) {
        self.momentum = self.momentum.add(additional_momentum);
    }

    pub fn push_z(&mut self, additional_momentum_z: Float) {
        self.momentum.z += additional_momentum_z;
    }

    pub fn clear_momentum(&mut self) {
        self.momentum = Vector3::from_s(0.);
    }

    pub fn clear_momentum_z(&mut self) {
        self.momentum.z = 0.;
    }

    pub fn clear_momentum_neg_z(&mut self) {
        if self.momentum.z < 0. {
            self.momentum.z = 0.;
        }
    }
}

impl Default for Player {
    fn default() -> Player {
        let mut player = Player {
            model: Model::default(),
            momentum: Vector3::from_s(0.),
            forward: Vector3::from_s(0.),
            speed: 0.5,
            jumping: false
        };
        player.set_translation(Vector3::new(0., 0., 200.));
        player.set_rotation(Vector3::new(45f32.to_radians(), 125f32.to_radians(), 0.));
        player
    }
}

impl Updatable for Player {
    fn tick(&mut self, _time_passed: u32) -> Result<(), UpdateError> {
        self.apply_momentum();
        if !self.jumping {
            self.clear_momentum();
        }
        Ok(())
    }
}

impl Translatable for Player {
    fn set_translation(&mut self, new_translation: Vector3<Float>) {
        self.model.set_translation(new_translation);
    }
    fn get_translation(&self) -> Vector3<Float> {
        self.model.get_translation()
    }
}

impl Rotatable for Player {
    fn set_rotation(&mut self, new_rotation: Vector3<Float>) {
        const THRESHOLD: f32 = 0.01;
        const MIN_Y: Float = THRESHOLD;
        const MAX_Y: Float = std::f32::consts::PI as Float - THRESHOLD;
        const DOUBLE_PI: Float = 2. * std::f32::consts::PI as Float;
        let mut fixed_rotation = new_rotation;
        if fixed_rotation.x >= DOUBLE_PI {
            fixed_rotation.x -= DOUBLE_PI;
        } else if fixed_rotation.x < 0. {
            fixed_rotation.x += DOUBLE_PI;
        }
        if fixed_rotation.y < MIN_Y {
            fixed_rotation.y = MIN_Y;
        } else if fixed_rotation.y > MAX_Y {
            fixed_rotation.y = MAX_Y;
        }
        self.model.set_rotation(fixed_rotation);
    }
    fn get_rotation(&self) -> Vector3<Float> {
        self.model.get_rotation()
    }
}
