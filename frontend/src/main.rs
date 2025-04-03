//! The frontend package for the house words app.
//!
//! A single-page yew wasm app providing a basic interface to search transcripts of
//! the House of Commons (and the Ontario legislature) and generate graphs based on
//! the usage of the search word or phrase. Also provides an info and about page.

use yew::prelude::*;
use yew_router::prelude::*;
use components::navbar::*;
use pages::about_me_page::AboutMePage;
use pages::error_page::error_page;
use pages::interface_page::InterfacePage;
use pages::info_page::InfoPage;
use uuid::Uuid;

mod pages;
mod components;
mod util;

/// A struct to store global frontend state. This is provided to all components in
/// the app via yew's context system.

#[derive(Clone, Debug, PartialEq)]
struct State {
    
    /// A unique user id, generated on page load. This is so we can cancel queries.
    
    uuid: Uuid,
    
    /// A bool to indicate whether we are running the federal or Ontario version.
    
    provincial: bool,
}

/// The router function, matching the route enum to a page.

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { 
            <InfoPage />
        },
        Route::Interface => html! { 
            <InterfacePage />
        },
        Route::About => html! {
            <AboutMePage />
        },
        Route::NotFound => html! {
            <NotFoundPage />
        },
    }
}

/// A simple not found page with a link back to the home page.

#[function_component(NotFoundPage)]
fn not_found_page() -> Html {
    let Some(navigator) = use_navigator() else {return error_page()};
    let onclick = Callback::from(move |_| navigator.push(&Route::Home));
    
    html! {
        <div>
            <div style="display: flex; justify-content: center">
                <h1 style="color: #ffffff">{ "404 not found" }</h1>
            </div>
            <div style="display: flex; justify-content: center">
                <button {onclick} style="background-color: #3f3f3f; border-color: #3f3f3f; border-radius: 10px; color: #ffffff; margin: 5px;">{ "go home" }</button>
            </div>
        </div>
    }
}

/// Our main frontend app.

#[function_component(App)]
fn app() -> Html {
    let Some(window) = web_sys::window() else {return error_page()};
    let Some(document) = window.document() else {return error_page()};
    let Ok(url) = document.url() else {return error_page()};
    
    let state = State {
        uuid: Uuid::new_v4(),
        provincial: url.contains("queen") || url.contains("localhost"),
    };
    
    html! {
        <ContextProvider<State> context={state}>
            <BrowserRouter>
                <Navbar />
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContextProvider<State>>
    }
}

/// Our main entry point. Short and sweet!

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
