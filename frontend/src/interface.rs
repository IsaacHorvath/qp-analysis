use common::BreakdownType;
use crate::breakdown::Breakdown;
use yew::prelude::*;

#[function_component(Interface)]
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
                <Breakdown breakdown_type={BreakdownType::Party} word={(*word).clone()}/>
                <Breakdown breakdown_type={BreakdownType::Gender} word={(*word).clone()}/>
            </span>
        </div>
    }
}
