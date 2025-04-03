use common::models::{BreakdownType, DataRequest, CancelRequest, SpeechResponse};
use crate::components::speech_box::SpeechBox;
use crate::pages::error_page::error_page;
use crate::State;
use crate::util::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;

/// Properties for the speech overlay component.

#[derive(Properties, PartialEq)]
pub struct SpeechOverlayProps {
    
    /// The current selected speech overlay, set by a plot.
    
    pub selection: OverlaySelection,
    
    /// The word the user searched for, to be highlighted.
    
    pub word: String,
    
    /// Whether the overlay is currently visible.
    
    pub visible: bool,
    
    /// A callback for the parent page to hide the overlay.
    
    pub hide: Callback<MouseEvent>,
    
    /// A reference to a store of a set of speakers.
    
    pub speakers: Rc<HashMap<i32, Speaker>>,
}

/// A speech overlay component, displaying a requested set of speeches.
///
/// This overlay mostly hides the page below.

#[function_component(SpeechOverlay)]
pub fn speech_overlay(props: &SpeechOverlayProps) -> Html {
    let data = use_state(|| None);
    let failed = use_state(|| false);
    let app_state = use_context::<State>();
    let selection_state = use_state(|| OverlaySelection { breakdown_type: BreakdownType::Party, id: 0, heading: String::from("")} ); // todo use default?

    {
        let data = data.clone();
        let app_state = app_state.clone();
        let selection = props.selection.clone();
        let word = props.word.clone();
        let visible = props.visible;
        let failed = failed.clone();
        use_effect(move || {
            if visible && (*selection_state) != selection {
                data.set(None);
                selection_state.set(selection.clone());
                spawn_local(async move {
                    let Some(state) = app_state
                        else { failed.set(true); return };
                    
                    let cancel_request = CancelRequest { uuid: state.uuid };
                    let Ok(_) = put("/api/cancel/speeches", cancel_request).await
                        else { failed.set(true); return };
                    
                    let uri = format!("/api/speeches/{}/{}", selection.breakdown_type, selection.id);
                    let speech_request = DataRequest { uuid: state.uuid, search: word };
                    let Ok(resp) = put(&uri, speech_request).await
                        else { failed.set(true); return };
                            
                    if resp.status() == 204 {
                        return;
                    }
                    
                    let Ok(result) = resp.text().await else { failed.set(true); return };
                    data.set(Some(result));
                });
            }

            || {}
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
                    
                    { match (*failed, data.as_ref()) {
                        (false, None) => {
                            html! {
                                <div class="loader-speech" />
                            }
                        },
                        (false, Some(data)) => {
                            let Ok(speech_data) = serde_json::from_str::<Vec<SpeechResponse>>(data) else {
                                failed.set(false);
                                return html! { <div/> }
                            };
                            
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
                        },
                        (true, _) => {
                            error_page()
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
