use common::models::{BreakdownType, DataRequest, SpeechResponse, Speaker};
use crate::components::speech_box::SpeechBox;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

//todo move to new utils file
#[derive(PartialEq, Clone)]
pub struct OverlaySelection {
    pub breakdown_type: BreakdownType,
    pub id: i32,
    pub heading: String,
}

#[derive(Properties, PartialEq)]
pub struct SpeechOverlayProps {
    pub selection: OverlaySelection,
    pub word: String,
    pub visible: bool,
    pub hide: Callback<MouseEvent>,
    pub speakers: Rc<HashMap<i32, Speaker>>,
}

#[function_component(SpeechOverlay)]
pub fn speech_overlay(props: &SpeechOverlayProps) -> Html {
    let data = use_state(|| None);
    let selection_state = use_state(|| OverlaySelection { breakdown_type: BreakdownType::Party, id: 0, heading: String::from("")} ); // todo use default?

    {
        let data = data.clone();
        let selection = props.selection.clone();
        let word = props.word.clone();
        let visible = props.visible;
        use_effect(move || {
            if visible && (*selection_state) != selection {
                data.set(None);
                selection_state.set(selection.clone());
                spawn_local(async move {
                    let speech_request = DataRequest { search: word };
                    let uri = format!("/api/speeches/{}/{}", selection.breakdown_type, selection.id);
                    let resp = Request::put(&uri)
                        .header("Content-Type", "application/json")
                        .json(&speech_request).expect("couldn't create request body")
                        .send().await.unwrap();
                    let result = {
                        if !resp.ok() {
                            Err(format!(
                                "error fetching speech data {} ({})",
                                resp.status(),
                                resp.status_text()
                            ))
                        } else {
                            resp.text().await.map_err(|err| err.to_string())
                        }
                    };
                    data.set(Some(result));
                });
            }

            || {} //todo check if can remove
        });
    }
    
    if !props.visible {
        return html! { <div style="display: none" /> }
    }
            
    html! {
        <div class="speech-overlay-mask">
             <div class="speech-overlay">
                <div class="speech-overlay-container">
                    <h1 class="speech-overlay-heading">{props.selection.heading.clone()}</h1>
                    { match data.as_ref() {
                        None => {
                            html! {
                                <div class="loader-speech" />
                            }
                        }
                        Some(Ok(data)) => {
                            let speech_data: Vec<SpeechResponse> = serde_json::from_str(data).unwrap();
                            
                            speech_data.into_iter().map(|speech| {
                                let speaker = &(*(props.speakers))[&speech.speaker];
                                let name = format!("{} {}", speaker.first_name, speaker.last_name);
                                
                                html!{
                                    <SpeechBox
                                        {name}
                                        start={speech.start}
                                        end={speech.end}
                                        link={speech.link}
                                        text={speech.text}
                                        word={props.word.clone()}
                                    />
                                }
                            }).collect::<Html>()
                        }
                        Some(Err(err)) => {
                            html! {
                                <div>{"error requesting data from server: "}{err}</div>
                            }
                        }
                    } }
                    
                    <div class="speech-overlay-exit">
                        <button onclick={&props.hide}> {"X"} </button>
                    </div>
                </div>
             </div>
        </div>
    }
}
