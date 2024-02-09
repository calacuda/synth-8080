use super::Envelope;
use crate::{Float, SAMPLE_RATE};
use anyhow::{bail, Result};

pub const N_INPUTS: u8 = 5;
pub const N_OUTPUTS: u8 = 1;

pub const ATTACK_IN: u8 = 0; // sets attack speed in seconds
pub const DECAY_IN: u8 = 1; // sets decay 1 speed in seconds
pub const DECAY_THRESHOLD: u8 = 2; // sets the threshold between decay 1 & 2 in amplitude

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Phase {
    Neutural,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone)]
pub struct Filter {
    // pressed: bool,
    phase: Phase,
    // i: usize,
    env: Float,
    pub decay_speed: Float,
    pub attack_speed: Float,
    pub threshold: Float,
    pub decay: Float,
    pub attack: Float,
    pub release: Float,
    sample_rate: Float,
    pub pressed: bool,
    release_threshold: Float,
}

impl Filter {
    pub fn new() -> Self {
        let sample_rate = SAMPLE_RATE as Float;
        let attack_speed = 0.01;
        let attack = 1.0 / (sample_rate * attack_speed);
        let decay_speed = 0.1;
        let decay = -1.0 / (sample_rate * decay_speed);

        Self {
            // pressed: false,
            phase: Phase::Neutural,
            // i: 0,
            env: 0.0,
            decay,
            attack,
            threshold: 0.9,
            sample_rate,
            attack_speed,
            decay_speed,
            release: -1.0 / (sample_rate * 0.0001),
            pressed: false,
            release_threshold: 0.05,
        }
    }

    fn set_attack(&mut self, atk_speed: Float) {
        if atk_speed != self.attack_speed {
            self.attack_speed = atk_speed;
            self.attack = 1.0 / (self.sample_rate * atk_speed);
        }
    }

    fn set_decay(&mut self, decay_speed: Float) {
        if decay_speed != self.decay_speed {
            self.decay_speed = decay_speed;
            self.decay = -1.0 / (self.sample_rate * decay_speed);
        }
    }

    fn set_threshold(&mut self, threshold: Float) {
        self.threshold = threshold;
    }

    fn internal_update_phase(&mut self) {
        if self.phase == Phase::Attack && self.env >= 1.0 {
            self.phase = Phase::Decay;
            self.env = 1.0;
            // info!("chaning phase to => {:?}", self.phase);
        } else if self.phase == Phase::Decay && self.env <= self.threshold {
            self.phase = Phase::Sustain;
            // info!("chaning phase to => {:?}", self.phase);
        } else if self.phase == Phase::Sustain && self.env <= self.release_threshold {
            self.phase = Phase::Release;
            // info!("chaning phase to => {:?}", self.phase);
        } else if self.phase == Phase::Release && self.env <= 0.0 {
            self.phase = Phase::Neutural;
            self.env = 0.0;
            // info!("chaning phase to => {:?}", self.phase);
        }
    }
}

impl Envelope for Filter {
    fn get_env(&mut self) -> Float {
        self.env
    }

    fn set_env(&mut self, env: Float) {
        self.env = env;
    }

    fn get_step(&mut self) -> Float {
        match self.phase {
            Phase::Attack => self.attack,
            Phase::Decay => self.decay,
            Phase::Sustain => 0.0,
            Phase::Release => self.release,
            Phase::Neutural => 0.0,
        }
    }

    fn update_phase(&mut self) {
        self.internal_update_phase()
    }

    fn open_filter(&mut self, samples: Vec<Float>) {
        let sample: Float = samples.iter().sum::<Float>().tanh();

        if self.pressed && sample <= 0.75 {
            // info!("release");
            self.phase = Phase::Release;
            self.pressed = false;
        } else if !self.pressed && self.phase == Phase::Neutural && sample >= 0.75 {
            // info!("pressed");
            self.phase = Phase::Attack;
            self.pressed = true;
        }
    }

    fn take_input(&mut self, input: u8, samples: Vec<Float>) -> Result<()> {
        let sample: Float = samples.iter().sum::<Float>().tanh();

        match input {
            // attack in
            0 => self.set_attack(sample),
            // decay speed in
            1 => self.set_decay(sample),
            // decay_threshold in
            2 => self.set_threshold(sample),
            n => bail!("{n} is not a valid input for the ADBDR filter."),
        }

        Ok(())
    }
}

// #[derive(Debug, Clone)]
// pub struct ADSRModule {
//     audio_in: Arc<Mutex<Float>>,
//     envelope: Arc<Mutex<ADSREnvelope>>,
//     outputs: Arc<Mutex<Vec<Connection>>>,
//     routing_table: Router,
//     id: u8,
// }
//
// impl ADSRModule {
//     pub fn new(routing_table: Router, id: u8) -> Self {
//         Self {
//             envelope: Arc::new(Mutex::new(ADSREnvelope::new())),
//             audio_in: Arc::new(Mutex::new(0.0)),
//             outputs: Arc::new(Mutex::new(Vec::new())),
//             routing_table,
//             id,
//         }
//     }
// }
//
// impl Module for ADSRModule {
//     fn start(&self) -> Result<JoinHandle<()>> {
//         let router = self.routing_table.clone();
//
//         // audio output
//         let outs = self.outputs.clone();
//
//         // inputs
//         let env = self.envelope.clone();
//         let env_2 = self.envelope.clone();
//         let env_3 = self.envelope.clone();
//         let env_4 = self.envelope.clone();
//         let env_5 = self.envelope.clone();
//         let id = self.id as usize;
//         let audio = self.audio_in.clone();
//         let audio_2 = self.audio_in.clone();
//
//         Ok(spawn(async move {
//             // prepare call back for event loop
//             let ins: &Vec<ModuleIn> = (*router)
//                 .0
//                 .get(id)
//                 .expect("this ADBDR Envelope Module was not found in the routing table struct.")
//                 .as_ref();
//             let gen_sample: Box<dyn FnMut() -> Float + Send> =
//                 Box::new(move || (*audio.lock().unwrap()) * env.lock().unwrap().step());
//             let outputs = vec![(outs, gen_sample)];
//
//             // get inputs and update values
//             let set_atk: Box<dyn FnMut(Vec<Float>) + Send> =
//                 Box::new(move |samples: Vec<Float>| {
//                     let atk = samples.iter().sum::<Float>() / (samples.len() as Float);
//                     let mut e = env_2.lock().unwrap();
//                     (*e).set_attack(atk);
//                 });
//             let set_decay: Box<dyn FnMut(Vec<Float>) + Send> =
//                 Box::new(move |samples: Vec<Float>| {
//                     let decay = samples.iter().sum::<Float>() / (samples.len() as Float);
//                     let mut e = env_3.lock().unwrap();
//                     (*e).set_decay(decay);
//                 });
//             let set_threshold: Box<dyn FnMut(Vec<Float>) + Send> =
//                 Box::new(move |samples: Vec<Float>| {
//                     let decay = samples.iter().sum::<Float>() / (samples.len() as Float);
//                     let mut e = env_4.lock().unwrap();
//                     (*e).set_threshold(decay);
//                 });
//             let set_pressed: Box<dyn FnMut(Vec<Float>) + Send> =
//                 Box::new(move |samples: Vec<Float>| {
//                     let in_val = samples.iter().sum::<Float>() / (samples.len() as Float);
//                     let mut e = env_5.lock().unwrap();
//
//                     // info!("set_pressed => in_val {in_val}, phase => {:?}", e.phase);
//                     if e.pressed && in_val <= 0.75 {
//                         // info!("release");
//                         (*e).phase = Phase::Release;
//                         (*e).pressed = false;
//                     } else if !e.pressed && e.phase == Phase::Neutural && in_val >= 0.75 {
//                         // info!("pressed");
//                         (*e).phase = Phase::Attack;
//                         (*e).pressed = true;
//                     }
//                 });
//             let set_audio: Box<dyn FnMut(Vec<Float>) + Send> =
//                 Box::new(move |samples: Vec<Float>| {
//                     let audio = samples.iter().sum::<Float>().tanh();
//                     let mut a = audio_2.lock().unwrap();
//                     (*a) = audio;
//                 });
//
//             let inputs = vec![
//                 (&ins[AUDIO_IN as usize], set_audio),
//                 (&ins[ENVELOPE_IN as usize], set_pressed),
//                 (&ins[ATTACK_IN as usize], set_atk),
//                 (&ins[DECAY_IN as usize], set_decay),
//                 (&ins[DECAY_THRESHOLD as usize], set_threshold),
//             ];
//
//             // info!("adsr filter event loop is about to start");
//             // start the event loop
//             event_loop(router.clone(), inputs, outputs).await;
//         }))
//     }
//
//     fn connect(&self, connection: Connection) -> Result<()> {
//         ensure!(
//             connection.src_output < N_OUTPUTS,
//             "invalid output selection"
//         );
//         ensure!(
//             !self.outputs.lock().unwrap().contains(&connection),
//             "module already connected"
//         );
//         self.outputs.lock().unwrap().push(connection);
//
//         // info!(
//         //     "connected output: {}, of module: {}, to input: {}, of module: {}",
//         //     connection.src_output,
//         //     connection.src_module,
//         //     connection.dest_input,
//         //     connection.dest_module
//         // );
//
//         Ok(())
//     }
//
//     fn disconnect(&self, connection: Connection) -> Result<()> {
//         ensure!(
//             connection.src_output < N_OUTPUTS,
//             "invalid output selection"
//         );
//         ensure!(
//             self.outputs.lock().unwrap().contains(&connection),
//             "module not connected"
//         );
//         self.outputs
//             .lock()
//             .unwrap()
//             .retain(|out| *out != connection);
//
//         Ok(())
//     }
// }