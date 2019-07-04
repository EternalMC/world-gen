use glm::{ Vector2, Vector3, GenNum };

use utility::Float;

pub const CHUNK_SIZE: i32 = 64;

pub fn get_chunk_pos(world_pos: Vector3<Float>) -> Vector2<i32> {
     let mut chunk_pos = Vector2::from_s(0);
     for i in 0..2 {
          chunk_pos[i] = world_pos[i].round() as i32 / CHUNK_SIZE;
          if world_pos[i] < 0. {
               chunk_pos[i] -= 1;
          }
     }
     chunk_pos
}

#[allow(unused)]
pub fn get_chunk_relative_pos(chunk_pos: Vector3<Float>, world_pos: Vector3<Float>, resolution: i32) -> [i32; 2] {
     [(world_pos.x - chunk_pos.x).round() as i32 / resolution,
      (world_pos.y - chunk_pos.y).round() as i32 / resolution]
}

pub fn get_world_pos(chunk_pos: &Vector2<i32>, offset: &[i32; 2], resolution: i32) -> [Float; 2] {
    [((chunk_pos.x * CHUNK_SIZE) + offset[0] * resolution) as Float,
     ((chunk_pos.y * CHUNK_SIZE) + offset[1] * resolution) as Float]
}
