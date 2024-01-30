use crate::Float;

pub mod sin_wt;

pub trait Osc: Send {
    fn get_sample(&mut self) -> Float;

    fn set_frequency(&mut self, frequency: Float);
}
