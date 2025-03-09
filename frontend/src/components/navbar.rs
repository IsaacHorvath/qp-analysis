use yew::prelude::*;
use yew_router::prelude::*;
//use yew_router::history::History;
use web_sys::History;
use log::info;

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
    
    let interface_style = format!(
        "background-color: #575757; border-color: #575757; color: #{}; border-radius: 10px; padding-block: 3px; padding-inline: 5px",
        if interface {"fee17d"} else {"dddddd"}
    );
    
    let info_style = format!(
        "background-color: #575757; border-color: #575757; color: #{}; border-radius: 10px; padding-block: 3px; padding-inline: 5px",
        if interface {"dddddd"} else {"fee17d"}
    );
    
    html! {
        <div style="background-color: #323232; display: flex; justify-content: center">
            <div style="margin-left: 2%; margin-top: 5px; margin-right: 2%; margin-bottom: 5px">
                <Link<Route> to={Route::Home}>
                    <button style={interface_style}>{"interface"}</button>
                </Link<Route>>
            </div>
            <div style="margin-left: 2%; margin-top: 5px; margin-right: 2%; margin-bottom: 5px">
                <Link<Route> to={Route::Info}>
                    <button style={info_style}>{"info"}</button>
                </Link<Route>>
            </div>
            <div style="margin-left: 2%; margin-top: 5px; margin-right: 2%; margin-bottom: 5px">
                <a href="https://github.com/IsaacHorvath/qp-analysis" target="_blank">
                    <button style="background-color: #575757; border-color: #575757; color: #dddddd; border-radius: 10px; padding-block: 3px; padding-inline: 5px" >{"github"}</button>
                </a>
            </div>
        </div>
    }
}
