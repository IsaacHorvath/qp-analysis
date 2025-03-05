use common::models::*;
use crate::components::population_plot::PopulationPlot;
use crate::components::speech_overlay::{OverlaySelection};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_hooks::prelude::use_window_size;
//use log::info;

#[derive(Properties, PartialEq)]
pub struct PopulationProps {
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
        let data = data.clone();
        let loading = loading.clone();
        let word = props.word.clone();
        use_effect(move || {
            if (*word_state) != word {
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

    match data.as_ref() {
        None => {
            html! {
                <div />
            }
        }
        Some(Ok(data)) => {
            match serde_json::from_str::<Option<Vec<PopulationResponse>>>(data).unwrap() {
                None => {
                    html! {
                        <div />
                    }
                }
                Some(population_data) => {
                    html! {
                        <PopulationPlot
                            data={population_data.clone()}
                            show_counts={props.show_counts}
                            loading={*loading}
                            window_width={window_size.0} 
                            get_speeches={&props.get_speeches}
                        />
                    }
                }
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    }
}
