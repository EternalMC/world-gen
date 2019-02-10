use crate::world::WorldError;

pub trait Updatable {
    fn tick(&mut self, time_passed: u32) -> Result<(), WorldError>;
}
