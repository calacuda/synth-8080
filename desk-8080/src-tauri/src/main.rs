// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::*;
use std::sync::Arc;
use synth_8080::{
    chorus,
    common::notes::Note,
    controller::{midi::MIDIControls, Controller},
    default_connections, envelope, mk_synth, start_logging, AudioGen,
};
use tauri::{async_runtime::spawn, State};
// use tokio::spawn;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// fn mk_synth() -> Result<Arc<HardwareControls>> {
//     start_logging();
//     let synth = mk_synth()?;
//
//     Ok(synth)
// }

#[tauri::command]
fn play_note(synth: State<'_, Arc<Controller>>, note: Note) {
    // info!("before play");
    synth.play(note);
    // info!("after play");
}

#[tauri::command]
fn stop_note(synth: State<'_, Arc<Controller>>, note: Note) {
    synth.stop(note);
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = start_logging() {
        eprintln!("{e}");
        return;
    };

    let (synth, (stream_handle, audio)) = match mk_synth().await {
        Ok(synth) => synth,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    let audio_gen = AudioGen {
        controller: synth.clone(),
    };

    let _audio_gen_thread = spawn(async { audio_gen.await });
    // spawn(async move {
    //     stream_handle.play_raw(audio).unwrap();
    // });
    // info!("starting audio stream");
    stream_handle.play_raw(audio).unwrap();
    // info!("audio stream started");

    let midi_con = MIDIControls::new(synth.clone());

    if let Err(e) = midi_con {
        error!("No MIDI for you! {e}");
    } else {
        info!("MIDI started");
    }
    // let _audio_out_thread = spawn(async { audio_handle.await });

    // _ = synth.connect(1, 0, 2, envelope::AUDIO_IN);
    // _ = synth.connect(2, 0, 5, chorus::AUDIO_INPUT);
    // _ = synth.connect(5, 0, 0, 0);
    // _ = synth.connect(3, 0, 1, vco::PITCH_BEND_INPUT);
    // synth.modules.lock().unwrap().lfo[0].set_pitch(1.0);
    // synth.modules.lock().unwrap().lfo[0].volume_in = 0.7;
    default_connections(synth.clone(), 10);

    tauri::Builder::default()
        .manage(synth)
        .invoke_handler(tauri::generate_handler![play_note, stop_note])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
