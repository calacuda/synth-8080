use crate::{
    common::{event_loop, Connection, Module},
    router::{ModuleIn, Router},
    Float, SAMPLE_RATE,
};
use anyhow::{ensure, Result};
use std::sync::{Arc, Mutex};
use tokio::task::{spawn, JoinHandle};
use tracing::info;

pub const N_INPUTS: u8 = 6;
pub const N_OUTPUTS: u8 = 1;

pub const AUDIO_IN: u8 = 0; // the audio this filter is enveloping
pub const ENVELOPE_IN: u8 = 1; // the gate signal that triggers the filter
pub const ATTACK_IN: u8 = 2; // sets attack speed in seconds
pub const DECAY_1_IN: u8 = 3; // sets decay 1 speed in seconds
pub const DECAY_THRESHOLD: u8 = 4; // sets the threshold between decay 1 & 2 in amplitude
pub const DECAY_2_IN: u8 = 5; // sets decay 2 speed in seconds

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Phase {
    Neutural,
    Attack,
    Decay1,
    Decay2,
    Release,
}

#[derive(Debug, Clone)]
pub struct ADBDREnvelope {
    // pressed: bool,
    phase: Phase,
    // i: usize,
    env: Float,
    pub decay_1_speed: Float,
    pub decay_2_speed: Float,
    pub attack_speed: Float,
    pub threshold: Float,
    pub decay_1: Float,
    pub decay_2: Float,
    pub attack: Float,
    pub release: Float,
    sample_rate: Float,
    pub pressed: bool,
    release_threshold: Float,
}

impl ADBDREnvelope {
    pub fn new() -> Self {
        let sample_rate = SAMPLE_RATE as Float;
        let attack_speed = 0.01;
        let attack = 1.0 / (sample_rate * attack_speed);
        let decay_1_speed = 0.1;
        let decay_1 = -1.0 / (sample_rate * decay_1_speed);
        let decay_2_speed = 20.0;
        let decay_2 = -1.0 / (sample_rate * decay_2_speed);

        Self {
            // pressed: false,
            phase: Phase::Neutural,
            // i: 0,
            env: 0.0,
            decay_1,
            decay_2,
            attack,
            threshold: 0.9,
            sample_rate,
            attack_speed,
            decay_1_speed,
            decay_2_speed,
            release: -1.0 / (sample_rate * 0.0001),
            pressed: false,
            release_threshold: 0.05,
        }
    }

    // fn atk_shift(&mut self) -> Float {
    //     1.0 / (self.sample_rate * self.attack)
    // }
    fn set_attack(&mut self, atk_speed: Float) {
        if atk_speed != self.attack_speed {
            self.attack_speed = atk_speed;
            self.attack = 1.0 / (self.sample_rate * atk_speed);
        }
    }

    fn set_decay_1(&mut self, decay_1_speed: Float) {
        if decay_1_speed != self.decay_1_speed {
            self.decay_1_speed = decay_1_speed;
            self.decay_1 = -1.0 / (self.sample_rate * decay_1_speed);
        }
    }

    fn set_decay_2(&mut self, decay_2_speed: Float) {
        if decay_2_speed != self.decay_2_speed {
            self.decay_2_speed = decay_2_speed;
            self.decay_2 =
                -(self.threshold - self.release_threshold) / (self.sample_rate * decay_2_speed);
        }
    }

    pub fn step(&mut self) -> Float {
        self.env += match self.phase {
            Phase::Attack => self.attack,
            Phase::Decay1 => self.decay_1,
            Phase::Decay2 => self.decay_2,
            Phase::Release => self.release,
            Phase::Neutural => 0.0,
        };

        // self.i += 1;
        self.update_phase();

        self.env
    }

    fn update_phase(&mut self) {
        if self.phase == Phase::Attack && self.env >= 1.0 {
            self.phase = Phase::Decay1;
            self.env = 1.0;
            // info!("chaning phase to => {:?}", self.phase);
        } else if self.phase == Phase::Decay1 && self.env <= self.threshold {
            self.phase = Phase::Decay2;
            // info!("chaning phase to => {:?}", self.phase);
        } else if self.phase == Phase::Decay2 && self.env <= self.release_threshold {
            self.phase = Phase::Release;
            // info!("chaning phase to => {:?}", self.phase);
        } else if self.phase == Phase::Release && self.env <= 0.0 {
            self.phase = Phase::Neutural;
            self.env = 0.0;
            // info!("chaning phase to => {:?}", self.phase);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ADBDRModule {
    audio_in: Arc<Mutex<Float>>,
    envelope: Arc<Mutex<ADBDREnvelope>>,
    outputs: Arc<Mutex<Vec<Connection>>>,
    routing_table: Router,
    id: u8,
}

impl ADBDRModule {
    pub fn new(routing_table: Router, id: u8) -> Self {
        Self {
            envelope: Arc::new(Mutex::new(ADBDREnvelope::new())),
            audio_in: Arc::new(Mutex::new(0.0)),
            outputs: Arc::new(Mutex::new(Vec::new())),
            routing_table,
            id,
        }
    }
}

impl Module for ADBDRModule {
    fn start(&self) -> Result<JoinHandle<()>> {
        let router = self.routing_table.clone();

        // audio output
        let outs = self.outputs.clone();

        // inputs
        let env = self.envelope.clone();
        let env_2 = self.envelope.clone();
        let env_3 = self.envelope.clone();
        let env_4 = self.envelope.clone();
        let env_5 = self.envelope.clone();
        let env_6 = self.envelope.clone();
        let id = self.id as usize;
        let audio = self.audio_in.clone();
        let audio_2 = self.audio_in.clone();

        Ok(spawn(async move {
            // prepare call back for event loop
            let ins: &Vec<ModuleIn> = (*router)
                .0
                .get(id)
                .expect("this ADBDR Envelope Module was not found in the routing table struct.")
                .as_ref();
            let gen_sample: Box<dyn FnMut() -> Float + Send> =
                Box::new(move || (*audio.lock().unwrap()) * env.lock().unwrap().step());
            let outputs = vec![(outs, gen_sample)];

            // get inputs and update values
            let set_atk: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let atk = samples.iter().sum::<Float>() / (samples.len() as Float);
                    let mut e = env_2.lock().unwrap();
                    (*e).set_attack(atk);
                });
            let set_decay_1: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let decay = samples.iter().sum::<Float>() / (samples.len() as Float);
                    let mut e = env_3.lock().unwrap();
                    (*e).set_decay_1(decay);
                });
            let set_decay_2: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let decay = samples.iter().sum::<Float>() / (samples.len() as Float);
                    let mut e = env_4.lock().unwrap();
                    (*e).set_decay_2(decay);
                });
            let set_decay_threshold: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let threshold = samples.iter().sum::<Float>() / (samples.len() as Float);
                    let mut e = env_5.lock().unwrap();
                    (*e).threshold = threshold;
                });
            let set_pressed: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let in_val = samples.iter().sum::<Float>() / (samples.len() as Float);
                    let mut e = env_6.lock().unwrap();

                    // info!("set_pressed => in_val {in_val}, phase => {:?}", e.phase);
                    if e.pressed && in_val <= 0.75 {
                        // info!("release");
                        (*e).phase = Phase::Release;
                        (*e).pressed = false;
                    } else if !e.pressed && e.phase == Phase::Neutural && in_val >= 0.75 {
                        // info!("pressed");
                        (*e).phase = Phase::Attack;
                        (*e).pressed = true;
                    }
                });
            let set_audio: Box<dyn FnMut(Vec<Float>) + Send> =
                Box::new(move |samples: Vec<Float>| {
                    let audio = samples.iter().sum::<Float>() / (samples.len() as Float);
                    let mut a = audio_2.lock().unwrap();
                    (*a) = audio;
                });

            let inputs = vec![
                (&ins[AUDIO_IN as usize], set_audio),
                (&ins[ENVELOPE_IN as usize], set_pressed),
                (&ins[ATTACK_IN as usize], set_atk),
                (&ins[DECAY_1_IN as usize], set_decay_1),
                (&ins[DECAY_THRESHOLD as usize], set_decay_threshold),
                (&ins[DECAY_2_IN as usize], set_decay_2),
            ];

            // start the event loop
            event_loop(router.clone(), inputs, outputs).await;
        }))
    }

    fn connect(&self, connection: Connection) -> Result<()> {
        ensure!(
            connection.src_output < N_OUTPUTS,
            "invalid output selection"
        );
        ensure!(
            !self.outputs.lock().unwrap().contains(&connection),
            "module already connected"
        );
        self.outputs.lock().unwrap().push(connection);

        info!(
            "connected output: {}, of module: {}, to input: {}, of module: {}",
            connection.src_output,
            connection.src_module,
            connection.dest_input,
            connection.dest_module
        );

        Ok(())
    }

    fn disconnect(&self, connection: Connection) -> Result<()> {
        ensure!(
            connection.src_output < N_OUTPUTS,
            "invalid output selection"
        );
        ensure!(
            self.outputs.lock().unwrap().contains(&connection),
            "module not connected"
        );
        self.outputs
            .lock()
            .unwrap()
            .retain(|out| *out != connection);

        Ok(())
    }
}
