use common::models::{BreakdownType, DataRequest, SpeechResponse, Speaker};
use crate::components::speech_box::SpeechBox;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;
use log::info;

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
    pub hide_overlay: Callback<MouseEvent>,
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
        <div style="position: fixed; left: 0; right: 0; bottom: 0; top: 0; background-color: rgba(0,0,0,0.85); z-index: 20">
             <div style="position: fixed; left: 20px; right: 20px; bottom: 20px; top: 20px; border: 2px solid #575757; border-radius: 15px; padding: 5px; background-color: rgba(0,0,0,0.75)">
                //<div style="height: 100%; display: flex; justify-content: center">
                    <div style="overflow: auto; height: 100%; width: 100%; display: flex; flex-direction: column; align-content: center">
                        <h1 style="text-align: center; color: #dddddd; margin: 0">{props.selection.heading.clone()}</h1>
                        { match data.as_ref() {
                            None => {
                                html! {
                                    <div style="text-align: center"> {"loading..."} </div>
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
                        
                        <div style="position: absolute; top: 2vh; right: 2vh">
                            <button style="background-color: #121212; border-color: #575757; color: #dddddd; border-radius: 10px; padding-block: 3px; padding-inline: 5px" onclick={&props.hide_overlay}> {"X"} </button>
                        </div>
                    </div>
                //</div>
             </div>
        </div>
    }
}
