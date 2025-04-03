use yew::prelude::*;

/// A basic error page, to be displayed in extreme cases when we might not even
/// have the router.

pub fn error_page() -> Html {
    html! {
        <div style="text-align: center">
            <h2 style="color: #ffffff">{ "yikes! something went terribly wrong" }</h2>
            <h2 style="color: #ffffff">{ "try refreshing the page maybe?" }</h2>
        </div>
    }
}
