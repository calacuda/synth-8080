use std::sync::Arc;

use anyhow::Result;
use synth_8080::{
    chorus,
    controller::{midi::MIDIControls, Controller},
    mk_synth, start_logging, AudioGen,
};
use synth_8080_lib::ModuleType;
use tokio::spawn;
use tracing::*;

fn start_midi(synth: Arc<Controller>) -> Result<MIDIControls> {
    let mut midi_con = MIDIControls::new(synth)?;
    midi_con.connect_default()?;

    Ok(midi_con)
}

#[tokio::main]
async fn main() {
    if let Err(e) = start_logging() {
        eprintln!("failed ot start logging, you wont see logs. {e}");
    }

    // let modules = default_modules();
    let modules = vec![
        ModuleType::MCO,
        ModuleType::Chorus,
        // ModuleType::Output,
        ModuleType::Lfo,
    ];

    let (synth, (stream_handle, audio)) = match mk_synth(&modules).await {
        Ok(synth) => synth,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    // let base_graph = mk_base_graph(&modules);

    let audio_gen = AudioGen {
        controller: synth.clone(),
    };

    let audio_gen_thread = spawn(async { audio_gen.await });
    // info!("starting audio stream");
    stream_handle.play_raw(audio).unwrap();
    // info!("audio stream started");

    // let midi_con = MIDIControls::new(synth.clone());

    let _midi_con = match start_midi(synth.clone()) {
        Err(e) => {
            error!("No MIDI for you! {e}");
            None
        }
        Ok(midi) => {
            info!("MIDI started");
            Some(midi)
        }
    };

    _ = synth.connect(1, 0, 2, chorus::AUDIO_INPUT);
    _ = synth.connect(2, 0, 0, 0);

    {
        synth.output.lock().unwrap().set_volume(0.5);
        // synth.modules.lock().unwrap().mco[0].set_volume(0.5);
    };

    // TODO: read serial input in a loop
    if let Err(e) = audio_gen_thread.await {
        error!("failed to start synth: {e}");
    };
}
