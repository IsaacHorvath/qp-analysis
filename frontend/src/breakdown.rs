use common::{BreakdownType, BreakdownResponse};
use crate::plot::Plot;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BreakdownPlotProps {
  pub breakdown_type: BreakdownType,
  pub word: String,
}

#[function_component(WordInputComponent)]
pub fn word_input_component() -> Html {
    let input_value = use_state(|| String::from(""));
    let word = use_state(|| String::from(""));
    
    let on_input = {
        let input_value = input_value.clone();
        Callback::from(move |e : Event| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                input_value.set(input.value())
            }
        })
    };
    
    let on_click = {
        let input_value = input_value.clone();
        let word = word.clone();
        Callback::from(move |e : MouseEvent| {
            if let Some(_input) = e.target_dyn_into::<web_sys::HtmlButtonElement>() {
                word.set((*input_value).clone())
            }
        })
    };
    
    html! {
        <div style="margin: auto">
            <span>
                <input type="text" value={(*input_value).clone()} onchange={on_input} />
                <button onclick={on_click}>{ "Submit" }</button>
            </span>
            
            <span>
                <BreakdownPlot breakdown_type={BreakdownType::Party} word={(*word).clone()}/>
                <BreakdownPlot breakdown_type={BreakdownType::Gender} word={(*word).clone()}/>
            </span>
        </div>
    }
}

#[function_component(BreakdownPlot)]
pub fn party_breakdown_plot(props: &BreakdownPlotProps) -> Html {
    let data = use_state(|| None);
    let word_state = use_state(|| props.word.clone());
    let loading = use_state(|| false);

    // Request `/api/hello` once
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
                    let uri = format!("/api/breakdown/{}/{}", breakdown_type, word);
                    let resp = Request::get(&uri).send().await.unwrap();
                    let result = {
                        if !resp.ok() {
                            Err(format!(
                                "Error fetching data {} ({})",
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
            let breakdown_data: Vec<BreakdownResponse> = serde_json::from_str(data).unwrap();
            
            html! {
                <Plot
                    breakdown_type={props.breakdown_type.clone()}
                    data={breakdown_data.clone()}
                    loading={*loading}
                />
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    }
}


// #[function_component(GenderBreakdownPlot)]
// pub fn gender_breakdown_plot(props: &BreakdownPlotProps) -> Html {
//     let data = use_state(|| None);
//     let word_state = use_state(|| props.word.clone());
//     let loading = use_state(|| false);
// 
//     // Request `/api/hello` once
//     {
//         let data = data.clone();
//         let loading = loading.clone();
//         let word = props.word.clone();
//         use_effect(move || {
//             if (*word_state) != word {
//                 loading.set(true);
//                 word_state.set(word.clone());
//                 spawn_local(async move {
//                     let uri = format!("/api/breakdown/gender/{}", word);
//                     let resp = Request::get(&uri).send().await.unwrap();
//                     let result = {
//                         if !resp.ok() {
//                             Err(format!(
//                                 "Error fetching data {} ({})",
//                                 resp.status(),
//                                 resp.status_text()
//                             ))
//                         } else {
//                             resp.text().await.map_err(|err| err.to_string())
//                         }
//                     };
//                     data.set(Some(result));
//                     loading.set(false);
//                 });
//             }
// 
//             || {}
//         });
//     }
// 
//     match data.as_ref() {
//         None => {
//             html! {
//                 <div />
//             }
//         }
//         Some(Ok(data)) => {
//             let breakdown_data: Vec<GenderBreakdownResponse> = serde_json::from_str(data).unwrap();
//             
//             html! {
//                 <GenderPlot data={breakdown_data.clone()} loading={*loading} />
//             }
//         }
//         Some(Err(err)) => {
//             html! {
//                 <div>{"Error requesting data from server: "}{err}</div>
//             }
//         }
//     }
// }


#[function_component(Breakdown)]
pub fn breakdown_plot() -> Html {
    html! {
        <WordInputComponent />
    }
}
