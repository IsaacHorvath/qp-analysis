use yew::prelude::*;
use yew_router::prelude::*;
use gloo::utils::window;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::Document;
use crate::error_page;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/search")]
    Interface,
    #[at("/about")]
    About,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(Navbar)]
pub fn navbar() -> Html {
    // todo better way to get where we are
    let Some(location) = use_location() else { return error_page() };
    let info = match location.path() {
        "/" => Route::Home,
        "/search" => Route::Interface,
        "/about" => Route::About,
        _ => Route::NotFound,
    };
    
    let window = window();
    let prev_s = use_state(|| 0);
    let navbar_class = use_state(|| "navbar".to_string());
    
    {
        let navbar_class = navbar_class.clone();
        let prev_s = prev_s.clone();
        let scroll = Closure::<dyn FnMut(_)>::new(move |e: web_sys::Event| {
            let Some(target) = e.target() else { return; };
            let Ok(document) = target.dyn_into::<Document>() else { return; };
            let Some(scrolling_element) = document.scrolling_element() else { return; };
            let s = scrolling_element.scroll_top();
            
            if s > *prev_s {
                navbar_class.set("navbar hide".to_string());
            }
            else if s < *prev_s {
                navbar_class.set("navbar".to_string());
            }
            prev_s.set(s);
        });
        
        window.set_onscroll(scroll.as_ref().dyn_ref());
        scroll.forget();
    }
    
    html! {
        <div class={(*navbar_class).clone()}>
            <div class="navbar-item">
                <Link<Route> to={Route::Interface}>
                    <button class={if info == Route::Interface {"button highlight"} else {"button"}} >{"search"}</button>
                </Link<Route>>
            </div>
            <div class="navbar-item">
                <Link<Route> to={Route::Home}>
                    <button class={if info == Route::Home {"button highlight"} else {"button"}}>{"info"}</button>
                </Link<Route>>
            </div>
            <div class="navbar-item">
                <Link<Route> to={Route::About}>
                    <button class={if info == Route::About {"button highlight"} else {"button"}}>{"about"}</button>
                </Link<Route>>
            </div>
            <div class="navbar-item">
                <a href="https://github.com/IsaacHorvath/qp-analysis" target="_blank">
                    <button class="button" >{"github"}</button>
                </a>
            </div>
        </div>
    }
}
