use common::models::{BreakdownType, SpeakerResponse, Speaker};
use yew::prelude::*;
use gloo::utils::body;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use crate::components::breakdown::Breakdown;
use crate::components::population::Population;
use crate::components::speech_overlay::{SpeechOverlay, OverlaySelection};
use std::collections::HashMap;
use std::rc::Rc;
use log::info;

#[function_component(Interface)]
pub fn interface() -> Html {
    let speakers = use_state(|| None);
    
    let loading = use_state(|| false);
    let input_value = use_state(|| String::from(""));
    let word = use_state(|| String::from(""));
    let show_counts = use_state(|| false);
    let overlay_visible = use_state(|| false);
    let selection = use_state(|| OverlaySelection {breakdown_type: BreakdownType::Party, id: 0, heading: String::from("")});

    {
        let speakers = speakers.clone();
        let loading = loading.clone();
        use_effect(move || {
            if *speakers == None && *loading == false {
                loading.set(true);
                spawn_local(async move {
                    let uri = format!("/api/speakers");
                    let resp = Request::get(&uri).send().await.unwrap();
                    let speaker_response: Vec<SpeakerResponse> = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
                    speakers.set(Some(Rc::new(speaker_response
                        .into_iter()
                        .map(|s| {(s.id, Speaker {first_name: s.first_name, last_name: s.last_name})})
                        .collect::<HashMap<i32, Speaker>>()
                    )));
                    loading.set(false);
                });
            }
    
            || {}
        });
    }
    
    let on_input = {
        let input_value = input_value.clone();
        Callback::from(move |e : Event| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                input_value.set(input.value());
            }
        })
    };
    
    let on_show_counts = {
        let show_counts = show_counts.clone();
        Callback::from(move |e : MouseEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                show_counts.set(input.checked());
            }
        })
    };
    
    let submit = {
        let input_value = input_value.clone();
        let word = word.clone();
        Callback::from(move |e : SubmitEvent| {
            e.prevent_default();
            word.set((*input_value).clone());
        })
    };
    
    let get_speeches = {
        let selection = selection.clone();
        let overlay_visible = overlay_visible.clone();
        Callback::from(move |s: OverlaySelection| {
            info!("{} get speeches made it to parent, id {}", s.breakdown_type, s.id);
            selection.set(s);
            body().set_attribute("style", "overflow: hidden; background-color: #121212").unwrap();
            overlay_visible.set(true);
        })
    };
    
    let hide_overlay = {
        let overlay_visible = overlay_visible.clone();
        Callback::from(move |_| {
            info!("hide overlay");
            body().set_attribute("style", "overflow: auto; background-color: #121212").unwrap();
            overlay_visible.set(false);
        })
    };
    
    html! {
        <div style="background-color: #121212">
            <div >
                <form style="display: flex; flex-wrap: wrap; justify-content: center; color: #ffeba9" onsubmit={submit}>
                    <div style="align-self: center; margin-inline: 10px">
                        <label for="word_input"> {"search term:"}</label>
                        <input type="text" id="word_input" value={(*input_value).clone()} onchange={on_input} style="background-color: #3f3f3f; border-color: #3f3f3f; border-radius: 10px; color: #dddddd; margin: 5px" />
                    </div>
                    <div style="align-self: center; margin-inline: 10px">
                        <label for="show_counts"> {"show total counts:"}</label>
                        <input type="checkbox" id="show_counts" onclick={on_show_counts}/>
                    </div>
                    <div style="align-self: center; margin-inline: 10px">
                        <input type="submit" value="Submit" style="background-color: #575757; border-color: #575757; color: #dddddd; border-radius: 10px; margin: 5px; padding-block: 3px; padding-inline: 5px" />
                    </div>
                </form>
            </div>
            
            <div style="display: flex; flex-wrap: wrap; justify-content: center">
                <Breakdown breakdown_type={BreakdownType::Party} word={(*word).clone()} show_counts={*show_counts} get_speeches={&get_speeches}/>
                <Breakdown breakdown_type={BreakdownType::Gender} word={(*word).clone()} show_counts={*show_counts} get_speeches={&get_speeches}/>
                <Breakdown breakdown_type={BreakdownType::Province} word={(*word).clone()} show_counts={*show_counts} get_speeches={&get_speeches}/>
                <Breakdown breakdown_type={BreakdownType::Speaker} word={(*word).clone()} show_counts={*show_counts} get_speeches={&get_speeches}/>
                <Population word={(*word).clone()} show_counts={*show_counts} get_speeches={&get_speeches}/>
            </div>
            if (*selection).id != 0 {
                if (*loading) == false {
                    <SpeechOverlay
                        selection={(*selection).clone()}
                        word={(*word).clone()}
                        visible={*overlay_visible}
                        {hide_overlay}
                        speakers={Rc::clone((*speakers).as_ref().unwrap())}
                    />
                }
            }
        </div>
    }
}
