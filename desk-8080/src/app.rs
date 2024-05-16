use leptos::leptos_dom::ev::SubmitEvent;
use leptos::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use synth_8080_lib::notes::Note;
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

#[component]
pub fn App() -> impl IntoView {
    let (playing, set_playing) = create_signal(false);

    let play = move |_| {
        spawn_local(async move {
            let args = to_value(&PlayArgs { note: Note::A4 }).unwrap();
            invoke("play_note", args).await;
            set_playing.set(true);
        });
    };

    let stop = move |_| {
        spawn_local(async move {
            let args = to_value(&PlayArgs { note: Note::A4 }).unwrap();
            invoke("stop_note", args).await;
            set_playing.set(false);
        });
    };

    view! {
        <main class="container">
            <button on:click=play>"play"</button>
            <button on:click=stop>"stop"</button>
            <div>
            { move || {
                    if playing.get() {
                        "you should hear sound"
                    } else {
                        "put on headphones"
                    }
                }
            }
            </div>
        </main>
    }
}
