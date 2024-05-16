use super::Envelope;
use crate::Float;

pub struct Filter {
    open: bool,
}

impl Filter {
    pub fn new() -> Self {
        Self { open: false }
    }
}

impl Envelope for Filter {
    fn get_env(&mut self) -> Float {
        self.open as u8 as Float
    }

    fn set_env(&mut self, _env: Float) {}

    fn get_step(&mut self) -> crate::Float {
        0.0
    }

    fn update_phase(&mut self) {}

    fn take_input(&mut self, _input: u8, _values: Vec<crate::Float>) -> anyhow::Result<()> {
        Ok(())
    }

    fn open_filter(&mut self, samples: Vec<crate::Float>) -> bool {
        self.open = samples.iter().sum::<Float>().tanh() >= 0.75;
        self.open
    }

    fn pressed(&mut self) -> bool {
        false
    }
}
