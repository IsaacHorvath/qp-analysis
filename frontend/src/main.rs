use yew::prelude::*;
use yew_router::prelude::*;
use components::navbar::*;
use pages::interface_page::InterfacePage;
use pages::info_page::InfoPage;

mod pages;
mod components;

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { 
            <InterfacePage />
        },
        Route::Info => html! { 
            <InfoPage />
        },
        Route::NotFound => html! {
            <NotFoundPage />
        }
    }
}

#[function_component(NotFoundPage)]
fn not_found_page() -> Html {
    let navigator = use_navigator().unwrap();

    let onclick = Callback::from(move |_| navigator.push(&Route::Home));
    html! {
        <div>
            <div style="display: flex; justify-content: center">
                <h1 style="color: #ffffff">{ "404 not found" }</h1>
            </div>
            <div style="display: flex; justify-content: center">
                <button {onclick} style="background-color: #3f3f3f; border-color: #3f3f3f; border-radius: 10px; color: #ffffff; margin: 5px;">{ "Go Home" }</button>
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
