use leptos::{ev::Event, leptos_dom::logging::console_log, *};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::str::FromStr;
use strum::IntoEnumIterator;
use synth_8080_lib::{notes::Note, FilterType, Float, ModuleType, OscType};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct PlayArgs {
    note: Note,
}

#[derive(Serialize, Deserialize)]
struct LfoStateArgs {
    id: u8,
}

#[derive(Serialize, Deserialize)]
struct LfoFreqSetArgs {
    id: u8,
    frequency: Float,
}

#[derive(Serialize, Deserialize)]
struct LfoVolSetArgs {
    id: u8,
    volume: Float,
}

#[derive(Serialize, Deserialize)]
struct LfoSetOscType {
    id: u8,
    osc_type: OscType,
}

#[derive(Serialize, Deserialize)]
struct VcoVolSetArgs {
    volume: Float,
}

#[derive(Serialize, Deserialize)]
struct VcoSetOscType {
    osc_type: OscType,
}

#[derive(Serialize, Deserialize)]
struct Empty {}

#[derive(Serialize, Deserialize)]
struct EnvSetArgs {
    value: Float,
}

#[derive(Serialize, Deserialize)]
struct VcoSetEnvType {
    env_type: FilterType,
}

#[derive(Serialize, Deserialize)]
struct EditConnectionArgs {
    src_mod: (ModuleType, usize, u8),
    dest_mod: (ModuleType, usize, u8),
}

#[derive(Serialize, Deserialize)]
struct PolyphonySetArgs {
    n: usize,
}

#[derive(Serialize, Deserialize)]
struct SetOvertonesArgs {
    enabled: bool,
}

#[component]
pub fn App() -> impl IntoView {
    // TODO: add a button to scan for midi devices and choose which vco to connect them to.

    view! {
        <main class="p-4">
            <div class="grid grid-cols-4 gap-4">
                <div class="col-span-2">
                    <LFOs/>
                </div>

                <div class="col-span-2 gap-4">
                    <div class="gap-4 col-start-1 col-end-2">
                        <div class="col-span-2 grid grid-cols-2 gap-4">
                            <div>
                                <VCO/>
                                <div class="grid grid-cols-2 gap-4">
                                    // volume
                                    <Volume/>
                                    // overdrive
                                    <Overdrive/>
                                </div>
                            </div>
                            <EnvFilter/>
                        </div>
                    <div class="col-span-2 grid grid-cols-3 gap-4">
                        <Echo/>
                        <Chorus/>
                        <Reverb/>
                    </div>

                    </div>
                </div>
            </div>
            <Connections/>
        </main>
    }
}

#[component]
fn Slider(on_input: Box<dyn FnMut(Event)>) -> impl IntoView {
    // TODO: make it change with back end.
    view! {
        <input
            type="range"
            min="1"
            max="10000"
            value="5000"
            on:input=on_input
        />
    }
}

#[component]
fn LFO(index: u8) -> impl IntoView {
    // make signal for LFO state
    let (lfo_type, set_lfo_type) = create_signal(OscType::Sine);

    spawn_local(async move {
        set_lfo_type.set(
            OscType::from_str(
                invoke(
                    "get_lfo_osc",
                    to_value(&LfoStateArgs { id: index }).unwrap(),
                )
                .await
                .as_string()
                .unwrap_or("Sine".to_string())
                .as_str(),
            )
            .unwrap_or(OscType::Sine),
        )
    });

    let on_pitch_input = move |ev| {
        spawn_local(async move {
            invoke(
                "set_lfo_freq",
                to_value(&LfoFreqSetArgs {
                    id: index,
                    frequency: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    let on_vol_input = move |ev| {
        spawn_local(async move {
            invoke(
                "set_lfo_vol",
                to_value(&LfoVolSetArgs {
                    id: index,
                    volume: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    view! {
        <div class="border-4 rounded-md border-black text-center">
            <h1>{move || format!("LFO {}", index) }</h1>

            <div class="grid grid-flow-col">
                <div>
                    // frequency slider
                    <p>"pitch"</p>
                    <Slider on_input=Box::new(on_pitch_input)/>

                    // volume slider
                    <p>"vol."</p>
                    <Slider on_input=Box::new(on_vol_input)/>
                </div>

                // oscillator type indicator
                <div class="grid grid-flex-row">
                    <For
                        each=move || OscType::iter()
                        key=move |key| (key.clone(), *key == lfo_type.get())
                        children=move |osc| {
                            view! {
                                <div>
                                    <button on:click=move |_| {
                                        spawn_local(async move {
                                            invoke(
                                                "set_lfo_osc",
                                                to_value(&LfoSetOscType { id: index, osc_type: osc }).unwrap(),
                                            )
                                            .await;
                                            set_lfo_type.set(osc);
                                        })
                                    }>
                                        { move ||
                                            if osc == lfo_type.get() {
                                                format!("- [x] {osc:?}")
                                            } else {
                                                format!("- [ ] {osc:?}")
                                            }
                                        }
                                    </button>
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn LFOs() -> impl IntoView {
    view! {
        <div class="text-center">
            <h1>"LFOs"</h1>
            <div class="grid grid-cols-2 gap-4">
                <LFO index=0/>
                <LFO index=1/>
                <LFO index=2/>
                <LFO index=3/>
            </div>
        </div>
    }
}

#[component]
fn VCO() -> impl IntoView {
    // make signal for LFO state
    let (vco_type, set_vco_type) = create_signal(OscType::Sine);
    let (overtones, set_overtones) = create_signal(true);

    spawn_local(async move {
        set_vco_type.set(
            OscType::from_str(
                invoke("get_vco_osc", to_value(&Empty {}).unwrap())
                    .await
                    .as_string()
                    .unwrap_or("Sine".to_string())
                    .as_str(),
            )
            .unwrap_or(OscType::Sine),
        )
    });

    let on_volume_input = move |ev| {
        spawn_local(async move {
            invoke(
                "set_vco_vol",
                to_value(&VcoVolSetArgs {
                    volume: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    let on_overtones_click = move |_ev| {
        set_overtones.set(!overtones.get());

        spawn_local(async move {
            invoke(
                "enable_overtones",
                to_value(&SetOvertonesArgs {
                    enabled: overtones.get(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    view! {
        <div class="text-center">
            <h1> MCO </h1>

            <div class="border-4 rounded-md border-black text-center grid grid-cols-2">
                <div>
                    <div>
                        <p> "vol." </p>
                        <Slider on_input=Box::new(on_volume_input)/>
                    </div>
                    <div>
                        "Polyphony: "
                        <input type="number" id="src_mod_index" name="src_mod_index"
                            on:change=move |ev| {
                                // set_src_mod_index.set(event_target_value(&ev).parse().unwrap_or(0))
                                spawn_local(async move {
                                    invoke(
                                        "set_polyphony",
                                        to_value(&PolyphonySetArgs { n: event_target_value(&ev).parse().unwrap_or(4) }).unwrap(),
                                    )
                                    .await;
                                })

                            }
                            min="1"
                            max="10"
                            value="4"
                        />
                    </div>
                    <div>
                        <button on:click=on_overtones_click>
                            { move || {
                                if overtones.get() {
                                    "[x] "
                                } else {
                                    "[ ] "
                                }}
                            }
                            Overtones
                        </button>
                    </div>
                </div>

                // oscillator type indicator
                <div class="grid grid-flex-row">
                    <For
                        each=move || OscType::iter()
                        key=move |key| (key.clone(), *key == vco_type.get())
                        children=move |osc| {
                            view! {
                                <div>
                                    <button on:click=move |_| {
                                        spawn_local(async move {
                                            invoke(
                                                "set_vco_osc",
                                                to_value(&VcoSetOscType { osc_type: osc }).unwrap(),
                                            )
                                            .await;
                                            set_vco_type.set(osc);
                                        })
                                    }>
                                        { move ||
                                            if osc == vco_type.get() {
                                                format!("- [x] {osc:?}")
                                            } else {
                                                format!("- [ ] {osc:?}")
                                            }
                                        }
                                    </button>
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn EnvFilter() -> impl IntoView {
    let (env_type, set_env_type) = create_signal(FilterType::ADSR);

    spawn_local(async move {
        set_env_type.set(
            FilterType::from_str(
                invoke("get_vco_env", to_value(&Empty {}).unwrap())
                    .await
                    .as_string()
                    .unwrap_or("ADSR".to_string())
                    .as_str(),
            )
            .unwrap_or(FilterType::ADSR),
        )
    });

    let nothing = move || {
        view! {
            <p> </p>
            <div> </div>
        }
    };

    let attack = move || {
        let set_attack = move |ev| {
            spawn_local(async move {
                invoke(
                    "set_env_atk",
                    to_value(&EnvSetArgs {
                        value: event_target_value(&ev).parse().unwrap(),
                    })
                    .unwrap(),
                )
                .await;
            })
        };

        // if env_type.get() != FilterType::OC {
        view! {
            <p> "attack" </p>
            <Slider on_input=Box::new(set_attack)/>
        }
        // } else {
        //     nothing()
        // }
    };

    let decay = move || {
        let set_decay = move |ev| {
            spawn_local(async move {
                invoke(
                    "set_env_decay",
                    to_value(&EnvSetArgs {
                        value: event_target_value(&ev).parse().unwrap(),
                    })
                    .unwrap(),
                )
                .await;
            })
        };

        // if env_type.get() != FilterType::OC {
        view! {
            <p> "decay" </p>
            <Slider on_input=Box::new(set_decay)/>
        }
        // } else {
        //     nothing()
        // }
    };

    let sustain = move || {
        let set_sustain = move |ev| {
            spawn_local(async move {
                invoke(
                    "set_env_sustain",
                    to_value(&EnvSetArgs {
                        value: event_target_value(&ev).parse().unwrap(),
                    })
                    .unwrap(),
                )
                .await;
            })
        };

        if env_type.get() == FilterType::ADSR {
            view! {
                <p> "sustain" </p>
                <Slider on_input=Box::new(set_sustain)/>
            }
        } else {
            nothing()
        }
    };

    // let display_break = move || {
    //     let set_break = move |ev| {
    //         spawn_local(async move {
    //             invoke(
    //                 "set_env_break",
    //                 to_value(&EnvSetArgs {
    //                     value: event_target_value(&ev).parse().unwrap(),
    //                 })
    //                 .unwrap(),
    //             )
    //             .await;
    //         })
    //     };
    //
    //     if env_type.get() == FilterType::ADBDR {
    //         view! {
    //             <p> "break" </p>
    //             <Slider on_input=Box::new(set_break)/>
    //         }
    //     } else {
    //         nothing()
    //     }
    // };

    // let decay_2 = move || {
    //     let set_decay = move |ev| {
    //         spawn_local(async move {
    //             invoke(
    //                 "set_env_decay_2",
    //                 to_value(&EnvSetArgs {
    //                     value: event_target_value(&ev).parse().unwrap(),
    //                 })
    //                 .unwrap(),
    //             )
    //             .await;
    //         })
    //     };
    //
    //     if env_type.get() == FilterType::ADBDR {
    //         view! {
    //             <p> "decay 2" </p>
    //             <Slider on_input=Box::new(set_decay)/>
    //         }
    //     } else {
    //         nothing()
    //     }
    // };

    let cutoff = move || {
        let set_cutoff = move |ev| {
            spawn_local(async move {
                invoke(
                    "set_env_cutoff",
                    to_value(&EnvSetArgs {
                        value: event_target_value(&ev).parse().unwrap(),
                    })
                    .unwrap(),
                )
                .await;
            })
        };

        view! {
            <p> "cutoff" </p>
            <Slider on_input=Box::new(set_cutoff)/>
        }
    };

    let resonance = move || {
        let set_resonance = move |ev| {
            spawn_local(async move {
                invoke(
                    "set_env_resonance",
                    to_value(&EnvSetArgs {
                        value: event_target_value(&ev).parse().unwrap(),
                    })
                    .unwrap(),
                )
                .await;
            })
        };

        view! {
            <p> "resonance" </p>
            <Slider on_input=Box::new(set_resonance)/>
        }
    };

    view! {
        <div class="text-center">
            <h1> { move || format!("{:?}", env_type.get()) } </h1>
            <div class="border-4 rounded-md border-black text-center">
                <div>
                    { attack }
                    { decay }
                    { sustain }
                    // { display_break }
                    // { decay_2 }
                    { cutoff }
                    { resonance }
                </div>

                // <div>
                    // <For
                    //     each=move || FilterType::iter()
                    //     key=move |key| (key.clone(), *key == env_type.get())
                    //     children=move |env| {
                    //         view! {
                    //             <div>
                    //                 <button on:click=move |_| {
                    //                     spawn_local(async move {
                    //                         invoke(
                    //                             "set_env",
                    //                             to_value(&VcoSetEnvType { env_type: env }).unwrap(),
                    //                         )
                    //                         .await;
                    //                         set_env_type.set(env);
                    //                     })
                    //                 }>
                    //                     { move ||
                    //                         if env == env_type.get() {
                    //                             format!("- [x] {env:?}")
                    //                         } else {
                    //                             format!("- [ ] {env:?}")
                    //                         }
                    //                     }
                    //                 </button>
                    //             </div>
                    //         }
                    //     }
                    // />
                // </div>
            </div>
        </div>
    }
}

#[component]
fn Echo() -> impl IntoView {
    let on_volume = move |ev| {
        spawn_local(async move {
            invoke(
                "set_echo_vol",
                to_value(&VcoVolSetArgs {
                    volume: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    let on_speed = move |ev| {
        spawn_local(async move {
            invoke(
                "set_echo_speed",
                to_value(&EnvSetArgs {
                    value: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    view! {
        <div class="text-center">
            <h1> "Echo" </h1>
            <div class="border-4 rounded-md border-black text-center grid grid-cols-2">
                <p> "speed" </p>
                <Slider on_input=Box::new(on_speed)/>
                <p> "vol." </p>
                <Slider on_input=Box::new(on_volume)/>
            </div>
        </div>
    }
}

#[component]
fn Chorus() -> impl IntoView {
    let on_volume = move |ev| {
        spawn_local(async move {
            invoke(
                "set_chorus_vol",
                to_value(&VcoVolSetArgs {
                    volume: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    let on_speed = move |ev| {
        spawn_local(async move {
            invoke(
                "set_chorus_speed",
                to_value(&EnvSetArgs {
                    value: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    view! {
        <div class="text-center">
            <h1> "Chorus" </h1>
            <div class="border-4 rounded-md border-black text-center grid grid-cols-2">
                <p> "speed" </p>
                <Slider on_input=Box::new(on_speed)/>
                <p> "vol." </p>
                <Slider on_input=Box::new(on_volume)/>
            </div>
        </div>
    }
}

#[component]
fn Overdrive() -> impl IntoView {
    let on_gain_input = move |ev| {
        spawn_local(async move {
            invoke(
                "set_od_gain",
                to_value(&VcoVolSetArgs {
                    volume: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    view! {
        <div class="text-center">
            <h1> Over Drive </h1>

            <div class="border-4 rounded-md border-black text-center">
                <p> "gain" </p>
                <Slider on_input=Box::new(on_gain_input)/>
            </div>
        </div>
    }
}

#[component]
fn Volume() -> impl IntoView {
    let on_volume_input = move |ev| {
        spawn_local(async move {
            invoke(
                "set_output_volume",
                to_value(&VcoVolSetArgs {
                    volume: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    view! {
        <div class="text-center">
            <h1> Volume </h1>

            <div class="border-4 rounded-md border-black text-center">
                <p> "vol." </p>
                <Slider on_input=Box::new(on_volume_input)/>
            </div>
        </div>
    }
}

#[component]
fn Reverb() -> impl IntoView {
    let on_gain = move |ev| {
        spawn_local(async move {
            invoke(
                "set_reverb_gain",
                to_value(&VcoVolSetArgs {
                    volume: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    let on_decay = move |ev| {
        spawn_local(async move {
            invoke(
                "set_reverb_decay",
                to_value(&EnvSetArgs {
                    value: event_target_value(&ev).parse().unwrap(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    view! {
        <div class="text-center">
            <h1> "Reverb" </h1>
            <div class="border-4 rounded-md border-black text-center grid grid-cols-2">
                <p> "decay" </p>
                <Slider on_input=Box::new(on_decay)/>
                <p> "gain" </p>
                <Slider on_input=Box::new(on_gain)/>
            </div>
        </div>
    }
}

#[component]
fn Connections() -> impl IntoView {
    let (connections, set_connections) =
        create_signal(Vec::<(ModuleType, u8, usize, ModuleType, u8, usize)>::new());

    spawn_local(async move {
        set_connections.set(
            from_value(invoke("get_connections", to_value(&Empty {}).unwrap()).await)
                .unwrap_or(Vec::new()),
        )
    });

    let (src_mod_type, set_src_mod_type) = create_signal::<Option<ModuleType>>(None);
    let (src_mod_index, set_src_mod_index) = create_signal::<usize>(0);
    let (src_mod_output, set_src_mod_output) = create_signal::<u8>(0);

    let (dest_mod_type, set_dest_mod_type) = create_signal::<Option<ModuleType>>(None);
    let (dest_mod_index, set_dest_mod_index) = create_signal::<usize>(0);
    let (dest_mod_output, set_dest_mod_input) = create_signal::<u8>(0);

    // let (svg_src, set_svg_src) = create_signal(String::new());

    // spawn_local(async move {
    //     set_svg_src.set(
    //         from_value(invoke("get_connection_graph", to_value(&Empty {}).unwrap()).await)
    //             .unwrap_or(String::new()),
    //     )
    // });

    let connect = move |_| {
        if let (Some(src_mod), Some(dest_mod)) = (src_mod_type.get(), dest_mod_type.get()) {
            spawn_local(async move {
                invoke(
                    "connect",
                    to_value(&EditConnectionArgs {
                        src_mod: (src_mod, src_mod_index.get(), src_mod_output.get()),
                        dest_mod: (dest_mod, dest_mod_index.get(), dest_mod_output.get()),
                    })
                    .unwrap(),
                )
                .await;
                console_log("connected");
            });

            spawn_local(async move {
                set_connections.set(
                    from_value(invoke("get_connections", to_value(&Empty {}).unwrap()).await)
                        .unwrap_or(Vec::new()),
                );
                console_log("refreshed list");
            });

            set_src_mod_type.set(None);
            set_dest_mod_type.set(None);
        }
    };

    let re_con = move |_| {
        console_log("reconnecting to MIDI Keyboard");

        spawn_local(async move {
            invoke("reconnect_midi", to_value(&Empty {}).unwrap()).await;
        });
    };

    view! {
        <div class="grid grid-cols-2 text-center p-4 gap-4">
            // create connection box
            <div>
                <h1> Make Connection </h1>
                <div class="border-4 rounded-md border-black justify-center text-center grid grid-cols-2 p-4 gap-4">
                    <div>
                        <h1> Source Module </h1>
                        <div class="border-4 rounded-md border-black justify-center text-center grid grid-cols-2">
                            // get mod_type
                            <div class="text-left">
                                <legend>Module Type:</legend>
                                <For
                                    each=move || ModuleType::iter()
                                    key = move |module| (module.clone(), Some(*module) == src_mod_type.get())
                                    children=move |module| {
                                        view! {
                                            <div>
                                                <button
                                                    on:click=move |_| {
                                                        if src_mod_type.get() == Some(module) {
                                                            set_src_mod_type.set(None);
                                                        } else {
                                                            set_src_mod_type.set(Some(module))
                                                        }
                                                    }
                                                > { if src_mod_type.get() == Some(module) { format!("- [x] {module}") } else { format!("- [ ] {module}") } } </button>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                            <div class="justify-center text-center grid grid-rows-2">
                                // get mod index
                                {
                                    move || { if src_mod_type.get().is_some() {
                                        view! {
                                            <div class="text-left">
                                                <legend>Index:</legend>
                                                <input type="number" id="src_mod_index" name="src_mod_index"
                                                    on:change=move |ev| {
                                                        set_src_mod_index.set(event_target_value(&ev).parse().unwrap_or(0))
                                                    }
                                                    min="0"
                                                    max="255"
                                                    value="0"
                                                />
                                                // <label for="src_mod_index"> { format!("{module:?}") } </label>
                                            </div>
                                        }
                                    } else {
                                        view! {
                                            <div class="text-left">
                                            </div>
                                        }
                                    }}
                                }
                                // get mod output
                                {
                                    move || { if src_mod_type.get().is_some() {
                                        view! {
                                            <div class="text-left">
                                                <legend>Output:</legend>
                                                <input type="number" id="src_mod_index" name="src_mod_index"
                                                    on:change=move |ev| {
                                                        set_src_mod_output.set(event_target_value(&ev).parse().unwrap_or(0))
                                                    }
                                                    min="0"
                                                    max="255"
                                                    value="0"
                                                />
                                            </div>
                                        }
                                    } else {
                                        view! {
                                            <div class="text-left">
                                            </div>
                                        }
                                    }}
                                }
                            </div>
                        </div>
                    </div>
                    <div>
                        <h1> Destination Module </h1>
                        <div class="border-4 rounded-md border-black justify-center text-center grid grid-cols-2">
                        // get mod_type
                        <div class="text-left">
                            <legend>Module Type:</legend>
                            <For
                                each=move || ModuleType::iter()
                                key = move |module| (module.clone(), Some(*module) == dest_mod_type.get())
                                children=move |module| {
                                    view! {
                                        <div>
                                            <button
                                                on:click=move |_| {
                                                    if dest_mod_type.get() == Some(module) {
                                                        set_dest_mod_type.set(None)
                                                    } else {
                                                        set_dest_mod_type.set(Some(module))
                                                    }
                                                }
                                            > { if dest_mod_type.get() == Some(module) { format!("- [x] {module}") } else { format!("- [ ] {module}") } } </button>
                                        </div>
                                    }
                                }
                            />
                        </div>
                        <div class="justify-center text-center grid grid-rows-2">
                            // get mod index
                        {
                            move || { if dest_mod_type.get().is_some() {
                                view! {
                                    <div class="text-left">
                                        <legend>Index:</legend>
                                        <input type="number"
                                            on:change=move |ev| {
                                                set_dest_mod_index.set(event_target_value(&ev).parse().unwrap_or(0))
                                            }
                                            min="0"
                                            max="255"
                                            value="0"
                                        />
                                        // <label for="src_mod_index"> { format!("{module:?}") } </label>
                                    </div>
                                }
                            } else {
                                view! {
                                    <div class="text-left">
                                    </div>
                                }
                            }}
                        }
                        // get mod output
                        {
                            move || { if dest_mod_type.get().is_some() {
                                view! {
                                    <div class="text-left">
                                        <legend>Input:</legend>
                                        <input type="number" id="src_mod_index"
                                            on:change=move |ev| {
                                                set_dest_mod_input.set(event_target_value(&ev).parse().unwrap_or(0))
                                            }
                                            min="0"
                                            max="255"
                                            value="0"
                                        />
                                    </div>
                                }
                            } else {
                                view! {
                                    <div class="text-left">
                                    </div>
                                }
                            }}
                        }
                    </div>
                    </div>                    </div>
                    <div class="col-span-2">
                        // display connection before connecting
                        <button on:click=connect> Connect </button>
                    </div>
                </div>
            </div>
            // what's connected box
            <div>
                <button on:click=re_con> Reconnect Midi </button>
                <h1> Connected </h1>
                <div class="border-4 rounded-md border-black justify-center text-center grid grid-cols-4 text-wrap">
                    // <For
                    //     each=move || connections.get().into_iter().enumerate()
                    //     key=move |(i, con)| (*i, con.clone())
                    //     children=move |(_i, (src_mod, src_out, src_mod_n, dest_mod, dest_in, dest_mod_n))| {
                    { move ||
                        connections.get().into_iter().map(move |(src_mod, src_out, src_mod_n, dest_mod, dest_in, dest_mod_n)| {
                            console_log(&format!("{:?}", src_mod));

                            view! {
                                // disconnect button
                                <button
                                    on:click=move |_| {
                                        spawn_local(async move {
                                            invoke("disconnect", to_value(&EditConnectionArgs {
                                                src_mod: (src_mod, src_mod_n, src_out),
                                                dest_mod: (dest_mod, dest_mod_n, dest_in),
                                            }).unwrap()).await;
                                            console_log("disconnected");
                                        });

                                        spawn_local(async move {
                                            set_connections.set(
                                                from_value(invoke("get_connections", to_value(&Empty {}).unwrap()).await)
                                                    .unwrap_or(Vec::new()),
                                            );

                                            console_log("updated connection list");
                                        });
                                    }
                                > X </button>
                                <div>
                                    { format!("{src_mod:?}[{src_mod_n}]:{src_out}") }
                                </div>
                                <div>
                                    "=>"
                                </div>
                                <div>
                                    { format!("{dest_mod:?}[{dest_mod_n}]:{dest_in}") }
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }
                </div>
                // <div class="border-4 rounded-md border-black justify-center text-center grid grid-cols-4 text-wrap">
                //     <h1> Graph </h1>
                //     <svg src=svg_src> </svg>
                // </div>
            </div>
        </div>
    }
}
