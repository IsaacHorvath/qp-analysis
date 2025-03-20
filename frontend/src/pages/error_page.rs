use yew::prelude::*;

pub fn error_page() -> Html {
    html! {
        <div style="text-align: center">
            <h2 style="color: #ffffff">{ "yikes! something went terribly wrong" }</h2>
            <h2 style="color: #ffffff">{ "try refreshing the page maybe?" }</h2>
        </div>
    }
}
