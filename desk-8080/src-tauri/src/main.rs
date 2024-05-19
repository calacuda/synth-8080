// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    mem::size_of,
    ops::{Index, IndexMut},
    sync::Arc,
};
use synth_8080::{
    chorus,
    common::{notes::Note, Module},
    controller::{midi::MIDIControls, Controller},
    echo, mk_synth,
    osc::OscType,
    overdrive, reverb, start_logging, AudioGen, Float, SAMPLE_RATE,
};
use synth_8080_lib::{FilterType, ModuleType};
use tauri::{async_runtime::spawn, State};
use tracing::*;

pub type Volume = Float;
pub type Frequency = Float;

#[tauri::command]
fn play_note(synth: State<'_, Arc<Controller>>, note: Note) {
    synth.play(note);
}

#[tauri::command]
fn stop_note(synth: State<'_, Arc<Controller>>, note: Note) {
    synth.stop(note);
}

#[tauri::command]
fn get_lfo_osc(synth: State<'_, Arc<Controller>>, id: u8) -> OscType {
    synth
        .modules
        .lock()
        .unwrap()
        .lfo
        .index(id as usize)
        .osc_type
}

#[tauri::command(rename_all = "snake_case")]
fn set_lfo_osc(synth: State<'_, Arc<Controller>>, id: u8, osc_type: OscType) {
    synth
        .modules
        .lock()
        .unwrap()
        .lfo
        .index_mut(id as usize)
        .set_osc_type(osc_type);
}

#[tauri::command]
fn set_lfo_freq(synth: State<'_, Arc<Controller>>, id: u8, frequency: Float) {
    synth.modules.lock().unwrap().lfo[id as usize].set_pitch(frequency / 12.0);
}

#[tauri::command]
fn set_lfo_vol(synth: State<'_, Arc<Controller>>, id: u8, volume: Float) {
    synth.modules.lock().unwrap().lfo[id as usize].volume_in = volume / 100.0;
}

#[tauri::command]
fn set_vco_vol(synth: State<'_, Arc<Controller>>, volume: Float) {
    synth.modules.lock().unwrap().mco[0].set_volume(volume / 100.0);
}

#[tauri::command(rename_all = "snake_case")]
fn set_vco_osc(synth: State<'_, Arc<Controller>>, osc_type: OscType) {
    synth.modules.lock().unwrap().mco[0].set_wave_form(osc_type);
}

#[tauri::command(rename_all = "snake_case")]
fn set_env(synth: State<'_, Arc<Controller>>, env_type: FilterType) {
    synth.modules.lock().unwrap().mco[0].set_env(env_type);
}

#[tauri::command]
fn get_vco_env(synth: State<'_, Arc<Controller>>) -> FilterType {
    synth.modules.lock().unwrap().mco[0].oscs[0].1.filter_type
}

#[tauri::command]
fn get_vco_osc(synth: State<'_, Arc<Controller>>) -> OscType {
    synth.modules.lock().unwrap().mco[0].oscs[0].0.osc.waveform
}

#[tauri::command]
fn set_env_atk(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_attack(value / 100.0);
}

#[tauri::command]
fn set_env_decay(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_decay(value / 100.0);
}

#[tauri::command]
fn set_env_break(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_break(value / 100.0);
}

#[tauri::command]
fn set_env_sustain(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_sustain(value / 100.0);
}

#[tauri::command]
fn set_env_decay_2(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_decay_2(value / 100.0);
}

#[tauri::command]
fn set_echo_vol(synth: State<'_, Arc<Controller>>, volume: Float) {
    synth.modules.lock().unwrap().echo[0].recv_samples(echo::DECAY_INPUT, &vec![volume / 100.0]);
}

#[tauri::command]
fn set_echo_speed(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().echo[0].recv_samples(echo::SPEED_INPUT, &vec![value / 100.0]);
}

#[tauri::command]
fn set_chorus_vol(synth: State<'_, Arc<Controller>>, volume: Float) {
    synth.modules.lock().unwrap().chorus[0]
        .recv_samples(chorus::DECAY_INPUT, &vec![volume / 100.0]);
}

#[tauri::command]
fn set_chorus_speed(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().chorus[0].recv_samples(chorus::SPEED_INPUT, &vec![value / 100.0]);
}

#[tauri::command]
fn set_od_gain(synth: State<'_, Arc<Controller>>, volume: Float) {
    synth.modules.lock().unwrap().over_drive[0]
        .recv_samples(overdrive::GAIN_INPUT, &vec![volume / 100.0]);
}

#[tauri::command]
fn set_output_volume(synth: State<'_, Arc<Controller>>, volume: Float) {
    let mut output = synth.output.lock().unwrap();
    (*output).volume = volume / 100.0;
}

#[tauri::command]
fn set_reverb_gain(synth: State<'_, Arc<Controller>>, volume: Float) {
    // Gain control of Reverb is an f32 (as mandated by the library)
    let v: f32 = (volume / 100.0) as f32;

    synth.modules.lock().unwrap().reverb[0].gain = v;
}

#[tauri::command]
fn set_reverb_decay(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().reverb[0].recv_samples(reverb::DECAY_INPUT, &vec![value / 100.0]);
}

#[tauri::command]
fn get_connections(
    synth: State<'_, Arc<Controller>>,
) -> Vec<(ModuleType, u8, usize, ModuleType, u8, usize)> {
    let tmp_mods = synth.modules.lock().unwrap();
    let mods = tmp_mods.indices.clone();
    // mods.indices.iter().for_each(|()| {});
    let tmp_cons = synth.connections.lock().unwrap();
    let cons = tmp_cons.clone();
    // forget(tmp_cons);
    // forget(tmp_mods);

    cons.iter()
        .map(|con| {
            // info!("src {}, dest {}", con.src_module, con.dest_module);

            (
                mods[(con.src_module - 1) as usize].0,
                con.src_output,
                mods[(con.src_module - 1) as usize].1,
                if con.dest_module > 1 {
                    mods[(con.dest_module - 1) as usize].0
                } else {
                    ModuleType::Output
                },
                con.dest_input,
                if con.dest_module > 1 {
                    mods[(con.dest_module - 1) as usize].1
                } else {
                    0
                },
            )
        })
        .collect()
}

#[tauri::command(rename_all = "snake_case")]
fn connect(
    synth: State<'_, Arc<Controller>>,
    src_mod: (ModuleType, usize, u8),
    dest_mod: (ModuleType, usize, u8),
) -> Option<()> {
    let mods = {
        let tmp_mods = synth.modules.lock().unwrap();
        tmp_mods.indices.clone()
    };

    // let tmp_cons = synth.connections.lock().unwrap();
    // let cons = tmp_cons.clone();
    let src_i = if src_mod.0 != ModuleType::Output {
        mods.iter().position(|m| *m == (src_mod.0, src_mod.1))? + 1
    } else {
        0
    };

    let dest_i = if dest_mod.0 != ModuleType::Output {
        mods.iter().position(|m| *m == (dest_mod.0, dest_mod.1))? + 1
    } else {
        0
    };

    if let Err(e) = synth.connect(
        src_i.try_into().unwrap(),
        src_mod.2,
        dest_i.try_into().unwrap(),
        dest_mod.2,
    ) {
        error!("failed to connect two modules: {e}");
    }

    Some(())
}

#[tauri::command(rename_all = "snake_case")]
fn disconnect(
    synth: State<'_, Arc<Controller>>,
    src_mod: (ModuleType, usize, u8),
    dest_mod: (ModuleType, usize, u8),
) -> Option<()> {
    let mods = {
        let tmp_mods = synth.modules.lock().unwrap();
        tmp_mods.indices.clone()
    };
    // let tmp_cons = synth.connections.lock().unwrap();
    // let cons = tmp_cons.clone();
    let src_i = if src_mod.0 != ModuleType::Output {
        mods.iter().position(|m| *m == (src_mod.0, src_mod.1))? + 1
    } else {
        0
    };

    // println!("found source between {:?} & {}", src_mod.0, src_mod.1);
    let dest_i = if dest_mod.0 != ModuleType::Output {
        mods.iter().position(|m| *m == (dest_mod.0, dest_mod.1))? + 1
    } else {
        0
    };

    // println!("found destination {:?} & {}", dest_mod.0, src_mod.1);
    //
    // println!("rming connection between {} & {}", src_i, dest_i);

    if let Err(e) = synth.disconnect(
        src_i.try_into().unwrap(),
        src_mod.2,
        dest_i.try_into().unwrap(),
        dest_mod.2,
    ) {
        error!("failed to disconnect two modules: {e}");
    }
    // else {
    //     println!("rmed");
    // }

    Some(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = start_logging() {
        eprintln!("{e}");
        return;
    } else {
        info!("desk-synth logging initialized");
    }

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
    // info!("starting audio stream");
    stream_handle.play_raw(audio).unwrap();
    // info!("audio stream started");

    let midi_con = MIDIControls::new(synth.clone());

    if let Err(e) = midi_con {
        error!("No MIDI for you! {e}");
    } else {
        info!("MIDI started");
    }

    // _ = synth.connect(2, 0, 1, midi_osc::VOLUME);
    _ = synth.connect(2, 0, 4, chorus::SPEED_INPUT);
    // _ = synth.connect(2, 0, 4, chorus::DECAY_INPUT);
    _ = synth.connect(1, 0, 4, chorus::AUDIO_INPUT);
    // _ = synth.connect(2, 0, 0, output::);
    _ = synth.connect(1, 0, 7, reverb::AUDIO_INPUT);
    _ = synth.connect(7, 0, 0, 0);
    _ = synth.connect(4, 0, 0, 0);
    // _ = synth.connect(1, 0, 0, 0);
    // _ = synth.connect(4, 0, 3, echo::AUDIO_INPUT);
    // _ = synth.connect(3, 0, 0, 0);
    // _ = synth.connect(1, 0, 0, 0);

    tauri::Builder::default()
        .manage(synth)
        .invoke_handler(tauri::generate_handler![
            play_note,
            stop_note,
            get_lfo_osc,
            set_lfo_osc,
            set_lfo_freq,
            set_lfo_vol,
            set_vco_vol,
            set_vco_osc,
            set_env,
            get_vco_env,
            get_vco_osc,
            set_env_atk,
            set_env_decay,
            set_env_break,
            set_env_sustain,
            set_env_decay_2,
            set_echo_vol,
            set_echo_speed,
            set_chorus_vol,
            set_chorus_speed,
            set_od_gain,
            set_output_volume,
            set_reverb_gain,
            set_reverb_decay,
            get_connections,
            connect,
            disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
