use std::collections::HashSet;
use rand::seq::IndexedRandom;
use cgmath::Rotation3;

use crate::instance::Instance;


macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {[ ($r as f32) / 256.0, ($g as f32) / 256.0, ($b as f32) / 256.0 ]};
}

const COLOR: &[[f32; 3]] = &[
    rgb!(116, 222, 215),
    rgb!(255, 0, 0),
    rgb!(247, 104, 31),
    rgb!(75, 151, 160),
    rgb!(254, 211, 86),
    rgb!(250, 231, 231),
    rgb!(132, 123, 14),
    rgb!(251, 155, 72),
    rgb!(14, 169, 30),
    rgb!(158, 235, 189),
    rgb!(2, 143, 146)
];

fn random_color() -> &'static [f32; 3] {
    let mut rng = rand::rng();
    COLOR.choose(&mut rng).unwrap()
}

#[derive(Copy, Clone, Debug)]
pub enum PipeType {
    I,
    L,
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    X,
    Y,
    Z,
    _X,
    _Y,
    _Z,
}

const ALL_DIRECTIONS: [Direction; 6] = [Direction::X, Direction::Y, Direction::Z, Direction::_X, Direction::_Y, Direction::_Z];
const PERPENDICULAR_X: [Direction; 4] = [Direction::Y, Direction::_Y, Direction::Z, Direction::_Z];
const PERPENDICULAR_Y: [Direction; 4] = [Direction::X, Direction::_X, Direction::Z, Direction::_Z];
const PERPENDICULAR_Z: [Direction; 4] = [Direction::Y, Direction::_Y, Direction::X, Direction::_X];

impl Direction {
    fn random() -> Direction {
        let mut rng = rand::rng();
        *ALL_DIRECTIONS.choose(&mut rng).unwrap()
    }

    fn random_perpendicular(self) -> Direction {
        use Direction::*;
        let options = match self {
            X | _X => &PERPENDICULAR_X,
            Y | _Y => &PERPENDICULAR_Y,
            Z | _Z => &PERPENDICULAR_Z,
        };
        let mut rng = rand::rng();
        *options.choose(&mut rng).unwrap()
    }
}

#[derive(Copy, Clone, Debug)]
struct Block {
    pipe_type: PipeType,
    direction: Direction, // direction of output pipe
    position: (u32, u32, u32),
    color: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct World {
    max_x_block: u32,
    max_y_block: u32,
    max_z_block: u32,

    turn_probability: f32,
    stop_probability: f32,

    i_pipe_instances: Vec<Instance>,
    l_pipe_instances: Vec<Instance>,

    occupied_blocks: HashSet<(u32, u32, u32)>,
    last_block: Option<Block>,
}

const WORLD_X: u32 = 30;
const WORLD_Y: u32 = 30;
const WORLD_Z: u32 = 30;
const TURN_PROBABILITY: f32 = 0.3;
const STOP_PROBABILITY: f32 = 0.0;

/**
    World coordinate system
    X: to the right
    Y: to the top
    Z: pop out of screen

            Y
            |
            |
            |
            |__________ X
           /
          /
       Z /

    I pipe: follow Y
    L pipe: follow Y and X
*/
impl World {
    pub fn new() -> Self {
        Self {
            // TODO consider scale to screen ratio
            max_x_block: WORLD_X,
            max_y_block: WORLD_Y,
            max_z_block: WORLD_Z,
            turn_probability: TURN_PROBABILITY,
            stop_probability: STOP_PROBABILITY,
            i_pipe_instances: vec![],
            l_pipe_instances: vec![],
            occupied_blocks: HashSet::with_capacity(128),
            last_block: None,
        }
    }

    pub fn get_I_pipe_instances(&self) -> &[Instance] {
        self.i_pipe_instances.as_slice()
    }

    pub fn get_L_pipe_instances(&self) -> &[Instance] {
        self.l_pipe_instances.as_slice()
    }

    pub fn add_pipe(&mut self) {
        let block = if rand::random::<f32>() < self.stop_probability || self.last_block.is_none() {
            self.random_block()
        } else {
            self.next_block()
        };

        match block.pipe_type {
            PipeType::I => {
                let instance = self.i_instance_at_block(&block);
                self.i_pipe_instances.push(instance);
            }
            PipeType::L => {
                let instance = self.l_instance_at_block(&block);
                self.l_pipe_instances.push(instance);
            }
        };

        self.occupied_blocks.insert(block.position);
        self.last_block = Some(block);
    }

    pub fn add_debug_pipe(&mut self, pipe_type: PipeType, position: (u32, u32, u32), direction: Direction, color: [f32; 3]) {
        let block = Block { pipe_type, direction, position, color };

        match block.pipe_type {
            PipeType::I => {
                let instance = self.i_instance_at_block(&block);
                self.i_pipe_instances.push(instance);
            }
            PipeType::L => {
                let instance = self.l_instance_at_block(&block);
                self.l_pipe_instances.push(instance);
            }
        };

        self.occupied_blocks.insert(block.position);
        self.last_block = Some(block);
    }

    fn random_block(&self) -> Block {
        let position = loop {
            let position = (
                rand::random_range(0..self.max_x_block / 2),
                rand::random_range(0..self.max_y_block / 2),
                rand::random_range(0..self.max_z_block / 2),
            );
            if !self.occupied_blocks.contains(&position) {
                break position;
            }
        };

        Block {
            pipe_type: PipeType::I, // always start with I for eases of impl
            direction: Direction::random(),
            color: *random_color(),
            position,
        }
    }

    fn next_block(&self) -> Block {
        use Direction::*;
        let last_block = self.last_block.as_ref().unwrap();
        let color = last_block.color;

        let position = match last_block.direction {
            X => (last_block.position.0 + 1, last_block.position.1, last_block.position.2),
            Y => (last_block.position.0, last_block.position.1 + 1, last_block.position.2),
            Z => (last_block.position.0, last_block.position.1, last_block.position.2 + 1),
            _X => (last_block.position.0 - 1, last_block.position.1, last_block.position.2),
            _Y => (last_block.position.0, last_block.position.1 - 1, last_block.position.2),
            _Z => (last_block.position.0, last_block.position.1, last_block.position.2 - 1),
        };

        // position is occupied, or out of the world dimension
        if !self.is_position_valid(&position) {
            return self.random_block();
        }

        if rand::random::<f32>() < self.turn_probability {
            Block {
                color,
                position,
                direction: last_block.direction.random_perpendicular(),
                pipe_type: PipeType::L,
            }
        } else {
            Block {
                color,
                position,
                direction: last_block.direction,
                pipe_type: PipeType::I,
            }
        }
    }

    fn is_position_valid(&self, position: &(u32, u32, u32)) -> bool {
        if position.0 > self.max_x_block
            || position.1 > self.max_y_block
            || position.2 > self.max_z_block
            || self.occupied_blocks.contains(position)
        {
            return false;
        }
        true
    }

    fn i_instance_at_block(&self, block: &Block) -> Instance {
        use Direction::*;
        let p = block.position;
        let position = (p.0 as f32, p.1 as f32, p.2 as f32).into();

        let rotation = match block.direction {
            Y | _Y => cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(0.0)),
            X | _X => cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(-90.0)),
            Z | _Z => cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(90.0)),
        };

        // TODO add model offset to position

        Instance { position, rotation, color: block.color }
    }

    fn l_instance_at_block(&self, block: &Block) -> Instance {
        use Direction::*;
        let last_block_dir = self.last_block.as_ref().unwrap().direction;
        let p = block.position;
        let position = (p.0 as f32, p.1 as f32, p.2 as f32).into();

        let rotation: cgmath::Quaternion<f32> = match block.direction {
            X => {
                let deg = match last_block_dir {
                    _Y => 0.0,
                    _Z => 90.0,
                    Y => 180.0,
                    Z => -90.0,
                    _ => panic!("Invalid direction"),
                };
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(deg))
            }
            _X => {
                let deg = match last_block_dir {
                    _Y => 0.0,
                    _Z => 90.0,
                    Y => 180.0,
                    Z => -90.0,
                    _ => panic!("Invalid direction"),
                };
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(deg)) *
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(90.0))
            }
            Y => {
                let deg = match last_block_dir {
                    _X => 0.0,
                    _Z => -90.0,
                    X => 180.0,
                    Z => 90.0,
                    _ => panic!("Invalid direction"),
                };
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(deg))
            }
            _Y => {
                let deg = match last_block_dir {
                    _X => 0.0,
                    _Z => -90.0,
                    X => 180.0,
                    Z => 90.0,
                    _ => panic!("Invalid direction"),
                };
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(deg)) *
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(180.0))
            }
            Z => {
                let deg = match last_block_dir {
                    _X => 0.0,
                    _Y => 90.0,
                    X => 180.0,
                    Y => -90.0,
                    _ => panic!("Invalid direction"),
                };
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(deg)) *
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(90.0))
            }
            _Z => {
                let deg = match last_block_dir {
                    _X => 0.0,
                    _Y => 90.0,
                    X => 180.0,
                    Y => -90.0,
                    _ => panic!("Invalid direction"),
                };
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(deg)) *
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(-90.0))
            }
        };

        // TODO add model offset to position

        Instance { position, rotation, color: block.color }
    }
}
