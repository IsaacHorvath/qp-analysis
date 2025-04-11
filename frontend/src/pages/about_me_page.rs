use yew::prelude::*;

/// A simple page giving details about me.

#[function_component(AboutMePage)]
pub fn about_me_page() -> Html {
    html! {
        <div class="info">
            <h2>{"Who are you?"}</h2>
            
            <p>{"My name is Isaac and I live around Ontario in a converted school bus."}</p>
            
            <div class="info-pic">
                <img src="img/bus.png" alt="my school bus"/>
            </div>
            
            <p>{"When I'm not messing around with data visualization in Rust, I'm exploring the backcountry with my partner, reading, making music with my friends, or helping out at "}<a href="https://www.willowgrove.ca/fraser-lake-camp">{"my old summer camp"}</a>{"."}</p>
            
            <p>{"If you'd like to contact me about this tool, or hire me for any of the above (I rarely get paid to read) please "}<a href="mailto:isaac.horvath@gmail.com">{"send me an email"}</a>{"."}</p>
        </div>
    }
}
