use common::models::{BreakdownType, CancelRequest, Speaker, SpeakerResponse};
use yew::prelude::*;
use gloo::utils::body;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use crate::components::charts::Charts;
use crate::components::speech_overlay::{SpeechOverlay, OverlaySelection};
use crate::pages::error_page::error_page;
use crate::State;
use crate::util::put;
use std::collections::HashMap;
use std::rc::Rc;

#[function_component(InterfacePage)]
pub fn interface_page() -> Html {
    let app_state = use_context::<State>();
    let speakers = use_state(|| None);
    let loading = use_state(|| false);
    let failed = use_state(|| false);
    
    let show_charts = use_state(|| false);
    let show_party = use_state(|| true);
    let show_gender = use_state(|| false);
    let show_province = use_state(|| false);
    let show_speaker = use_state(|| false);
    let show_pop = use_state(|| false);
    let input_value = use_state(|| String::from(""));
    let word = use_state(|| String::from(""));
    let show_counts = use_state(|| false);
    let speech_overlay_word = use_state(|| String::from(""));
    let speech_overlay_visible = use_state(|| false);
    let selection = use_state(|| OverlaySelection {breakdown_type: BreakdownType::Party, id: 0, heading: String::from("")});

    {
        let speakers = speakers.clone();
        let loading = loading.clone();
        let failed = failed.clone();
        use_effect(move || {
            if *speakers == None && *loading == false && *failed == false {
                loading.set(true);
                spawn_local(async move {
                    let uri = format!("/api/speakers");
                    let Ok(resp) = Request::get(&uri).send().await else {failed.set(true); return};
                    let Ok(resp_text) = &resp.text().await else {failed.set(true); return};
                    let Ok(speaker_response) = serde_json::from_str::<Vec<SpeakerResponse>>(resp_text) else {failed.set(true); return};
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
    
    fn build_on(state: UseStateHandle<bool>) -> Callback<MouseEvent> {
        Callback::from(move |e : MouseEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                state.set(input.checked());
            }
        })
    }
    
    let on_party = build_on(show_party.clone());
    let on_gender = build_on(show_gender.clone());
    let on_province = build_on(show_province.clone());
    let on_speaker = build_on(show_speaker.clone());
    let on_pop = build_on(show_pop.clone());
    let on_show_counts = build_on(show_counts.clone());
    
    let on_input = {
        let input_value = input_value.clone();
        Callback::from(move |e : Event| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                input_value.set(input.value());
            }
        })
    };
    
    let submit = {
        let input_value = input_value.clone();
        let word = word.clone();
        let app_state = app_state.clone();
        let failed = failed.clone();
        Callback::from(move |e : SubmitEvent| {
            e.prevent_default();
            let input_value = input_value.clone();
            let word = word.clone();
            let app_state = app_state.clone();
            if let Some(state) = app_state {
                spawn_local(async move {
                    let cancel_request = CancelRequest { uuid: state.uuid };
                    let _ = put("api/cancel", cancel_request).await;
                    word.set((*input_value).clone());
                });
            } else {
                failed.set(true);
            }
        })
    };
    
    let get_speeches = {
        let selection = selection.clone();
        let word = word.clone();
        let speech_overlay_word = speech_overlay_word.clone();
        let speech_overlay_visible = speech_overlay_visible.clone();
        Callback::from(move |s: OverlaySelection| {
            selection.set(s);
            body().set_class_name("body-covered");
            speech_overlay_word.set((*word).clone());
            speech_overlay_visible.set(true);
        })
    };
    
    let hide_speech_overlay = {
        let speech_overlay_visible = speech_overlay_visible.clone();
        Callback::from(move |_| {
            body().set_class_name("body");
            speech_overlay_visible.set(false);
        })
    };
    
    let toggle_charts = |toggle| {
        let show_charts = show_charts.clone();
        Callback::from(move |_| {
            if toggle {show_charts.set(!*show_charts);}
            else {show_charts.set(false);}
        })
    };
    
    let dummy_ref = Rc::new(HashMap::new());
    let provincial = if let Some(state) = app_state {state.provincial} else {failed.set(true); false};
    
    html! {
        <div class="interface">
            <div class="form-wrapper">
                <form onsubmit={submit}>
                    <div onmouseleave={&toggle_charts(false)}>
                        <button type="button" class="button" onclick={&toggle_charts(true)} >{"charts"}</button>
                        <div class="chart-dropdown" style={if *show_charts {"display: block"} else {"display: none"}} >
                            <div>
                                <label for="show_party"> {"party"}</label>
                                <input type="checkbox" id="show_party" onclick={on_party} checked={*show_party}/>
                            </div>
                            <div>
                                <label for="show_gender"> {"gender"}</label>
                                <input type="checkbox" id="show_gender" onclick={on_gender} />
                            </div>
                            if !provincial {
                                <div>
                                    <label for="show_province"> {"province"}</label>
                                    <input type="checkbox" id="show_province" onclick={on_province} />
                                </div>
                            }
                            <div>
                                <label for="show_speaker"> {"speaker"}</label>
                                <input type="checkbox" id="show_speaker" onclick={on_speaker} />
                            </div>
                            if !provincial {
                                <div>
                                    <label for="show_pop"> {"pop density"}</label>
                                    <input type="checkbox" id="show_pop" onclick={on_pop} />
                                </div>
                            }
                        </div>
                    </div>
                    <div class="form-section">
                        <label for="word_input"> {"search term:"}</label>
                        <input type="text" id="word_input" value={(*input_value).clone()} onchange={on_input} class="word"/>
                    </div>
                    <div class="form-section">
                        <label for="show_counts"> {"total counts"}</label>
                        <input type="checkbox" id="show_counts" onclick={on_show_counts}/>
                    </div>
                    <div class="form-section">
                        <input type="submit" value="submit" class="button"/>
                    </div>
                </form>
            </div>
            
            if !*failed {
                <Charts
                    {provincial}
                    word={(*word).clone()}
                    show_counts={*show_counts}
                    show_party={*show_party}
                    show_gender={*show_gender}
                    show_province={*show_province}
                    show_speaker={*show_speaker}
                    show_pop={*show_pop}
                    get_speeches={&get_speeches}
                />
            } else {
                {error_page()}
            }
            
            if (*selection).id != 0 {
                if !*loading && !*failed {
                    <SpeechOverlay
                        selection={(*selection).clone()}
                        word={(*speech_overlay_word).clone()}
                        visible={*speech_overlay_visible}
                        hide={hide_speech_overlay}
                        speakers={Rc::clone((*speakers).as_ref().unwrap_or_else(|| {failed.set(true); &dummy_ref}))}
                    />
                }
            }
        </div>
    }
}
