use std::collections::HashSet;

use cgmath::Rotation3;

use crate::instance::Instance;
use crate::world::Direction::{_X, _Y, _Z, X, Y, Z};

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

impl Direction {
    fn random() -> Direction {
        let x = rand::random::<f32>();
        if x < 1.0 / 6.0 {
            return Direction::X;
        }
        if x < 2.0 / 6.0 {
            return Direction::Y;
        }
        if x < 3.0 / 6.0 {
            return Direction::Z;
        }
        if x < 4.0 / 6.0 {
            return Direction::_X;
        }
        if x < 5.0 / 6.0 {
            return Direction::_Y;
        }
        Direction::_Z
    }

    fn random_perpendicular(self) -> Direction {
        use Direction::*;
        let options = match self {
            X | _X => (Y, _Y, Z, _Z),
            Y | _Y => (X, _X, Z, _Z),
            Z | _Z => (Y, _Y, X, _X),
        };
        let val = rand::random::<f32>();
        if val < 0.25 {
            return options.0;
        }
        if val < 0.5 {
            return options.1;
        }
        if val < 0.75 {
            return options.2;
        }
        options.3
    }
}

#[derive(Copy, Clone, Debug)]
struct Block {
    pipe_type: PipeType,
    direction: Direction, // direction of output pipe
    position: (u32, u32, u32),
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
            max_x_block: 10,
            max_y_block: 8,
            max_z_block: 8,
            turn_probability: 0.1,
            stop_probability: 0.1,
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

        dbg!(block);

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

    pub fn add_debug_pipe(&mut self, pipe_type: PipeType, position: (u32, u32, u32), direction: Direction) {
        let block = Block { pipe_type, direction, position };

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
                rand::random_range(0..self.max_x_block),
                rand::random_range(0..self.max_y_block),
                rand::random_range(0..self.max_z_block),
            );
            if !self.occupied_blocks.contains(&position) {
                break position;
            }
        };

        Block {
            pipe_type: PipeType::I, // always start with I for eases of impl
            direction: Direction::random(),
            position,
        }
    }

    fn next_block(&self) -> Block {
        use Direction::*;
        let last_block = self.last_block.as_ref().unwrap();

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
                position,
                direction: last_block.direction.random_perpendicular(),
                pipe_type: PipeType::L,
            }
        } else {
            Block {
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

        Instance { position, rotation }
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
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(90.0))
                    * cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(deg))
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
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(180.0))
                    * cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(deg))
            }
            Z => {
                let deg = match last_block_dir {
                    _X => 0.0,
                    _Y => 90.0,
                    X => 180.0,
                    Y => -90.0,
                    _ => panic!("Invalid direction"),
                };
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(90.0))
                    * cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(deg))
            }
            _Z => {
                let deg = match last_block_dir {
                    _X => 0.0,
                    _Y => 90.0,
                    X => 180.0,
                    Y => -90.0,
                    _ => panic!("Invalid direction"),
                };
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(-90.0))
                    * cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(deg))
            }
        };

        // TODO add model offset to position

        Instance { position, rotation }
    }
}
