use yew::prelude::*;
use yew_router::prelude::*;
use frontend::interface::Interface;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { 
            <Interface />
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
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

//#function

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
