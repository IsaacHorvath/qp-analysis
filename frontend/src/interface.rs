use common::BreakdownType;
use crate::breakdown::{Breakdown, Args};
use yew::prelude::*;
use log::info;

#[function_component(Interface)]
pub fn word_input_component() -> Html {
    let input_value = use_state(|| String::from(""));
    let checkbox_value = use_state(|| false);
    let args = use_state(|| Args { word: String::from(""), show_counts: false } ); // one wrapping arg prop seems necessary to not update twice
    
    let on_input = {
        let input_value = input_value.clone();
        Callback::from(move |e : Event| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                input_value.set(input.value());
            }
        })
    };
    
    let on_show_counts = {
        let checkbox_value = checkbox_value.clone();
        Callback::from(move |e : MouseEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                checkbox_value.set(input.checked());
            }
        })
    };
    
    let submit = {
        let input_value = input_value.clone();
        let args = args.clone();
        Callback::from(move |e : SubmitEvent| {
            e.prevent_default();
            args.set(Args { word: (*input_value).clone(), show_counts: *checkbox_value });
        })
    };
    
    html! {
        <div style="background-color: #121212">
            // <span style="display: flex; flex-wrap: wrap; justify-content: center">
            //     <input type="text" value={(*input_value).clone()} onchange={on_input} style="background-color: #282828; border-color: #282828; border-radius: 10px; color: #ffffff; margin: 5px" />
            //     <button onclick={submit} style="background-color: #3f3f3f; border-color: #3f3f3f; border-radius: 10px; color: #ffffff; margin: 5px;">{ "Submit" }</button>
            // </span>
            
            <div style="display: flex; flex-wrap: wrap; justify-content: center; color: #ffeba9">
                <form onsubmit={submit}>
                    <span style="margin-inline: 10px">
                        <label for="word_input"> {"search term:"}</label>
                        <input type="text" id="word_input" value={(*input_value).clone()} onchange={on_input} style="background-color: #3f3f3f; border-color: #3f3f3f; border-radius: 10px; color: #ffffff; margin: 5px" />
                    </span>
                    <span style="margin-inline: 10px">
                        <label for="show_counts"> {"show total counts:"}</label>
                        <input type="checkbox" id="show_counts" onclick={on_show_counts}/>
                    </span>
                    <span style="margin-inline: 10px">
                        <input type="submit" value="Submit" style="background-color: #575757; border-color: #575757; color: #ffffff; border-radius: 10px; margin: 5px; padding: 3px" />
                    </span>
                </form>
            </div>
            
            <div style="display: flex; flex-wrap: wrap; justify-content: center">
                <Breakdown breakdown_type={BreakdownType::Party} args={(*args).clone()} />
                <Breakdown breakdown_type={BreakdownType::Gender} args={(*args).clone()} />
                <Breakdown breakdown_type={BreakdownType::Speaker} args={(*args).clone()} />
                // <Breakdown breakdown_type={BreakdownType::Gender} word={(*args).0.clone()} show_counts={(*args).1} />
                // <Breakdown breakdown_type={BreakdownType::Speaker} word={(*args).0.clone()} show_counts={(*args).1} />
            </div>
        </div>
    }
}
