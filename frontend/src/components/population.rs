// todo combine with breakdown

use common::models::*;
use crate::components::plot::Plot;
use crate::components::population_plot_engine::PopulationEngine;
use crate::components::speech_overlay::{OverlaySelection};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_hooks::prelude::use_window_size;
use std::rc::Rc;
//use log::info;

#[derive(Properties, PartialEq)]
pub struct PopulationProps {
    pub visible: bool,
    pub word: String,
    pub show_counts: bool,
    pub get_speeches: Callback<OverlaySelection>,
}

#[function_component(Population)]
pub fn population(props: &PopulationProps) -> Html {
    let data = use_state(|| None);
    let word_state = use_state(|| String::from("")); // todo: rename all word to search or something
    let loading = use_state(|| false);
    let window_size = use_window_size();

    {
        let visible = props.visible.clone();
        let data = data.clone();
        let loading = loading.clone();
        let word = props.word.clone();
        use_effect(move || {
            if (*word_state) != word && visible {
                loading.set(true);
                word_state.set(word.clone());
                spawn_local(async move {
                    let population_request = DataRequest { search: word };
                    let uri = "/api/population";
                    let resp = Request::put(&uri)
                        .header("Content-Type", "application/json")
                        .json(&population_request).expect("couldn't create request body")
                        .send().await.unwrap();
                    let result = {
                        if !resp.ok() {
                            Err(format!(
                                "error fetching population density data {} ({})",
                                resp.status(),
                                resp.status_text()
                            ))
                        } else {
                            resp.text().await.map_err(|err| err.to_string())
                        }
                    };
                    data.set(Some(result));
                    loading.set(false);
                });
            }

            || {}
        });
    }

    let mut population_data: Option<Result<Rc<Vec<PopulationResponse>>, String>> = None;
    if let Some(res) = data.as_ref() {
        population_data = match res {
            Ok(data) => Some(Ok(Rc::from(serde_json::from_str::<Vec<PopulationResponse>>(data).unwrap()))),
            Err(err) => Some(Err(err.to_string())),
        };
    }
    
    html! {
        if props.visible {
            <Plot<PopulationEngine, PopulationResponse>
                breakdown_type={BreakdownType::Speaker}
                data={population_data}
                show_counts={props.show_counts}
                loading={*loading}
                window_width={window_size.0} 
                get_speeches={&props.get_speeches}
            />
        }
        else {
            <div />
        }
    }
}
