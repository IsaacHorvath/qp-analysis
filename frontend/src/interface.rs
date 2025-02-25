use common::BreakdownType;
use crate::breakdown::{Breakdown, Args};
use crate::speech_overlay::SpeechOverlay;
use yew::prelude::*;
use gloo::utils::body;
use log::info;

#[function_component(Interface)]
pub fn word_input_component() -> Html {
    let input_value = use_state(|| String::from(""));
    let checkbox_value = use_state(|| false);
    let args = use_state(|| Args { word: String::from(""), show_counts: false } ); // one wrapping arg prop seems necessary to not update twice
    let overlay_visible = use_state(|| false);
    let id = use_state(|| 0);
    
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
    
    let get_speeches = {
        let id = id.clone();
        let overlay_visible = overlay_visible.clone();
        Callback::from(move |new_id| {
            info!("get speeches made it to parent, id {}", new_id);
            id.set(new_id);
            body().set_attribute("style", "overflow: hidden; background-color: #121212").unwrap();
            overlay_visible.set(true);
        })
    };
    
    let hide_overlay = {
        let overlay_visible = overlay_visible.clone();
        info!("hide overlay");
        Callback::from(move |_| {
            body().set_attribute("style", "overflow: auto; background-color: #121212").unwrap();
            overlay_visible.set(false);
        })
    };
    
    html! {
        <div style="background-color: #121212">
            <div >
                <form style="display: flex; flex-wrap: wrap; justify-content: center; color: #ffeba9" onsubmit={submit}>
                    <div style="align-self: center; margin-inline: 10px">
                        <label for="word_input"> {"search term:"}</label>
                        <input type="text" id="word_input" value={(*input_value).clone()} onchange={on_input} style="background-color: #3f3f3f; border-color: #3f3f3f; border-radius: 10px; color: #dddddd; margin: 5px" />
                    </div>
                    <div style="align-self: center; margin-inline: 10px">
                        <label for="show_counts"> {"show total counts:"}</label>
                        <input type="checkbox" id="show_counts" onclick={on_show_counts}/>
                    </div>
                    <div style="align-self: center; margin-inline: 10px">
                        <input type="submit" value="Submit" style="background-color: #575757; border-color: #575757; color: #dddddd; border-radius: 10px; margin: 5px; padding-block: 3px; padding-inline: 5px" />
                    </div>
                </form>
            </div>
            
            <div style="display: flex; flex-wrap: wrap; justify-content: center">
                <Breakdown breakdown_type={BreakdownType::Party} args={(*args).clone()} get_speeches={&get_speeches}/>
                <Breakdown breakdown_type={BreakdownType::Gender} args={(*args).clone()} get_speeches={&get_speeches}/>
                <Breakdown breakdown_type={BreakdownType::Speaker} args={(*args).clone()} get_speeches={&get_speeches}/>
            </div>
            <SpeechOverlay id={*id} word={(*args).word.clone()} visible={*overlay_visible} {hide_overlay}/>
        </div>
    }
}
