use yew::prelude::*;

#[function_component(ProvInfoPage)]
pub fn prov_info_page() -> Html {
    html! {
        <div class="info">
            <h2>{"What is this tool?"}</h2>
            <p>{"This tool is a word search that lets you compare how different categories of provincial MMPs use language in the Ontario legislature. Click the search button at the top to try it out!"}</p>
            <p>{"The rest of this info page stuck in development as I am currently working on the federal version of this tool hosted "}<a href="https://housewords.chunkerbunker.cc">{"here"}</a>{"."}</p>
        </div>
    }
}
