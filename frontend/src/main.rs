use yew::prelude::*;
use yew_router::prelude::*;
use components::navbar::*;
use pages::error_page::error_page;
use pages::interface_page::InterfacePage;
use pages::info_page::InfoPage;
use pages::prov_info_page::ProvInfoPage;
use uuid::Uuid;

mod pages;
mod components;
mod util;

fn switch(routes: Route) -> Html {
    // what kind of error handling options are there in yew?
    // eventually this will be a call to the backend anyway so we can be url agnostic
    let Some(window) = web_sys::window() else {return error_page()};
    let Some(document) = window.document() else {return error_page()};
    let Ok(url) = document.url() else {return error_page()};
    
    let uuid = Uuid::new_v4();
    
    let provincial = url.contains("queen") || url.contains("localhost");
    match (routes, provincial) {
        (Route::Home, false) => html! { 
            <InfoPage {uuid} />
        },
        (Route::Home, true) => html! { 
            <ProvInfoPage />
        },
        (Route::Interface, _) => html! { 
            <InterfacePage {provincial} {uuid} />
        },
        (Route::NotFound, _) => html! {
            <NotFoundPage />
        }
    }
}

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

#[function_component(App)]
fn app() -> Html {
    html! {
        <div>
            <BrowserRouter>
                <Navbar />
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
