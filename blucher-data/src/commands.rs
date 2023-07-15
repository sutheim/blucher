use bincode::{ Decode, Encode };

#[derive(Encode, Decode)]
pub enum Command {
    SetThrust {
        thrust: f32,
    },
}

#[derive(Encode, Decode)]
pub enum SystemReport {
    Locomotion {
        thrust: f32,
        direction: f32
    }
}
