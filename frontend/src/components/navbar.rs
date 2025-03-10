use yew::prelude::*;
use yew_router::prelude::*;
use gloo::utils::window;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::Document;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/info")]
    Info,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let location = use_location().unwrap();
    let interface = location.path() == "/";
    
    let window = window();
    let prev_y = use_state(|| 0);
    let navbar_class = use_state(|| "navbar".to_string());
    
    {
        let navbar_class = navbar_class.clone();
        let prev_y = prev_y.clone();
        let scroll = Closure::<dyn FnMut(_)>::new(move |e: web_sys::Event| {
            let rect = e.target().unwrap().dyn_into::<Document>().unwrap().scrolling_element().unwrap();
            
            if rect.scroll_top() > *prev_y {
                navbar_class.set("navbar hide".to_string());
            }else{
                navbar_class.set("navbar".to_string());
            }
            prev_y.set(rect.scroll_top());
        });
        
        window.set_onscroll(scroll.as_ref().dyn_ref());
        scroll.forget();
    }
    
    html! {
        <div class={(*navbar_class).clone()}>
            <div class="navbar-item">
                <Link<Route> to={Route::Home}>
                    <button class={if interface {"button highlight"} else {"button"}} >{"interface"}</button>
                </Link<Route>>
            </div>
            <div class="navbar-item">
                <Link<Route> to={Route::Info}>
                    <button class={if interface {"button"} else {"button highlight"}}>{"info"}</button>
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
