use common::models::*;
use crate::components::breakdown_plot::BreakdownPlot;
use crate::components::speech_overlay::{OverlaySelection};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_hooks::prelude::use_window_size;
//use log::info;

#[derive(Properties, PartialEq)]
pub struct BreakdownProps {
  pub breakdown_type: BreakdownType,
  pub word: String,
  pub show_counts: bool,
  pub get_speeches: Callback<OverlaySelection>,
}

#[function_component(Breakdown)]
pub fn breakdown(props: &BreakdownProps) -> Html {
    let data = use_state(|| None);
    let word_state = use_state(|| String::from("")); // todo: rename all word to search or something
    let loading = use_state(|| false);
    let window_size = use_window_size();

    {
        let data = data.clone();
        let loading = loading.clone();
        let word = props.word.clone();
        let breakdown_type = props.breakdown_type.clone();
        use_effect(move || {
            if (*word_state) != word {
                loading.set(true);
                word_state.set(word.clone());
                spawn_local(async move {
                    let breakdown_request = DataRequest { search: word };
                    let uri = format!("/api/breakdown/{}", breakdown_type);
                    let resp = Request::put(&uri)
                        .header("Content-Type", "application/json")
                        .json(&breakdown_request).expect("couldn't create request body")
                        .send().await.unwrap();
                    let result = {
                        if !resp.ok() {
                            Err(format!(
                                "error fetching breakdown data {} ({})",
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

    let mut breakdown_data: Option<Result<Vec<BreakdownResponse>, String>> = None;
    if let Some(res) = data.as_ref() {
        breakdown_data = match res {
            Ok(data) => Some(Ok(serde_json::from_str::<Vec<BreakdownResponse>>(data).unwrap())),
            Err(err) => Some(Err(err.to_string())),
        };
    }
    
    html! {
        <BreakdownPlot
            breakdown_type={props.breakdown_type.clone()}
            data={breakdown_data}
            show_counts={props.show_counts}
            loading={*loading}
            window_width={window_size.0} 
            get_speeches={&props.get_speeches}
        />
    }
}
