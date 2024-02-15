use super::Envelope;
use crate::Float;

pub struct Filter {}

impl Filter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Envelope for Filter {
    fn get_env(&mut self) -> crate::Float {
        1.0
    }

    fn set_env(&mut self, _env: crate::Float) {}

    fn get_step(&mut self) -> crate::Float {
        0.0
    }

    fn update_phase(&mut self) {}

    fn take_input(&mut self, _input: u8, _values: Vec<crate::Float>) -> anyhow::Result<()> {
        Ok(())
    }

    fn open_filter(&mut self, samples: Vec<crate::Float>) -> bool {
        let input = samples.iter().sum::<Float>();

        input >= 0.75
    }
}
