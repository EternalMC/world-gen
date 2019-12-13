use glm::{normalize, GenNum, Vector2, Vector3};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::ops::Add;

use super::cell::Cell;
use super::direction::{get_neighbour_pos, get_opposite_direction, Direction};
use crate::chunk::height_map::HeightMap;

// http://www-ljk.imag.fr/Publications/Basilic/com.lmc.publi.PUBLI_Inproceedings@117681e94b6_fff75c/FastErosion_PG07.pdf

const GRAVITY: f64 = 1.;
const TIME_DELTA: f64 = 0.005;
const PIPE_AREA: f64 = 0.001;
const PIPE_LENGTH: f64 = 1.;
const GRID_DISTANCE: [f64; 2] = [1., 1.];
const SEDIMENT_CAPACITY_CONSTANT: f64 = 35.;
const DISSOLVING_CONSTANT: f64 = 0.0012;
const DEPOSITION_CONSTANT: f64 = 0.0012;
const EVAPORATION_CONSTANT: f64 = 0.001;

const NEIGHBOURS: [Direction; 4] = [
    Direction::TOP,
    Direction::RIGHT,
    Direction::BOTTOM,
    Direction::LEFT,
];

pub struct HydraulicErosion {
    rng: SmallRng,
    size: i32,
    cells: Vec<Cell>,
    pipe_area: f64,
    pipe_length: f64,
    grid_distance: [f64; 2],
    gravity: f64,
}

impl HydraulicErosion {
    pub fn new<R: Rng + ?Sized>(height_map: &HeightMap, rng_input: &mut R) -> Self {
        let size = height_map.get_size();
        let mut cells = Vec::with_capacity((size * size) as usize);
        for y in 0..size {
            for x in 0..size {
                let pos = [x, y];
                let mut cell = Cell::default();
                cell.set_terrain_height(height_map.get(&pos));
                cells.push(cell);
            }
        }

        let erosion = Self {
            rng: SmallRng::from_rng(rng_input).unwrap(),
            size: size,
            cells: cells,
            pipe_area: PIPE_AREA,
            pipe_length: PIPE_LENGTH,
            grid_distance: GRID_DISTANCE,
            gravity: GRAVITY,
        };
        erosion
    }

    pub fn print(&self) {
        for y in 0..self.size {
            for x in 0..self.size {
                let c = self.get_cell(&[x, y]);
                print!("{:.2} ", c.get_water_height());
            }
            print!("\n");
        }
        print!("\n\n");
    }

    pub fn simulate(&mut self, count: u32) {
        for _i in 0..count {
            if !self.has_water() {
                break;
            }
            self.print();
            self.tick();
        }
    }

    fn tick(&mut self) {
        self.update_outflow(TIME_DELTA);
        self.apply_waterflow(TIME_DELTA);
        self.apply_erosion_deposition();
        self.update_transported_sediment(TIME_DELTA);
        self.apply_sediment_transportation();
        self.apply_water_evaporation(TIME_DELTA);
    }

    pub fn rain(&mut self, drop_count: u32, drop_size: f64) {
        for _ in 0..drop_count {
            self.add_water_drop(drop_size);
        }
    }

    pub fn create_heightmap(&self) -> HeightMap {
        let mut height_map = HeightMap::new(self.size, 1);
        for y in 0..self.size {
            for x in 0..self.size {
                let pos = [x, y];
                let height = self.get_cell(&pos).get_terrain_height();
                height_map.set(&pos, height);
            }
        }
        height_map
    }

    fn has_water(&self) -> bool {
        for y in 0..self.size {
            for x in 0..self.size {
                if self.get_cell(&[x, y]).has_water() {
                    return true;
                }
            }
        }
        false
    }

    fn update_outflow(&mut self, time_delta: f64) {
        let flow_factor = (time_delta * self.pipe_area * self.gravity) / self.pipe_length;
        for y in 0..self.size {
            for x in 0..self.size {
                self.update_cell_outflow(&[x, y], flow_factor, time_delta);
            }
        }
    }

    fn apply_waterflow(&mut self, time_delta: f64) {
        for y in 0..self.size {
            for x in 0..self.size {
                self.apply_cell_waterflow(&[x, y], time_delta);
            }
        }
    }

    fn update_cell_outflow(&mut self, pos: &[i32; 2], flow_factor: f64, time_delta: f64) {
        let cell = self.get_cell(pos);
        if cell.has_water() {
            let mut new_flow = [0., 0., 0., 0.];
            let mut flow_sum = 0.;
            for dir in &NEIGHBOURS {
                if let Some(nb) = self.get_neighbour(pos, *dir) {
                    let water_delta = cell.get_water_level() - nb.get_water_level();
                    let index: usize = Direction::into(*dir);
                    new_flow[index] = f64::max(0., cell.get_flow(*dir) + flow_factor * water_delta);
                    flow_sum += new_flow[index];
                } else {
                    let water_delta = cell.get_water_level();
                    let index: usize = Direction::into(*dir);
                    new_flow[index] = f64::max(0., cell.get_flow(*dir) + flow_factor * water_delta);
                    flow_sum += new_flow[index];
                }
            }
            let k = f64::min(
                1.,
                (cell.get_water_height() * self.grid_distance[0] * self.grid_distance[1])
                    / (flow_sum * time_delta),
            );
            let cell = self.get_cell_mut(pos);
            for dir in &NEIGHBOURS {
                let index: usize = Direction::into(*dir);
                let scaled_flow = k * new_flow[index];
                cell.set_flow(*dir, scaled_flow);
            }
        }
    }

    fn apply_cell_waterflow(&mut self, pos: &[i32; 2], time_delta: f64) {
        let cell = self.get_cell(pos);
        let mut flow_delta: [f64; 2] = [0., 0.];
        for dir in &NEIGHBOURS {
            if let Some(nb) = self.get_neighbour(pos, *dir) {
                let inflow = nb.get_flow(get_opposite_direction(*dir));
                let outflow = cell.get_flow(*dir);
                match dir {
                    Direction::TOP | Direction::BOTTOM => flow_delta[1] += inflow - outflow,
                    Direction::LEFT | Direction::RIGHT => flow_delta[0] += inflow - outflow,
                }
            }
        }
        let water_delta = (time_delta * (flow_delta[0] + flow_delta[1]))
            / (self.grid_distance[0] * self.grid_distance[1]);
		println!("{}", water_delta);
        if water_delta.abs() > 1e-5 {
            let mut new_velocity = Vector2::from_s(0.);
            for axis in 0..2 {
                let flow_avg = flow_delta[axis] / 2.;
                let d = water_delta / 2.;
                new_velocity[axis] = flow_avg / (d * self.grid_distance[(axis + 1) % 2]);
                new_velocity[axis] = f64::min(new_velocity[axis], self.grid_distance[axis]);
                if new_velocity[axis] * time_delta > self.grid_distance[axis] {
                    warn!("Stability: Product of velocity and time bigger than grid distance: {} > {}", new_velocity[axis] * time_delta, self.grid_distance[axis]);
                }
            }
            let cell = self.get_cell_mut(pos);
            cell.mod_water(water_delta);
            cell.set_velocity(new_velocity);
        } else {
            self.get_cell_mut(pos).set_water(0.);
        }
    }

    fn apply_erosion_deposition(&mut self) {
        for y in 0..self.size {
            for x in 0..self.size {
                if self.get_cell(&[x, y]).has_water() {
                    self.update_cell_normal(&[x, y]);
                    let cell = self.get_cell_mut(&[x, y]);
                    cell.update_transport_capacity(SEDIMENT_CAPACITY_CONSTANT);
                    cell.apply_erosion_deposition(DISSOLVING_CONSTANT, DEPOSITION_CONSTANT);
                }
            }
        }
    }

    fn update_transported_sediment(&mut self, time_delta: f64) {
        for y in 0..self.size {
            for x in 0..self.size {
                let transported_sediment = self.calculate_transported_sediment(&[x, y], time_delta);
                if transported_sediment > 0. {
                    println!("Transported sediment: {}", transported_sediment);
                }
                debug_assert!(!transported_sediment.is_nan());
                self.get_cell_mut(&[x, y])
                    .set_transported_sediment(transported_sediment);
            }
        }
    }

    fn apply_sediment_transportation(&mut self) {
        for y in 0..self.size {
            for x in 0..self.size {
                self.get_cell_mut(&[x, y]).apply_sediment_transportation();
            }
        }
    }

    fn apply_water_evaporation(&mut self, time_delta: f64) {
        let evap_factor = f64::max(0., 1. - EVAPORATION_CONSTANT * time_delta);
        for y in 0..self.size {
            for x in 0..self.size {
                self.get_cell_mut(&[x, y])
                    .apply_water_evaporation(evap_factor);
            }
        }
    }

    fn calculate_transported_sediment(&mut self, pos: &[i32; 2], time_delta: f64) -> f64 {
        const OFFSETS: [[i32; 2]; 4] = [[0, 0], [0, 1], [1, 0], [1, 1]];
        if self.get_cell(pos).has_velocity() {
            let cell_pos = Vector2::new(pos[0] as f64, pos[1] as f64);
            let source_pos =
                cell_pos.add(self.get_cell(pos).get_sediment_source_offset(time_delta));

            let mut neighbour_values: [f64; 4] = [0., 0., 0., 0.];

            let root: [i32; 2] = [
                f64::floor(source_pos.x) as i32,
                f64::floor(source_pos.y) as i32,
            ];
            for (off, val) in OFFSETS.iter().zip(neighbour_values.iter_mut()) {
                let position = [root[0] + off[0], root[1] + off[1]];
                if self.in_boundary(&position) {
                    *val = self.get_cell(&position).get_suspended_sediment();
                }
            }
            interpolate([source_pos.x, source_pos.y], neighbour_values)
        } else {
            0.
        }
    }

    fn update_cell_normal(&mut self, pos: &[i32; 2]) {
        let cell = self.get_cell(pos);
        let mut nb_height: [f64; 4] = [
            cell.get_water_level(),
            cell.get_water_level(),
            cell.get_water_level(),
            cell.get_water_level(),
        ];
        for dir in &NEIGHBOURS {
            if let Some(nb) = self.get_neighbour(pos, *dir) {
                let index: usize = (*dir).into();
                nb_height[index] = nb.get_water_level();
            }
        }
        let mut slope: [f64; 2] = [0., 0.];
        for axis in 0..2 {
            slope[axis] = nb_height[axis] - nb_height[axis + 2]; // top/right - bottom/left
        }
        let normal = normalize(Vector3::<f64>::new(slope[0], slope[1], 2.));
        self.get_cell_mut(pos).set_normal(normal);
    }

    fn add_water_drop(&mut self, size: f64) {
        let pos = self.get_random_pos();
        self.get_cell_mut(&pos).mod_water(size);
        for dir in &NEIGHBOURS {
            if let Some(nb) = self.get_neighbour_mut(&pos, *dir) {
                nb.mod_water(size / 4.);
            }
        }
    }

    fn get_random_pos(&mut self) -> [i32; 2] {
        [
            self.rng.gen_range(0, self.size),
            self.rng.gen_range(0, self.size),
        ]
    }

    fn in_boundary(&self, pos: &[i32; 2]) -> bool {
        pos[0] >= 0 && pos[0] < self.size && pos[1] >= 0 && pos[1] < self.size
    }

    fn pos_to_index(&self, pos: &[i32; 2]) -> usize {
        debug_assert!(self.in_boundary(pos));
        let index = (pos[1] * self.size + pos[0]) as usize;
        debug_assert!(index < self.cells.len());
        index
    }

    fn get_cell(&self, pos: &[i32; 2]) -> &Cell {
        &self.cells[self.pos_to_index(pos)]
    }
    fn get_cell_mut(&mut self, pos: &[i32; 2]) -> &mut Cell {
        let index = self.pos_to_index(pos);
        &mut self.cells[index]
    }

    fn get_neighbour(&self, pos: &[i32; 2], dir: Direction) -> Option<&Cell> {
        let nb_pos = get_neighbour_pos(pos, dir);
        let nb_index = (nb_pos[1] * self.size + nb_pos[0]) as usize;
        if nb_index >= self.cells.len() {
            None
        } else {
            Some(&self.cells[nb_index])
        }
    }
    fn get_neighbour_mut(&mut self, pos: &[i32; 2], dir: Direction) -> Option<&mut Cell> {
        let nb_pos = get_neighbour_pos(pos, dir);
        let nb_index = (nb_pos[1] * self.size + nb_pos[0]) as usize;
        if nb_index >= self.cells.len() {
            None
        } else {
            Some(&mut self.cells[nb_index])
        }
    }
}

fn interpolate(p: [f64; 2], reference: [f64; 4]) -> f64 {
    let anchor = [p[0].floor() as i32, p[1].floor() as i32];
    let a = anchor[0] as f64 + 1. - p[0];
    let b = p[0] - anchor[0] as f64;
    let r_1 = a * reference[0] + b * reference[1];
    let r_2 = a * reference[2] + b * reference[3];
    let c = anchor[1] as f64 + 1. - p[1];
    let d = p[1] - anchor[1] as f64;
    c * r_1 + d * r_2
}
