// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// use graphs::{mk_graph, ModCounter};
// use graphviz_rust::{cmd::Format, dot_generator::*, exec, printer::PrinterContext};
use std::{
    ops::{DerefMut, Index, IndexMut},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use synth_8080::{
    chorus::{self, Chorus},
    common::{notes::Note, Module},
    controller::{midi::MIDIControls, Controller},
    default_modules,
    delay::Delay,
    echo::{self, Echo},
    envelope::EnvelopeFilter,
    lfo::Lfo,
    midi_osc::MidiOsc,
    mk_synth,
    osc::OscType,
    output::Output,
    overdrive::{self, OverDrive},
    reverb::{self, ReverbModule},
    start_logging,
    vco::Vco,
    AudioGen, Float,
};
use synth_8080_lib::{FilterType, ModuleType};
use tauri::{async_runtime::spawn, Emitter, Manager, State, Window};
use tracing::*;

// pub mod graphs;

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
    synth.modules.lock().unwrap().lfo[id as usize].volume_in = volume;
}

#[tauri::command]
fn set_vco_vol(synth: State<'_, Arc<Controller>>, volume: Float) {
    synth.modules.lock().unwrap().mco[0].set_volume(volume);
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
    synth.modules.lock().unwrap().mco[0].set_attack(value);
}

#[tauri::command]
fn set_env_decay(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_decay(value);
}

#[tauri::command]
fn set_env_break(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_break(value);
}

#[tauri::command]
fn set_env_sustain(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_sustain(value);
}

#[tauri::command]
fn set_env_decay_2(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_decay_2(value);
}

#[tauri::command]
fn set_echo_vol(synth: State<'_, Arc<Controller>>, volume: Float) {
    synth.modules.lock().unwrap().echo[0].recv_samples(echo::DECAY_INPUT, &vec![volume]);
}

#[tauri::command]
fn set_echo_speed(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().echo[0].recv_samples(echo::SPEED_INPUT, &vec![value]);
}

#[tauri::command]
fn set_chorus_vol(synth: State<'_, Arc<Controller>>, volume: Float) {
    synth.modules.lock().unwrap().chorus[0].recv_samples(chorus::DECAY_INPUT, &vec![volume]);
}

#[tauri::command]
fn set_chorus_speed(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().chorus[0].recv_samples(chorus::SPEED_INPUT, &vec![value]);
}

#[tauri::command]
fn set_od_gain(synth: State<'_, Arc<Controller>>, volume: Float) {
    synth.modules.lock().unwrap().over_drive[0].recv_samples(overdrive::GAIN_INPUT, &vec![volume]);
}

#[tauri::command]
fn set_output_volume(synth: State<'_, Arc<Controller>>, volume: Float) {
    let mut output = synth.output.lock().unwrap();
    (*output).volume = volume;
}

#[tauri::command]
fn set_reverb_gain(synth: State<'_, Arc<Controller>>, volume: Float) {
    // Gain control of Reverb is an f32 (as mandated by the library)
    let v: f32 = volume as f32;

    synth.modules.lock().unwrap().reverb[0].gain = v;
}

#[tauri::command]
fn set_reverb_decay(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().reverb[0].recv_samples(reverb::DECAY_INPUT, &vec![value]);
}

#[tauri::command]
fn get_connections(
    synth: State<'_, Arc<Controller>>,
) -> Vec<(ModuleType, u8, usize, ModuleType, u8, usize)> {
    let mods = {
        let tmp_mods = synth.modules.lock().unwrap();
        tmp_mods.indices.clone()
    };

    let cons = {
        let tmp_cons = synth.connections.lock().unwrap();
        tmp_cons.clone()
    };

    cons.iter()
        .map(|con| {
            // info!("src {}, dest {}", con.src_module, con.dest_module);

            (
                mods[(con.src_module - 1) as usize].0,
                con.src_output,
                mods[(con.src_module - 1) as usize].1,
                if con.dest_module >= 1 {
                    mods[(con.dest_module - 1) as usize].0
                } else {
                    ModuleType::Output
                },
                con.dest_input,
                if con.dest_module >= 1 {
                    mods[(con.dest_module - 1) as usize].1
                } else {
                    0
                },
            )
        })
        .collect()
}

// /// returns an HTML displayable network graph of connections
// #[tauri::command]
// fn get_connection_graph(
//     synth: State<'_, Arc<Controller>>,
//     base_graph: State<'_, Arc<Mutex<ModCounter>>>,
// ) -> String {
//     use graphviz_rust::dot_structures::*;
//
//     let mods = {
//         let tmp_mods = synth.modules.lock().unwrap();
//         tmp_mods.indices.clone()
//     };
//
//     let cons = {
//         let tmp_cons = synth.connections.lock().unwrap();
//         tmp_cons.clone()
//     };
//
//     let mut connections: Vec<Stmt> = cons
//         .iter()
//         .map(|con| {
//             let src_type = mods[(con.src_module - 1) as usize].0;
//             let src_index = mods[(con.src_module - 1) as usize].1;
//             let src_output = con.src_output;
//
//             let dest_type = if con.dest_module > 1 {
//                 mods[(con.dest_module - 1) as usize].0
//             } else {
//                 ModuleType::Output
//             };
//             let dest_index = if con.dest_module > 1 {
//                 mods[(con.dest_module - 1) as usize].1
//             } else {
//                 0
//             };
//             let dest_input = con.dest_input;
//
//             let src_name = format!("{src_type:?}-{src_index}-{src_output}");
//             let dest_name = format!("{dest_type:?}-{dest_index}-{dest_input}");
//
//             Stmt::Edge(edge!(node_id!(src_name) => node_id!(dest_name)))
//         })
//         .collect();
//
//     let mut base = base_graph.lock().unwrap().graphs.clone();
//     base.append(&mut connections);
//
//     let graph = graph!(strict di id!("t"), base);
// }

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
    } else {
        info!("connected");
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

    if let Err(e) = synth.disconnect(
        src_i.try_into().unwrap(),
        src_mod.2,
        dest_i.try_into().unwrap(),
        dest_mod.2,
    ) {
        error!("failed to disconnect two modules: {e}");
    } else {
        info!("disconnected");
    }

    Some(())
}

// #[tauri::command(rename_all = "snake_case")]
#[tauri::command]
fn update_connection_list(window: Window, synth: State<'_, Arc<Controller>>) {
    let synth = Arc::clone(&synth);

    thread::spawn(move || {
        let mut connections = {
            let tmp_cons = synth.connections.lock().unwrap();
            tmp_cons.clone()
        };

        let split_second: f64 = 0.25; // 1.0 / (SAMPLE_RATE as f64 * 0.25);

        loop {
            let new_cons = {
                let tmp_cons = synth.connections.lock().unwrap();
                let cons = tmp_cons.clone();

                cons
            };

            if new_cons != connections {
                info!("telling front end to updated connection list.");
                window.emit("update-connections-list", ()).unwrap();
            }

            connections = new_cons;

            thread::sleep(Duration::from_secs_f64(split_second));
        }
    });

    info!("started update checking thread");
}

#[tauri::command]
fn list_midi_controllers(midi_con: State<'_, Arc<Mutex<Option<MIDIControls>>>>) -> Vec<String> {
    let mut connection = midi_con.lock().unwrap();

    if let Some(ref mut midi) = connection.deref_mut() {
        midi.list_ports()
            .unwrap_or(Vec::new())
            .into_iter()
            .map(|port| port.1)
            .collect()
    } else {
        Vec::new()
    }
}

#[tauri::command]
fn reconnect_midi(midi_con: State<'_, Arc<Mutex<Option<MIDIControls>>>>) {
    let mut connection = midi_con.lock().unwrap();

    if let Some(ref mut midi) = connection.deref_mut() {
        if let Err(e) = midi.connect_default() {
            // if let Err(e) = midi.connect_all() {
            error!("failed to reconnect to midi device: {e}");
        } else {
            info!("reconnected to midi device.");
        }
    }
    // midi_con
    //     .lock()
    //     .unwrap()
    //     .list_ports()
    //     .unwrap_or(Vec::new())
    //     .into_iter()
    //     .map(|port| port.1)
    //     .collect()
}

#[tauri::command]
fn set_polyphony(synth: State<'_, Arc<Controller>>, n: usize) {
    synth.modules.lock().unwrap().mco[0].set_polyphony(n);
}

#[tauri::command(rename_all = "snake_case")]
fn get_module_outputs(module_type: ModuleType) -> Vec<String> {
    match module_type {
        ModuleType::EnvFilter => EnvelopeFilter::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Vco => Vco::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Reverb => ReverbModule::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Lfo => Lfo::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::MCO => MidiOsc::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Echo => Echo::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Delay => Delay::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Output => Output::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Chorus => Chorus::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::OverDrive => OverDrive::get_output_names()
            .map(|name| format!("{name}"))
            .collect(),
    }
}

#[tauri::command(rename_all = "snake_case")]
fn get_module_inputs(module_type: ModuleType) -> Vec<String> {
    match module_type {
        ModuleType::EnvFilter => EnvelopeFilter::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Vco => Vco::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Reverb => ReverbModule::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Lfo => Lfo::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::MCO => MidiOsc::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Echo => Echo::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Delay => Delay::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Output => Output::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::Chorus => Chorus::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
        ModuleType::OverDrive => OverDrive::get_input_names()
            .map(|name| format!("{name}"))
            .collect(),
    }
}

#[tauri::command(rename_all = "snake_case")]
fn enable_overtones(synth: State<'_, Arc<Controller>>, enabled: bool) {
    synth.modules.lock().unwrap().mco[0].set_overtones(enabled);
}

#[tauri::command]
fn set_env_cutoff(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_cutoff(value);
}

#[tauri::command]
fn set_env_resonance(synth: State<'_, Arc<Controller>>, value: Float) {
    synth.modules.lock().unwrap().mco[0].set_resonance(value);
}

fn start_midi(synth: Arc<Controller>) -> anyhow::Result<MIDIControls> {
    let mut midi_con = MIDIControls::new(synth)?;
    midi_con.connect_default()?;

    Ok(midi_con)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = start_logging() {
        eprintln!("{e}");
        return;
    } else {
        info!("desk-synth logging initialized");
    }

    let modules = default_modules();

    let (synth, (sink, audio)) = match mk_synth(&modules).await {
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

    let _audio_gen_thread = spawn(async { audio_gen.await });
    // info!("starting audio stream");
    // let sink = rodio::Sink::connect_new(&stream.mixer());
    // stream_handle.play_raw(audio).unwrap();
    sink.append(audio);
    sink.play();
    // info!("audio stream started");

    // let midi_con = MIDIControls::new(synth.clone());

    let midi_con = match start_midi(synth.clone()) {
        Err(e) => {
            error!("No MIDI for you! {e}");
            None
        }
        Ok(midi) => {
            info!("MIDI started");
            Some(midi)
        }
    };

    _ = synth.connect(1, 0, 4, chorus::AUDIO_INPUT);
    // _ = synth.connect(1, 0, 7, reverb::AUDIO_INPUT);
    // _ = synth.connect(7, 0, 0, 0);
    _ = synth.connect(4, 0, 0, 0);

    // mco => chorus => echo => output
    // _ = synth.connect(1, 0, 4, chorus::AUDIO_INPUT);
    // _ = synth.connect(4, 0, 3, echo::AUDIO_INPUT);
    // _ = synth.connect(3, 0, 0, 0);

    // mco => chorus => echo => overdrive => output
    // _ = synth.connect(1, 0, 4, chorus::AUDIO_INPUT);
    // _ = synth.connect(4, 0, 3, echo::AUDIO_INPUT);
    // _ = synth.connect(3, 0, 6, overdrive::AUDIO_INPUT);
    // _ = synth.connect(6, 0, 0, 0);

    // _ = synth.connect(1, 0, 0, 0);

    {
        synth.output.lock().unwrap().set_volume(0.5);
        // synth.modules.lock().unwrap().mco[0].set_volume(0.5);
    }

    tauri::Builder::default()
        .manage(synth)
        .manage(Arc::new(Mutex::new(midi_con)))
        .manage(Arc::new(Mutex::new(modules)))
        // .manage(Arc::new(Mutex::new(base_graph)))
        // .setup(|app| {})
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
            update_connection_list,
            list_midi_controllers,
            reconnect_midi,
            set_polyphony,
            // get_connection_graph,
            get_module_inputs,
            get_module_outputs,
            enable_overtones,
            set_env_cutoff,
            set_env_resonance,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
