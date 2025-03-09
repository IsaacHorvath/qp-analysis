use yew::prelude::*;
use yew_hooks::prelude::use_window_size;
use common::models::{Speaker, BreakdownType, BreakdownResponse, PopulationResponse, SpeechResponse, SpeakerResponse};
use time::PrimitiveDateTime;
use time::macros::{date, time};
use gloo::utils::body;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use crate::components::breakdown_plot::BreakdownPlot;
use crate::components::population_plot::PopulationPlot;
use crate::components::speech_overlay::{SpeechOverlay, OverlaySelection};
use crate::components::speech_box::SpeechBox;
use crate::pages::info_page_data::*;
use std::collections::HashMap;
use std::cmp::{max, min};
use std::rc::Rc;

#[function_component(InfoPage)]
pub fn info_page() -> Html {
    let speakers = use_state(|| None);
    
    let loading = use_state(|| false);
    let speech_overlay_word = use_state(|| String::from(""));
    let speech_overlay_visible = use_state(|| false);
    let selection = use_state(|| OverlaySelection {breakdown_type: BreakdownType::Party, id: 0, heading: String::from("")});
    
    let window_size = use_window_size();

    {
        let speakers = speakers.clone();
        let loading = loading.clone();
        use_effect(move || {
            if *speakers == None && *loading == false {
                loading.set(true);
                spawn_local(async move {
                    let uri = format!("/api/speakers");
                    let resp = Request::get(&uri).send().await.unwrap();
                    let speaker_response: Vec<SpeakerResponse> = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
                    speakers.set(Some(Rc::new(speaker_response
                        .into_iter()
                        .map(|s| {(s.id, Speaker {first_name: s.first_name, last_name: s.last_name})})
                        .collect::<HashMap<i32, Speaker>>()
                    )));
                    loading.set(false);
                });
            }
    
            || {}
        });
    }
    
    let get_speeches = |w: String| {
        let speech_overlay_word = speech_overlay_word.clone();
        let speech_overlay_visible = speech_overlay_visible.clone();
        let selection = selection.clone();
        Callback::from(move |s: OverlaySelection| {
            selection.set(s);
            body().set_attribute("style", "overflow: hidden; background-color: #121212").unwrap();
            speech_overlay_word.set(w.clone());
            speech_overlay_visible.set(true);
        })
    };
    
    let hide_speech_overlay = {
        let speech_overlay_visible = speech_overlay_visible.clone();
        Callback::from(move |_| {
            body().set_attribute("style", "overflow: auto; background-color: #121212").unwrap();
            speech_overlay_visible.set(false);
        })
    };
    
    let style = format!("max-width: {}px; margin-left: 1%; margin-right: 1%; flex-shrink: 0", min(max(480, window_size.0 as u32), 960));
    let speech_style = format!("max-width: {}px; margin-left: 1%; margin-right: 1%; flex-shrink: 0; display: flex; flex-direction: column", min(max(480, window_size.0 as u32), 960));
    let div_style = format!("max-width: {}px; margin-left: 1%; margin-right: 1%; display: flex", (window_size.0 * 0.98) as u32);
    
    let speech_data = speech_data();
            
    html! {
        <div style="overflow: auto; height: 100%; width: 100%; display: flex; flex-direction: column; align-items: center; color: #dddddd;">
            <h2 style={style.clone()}>{"What is this tool?"}</h2>
            
            <p style={style.clone()}>{"This tool is an interface that lets you compare how different categories of Canadian MPs use language in the House of Commons during the 44th parliament."}</p>
            
            <p style={style.clone()}>{"Start by typing in a word or phrase into the input box, and hitting submit. For example, here's what happens when you try \"pipeline\":"}</p>
            <div style={div_style.clone()}>
                <BreakdownPlot
                    breakdown_type={BreakdownType::Party}
                    data={pipeline_party_data()}
                    show_counts={false}
                    loading={false}
                    window_width={window_size.0} 
                    get_speeches={get_speeches("pipeline".to_owned())}
                />
            </div>
            <p style={style.clone()}>{"As you can see, the Green party uses this word a lot. They have only two members in the house, but those two said \"pipeline\" more than 200 times. Their bar on the chart above is much taller than other parties because it's measuring the number of times they used the word for every 100,000 words they spoke in total."}</p>

            <p style={style.clone()}>{"Members of the Conservative Party said \"pipeline\" more than 650 times, in fact, but this is a much smaller number of mentions in proportion to the 120 members they have. If you're wondering where I got that 650 number, you can check \"show word counts\" at the top of the interface and see a second set of bars on the chart to the right of the original ones. These correspond with an axis on the right side of the chart that measures the total times each party said the word you searched:"}</p>
            <div style={div_style.clone()}>
                <BreakdownPlot
                    breakdown_type={BreakdownType::Party}
                    data={pipeline_party_data()}
                    show_counts={true}
                    loading={false}
                    window_width={window_size.0} 
                    get_speeches={get_speeches("pipeline".to_owned())}
                />
            </div>
            <p style={style.clone()}>{"So what kind of meaning can we draw from this? Well, not too much at this point, and we have to be careful. There are a lot of limitations that come with using raw word counts to make assumptions about the kinds of things a group of people are talking about."}</p>

            <p style={style.clone()}>{"Let's go back to our example. If you've ever followed Canadian federal politics, you might guess that members and supporters of the Green Party and the Conservative Party would have very different opinions on investment in and construction of oil and gas pipelines. So would it be fair to say that the Green Party spends a lot more of their (limited) time criticizing these pipelines than the Conservative Party spends defending them?"}</p>

            <p style={style.clone()}>{"The best way to find out is to click on the bar directly! This will bring up a list of speeches, in chronological order, where a member of that party used the term you searched. Try it on the graph above."}</p>
            
            <p style={style.clone()}>{"If you'd like to read the original Hansard House Debates, or watch the accompanying videos, click the date at the top of the speech:"}</p>
            
            <div style={speech_style.clone()}>
                <SpeechBox
                    name={"Elizabeth May"}
                    start={speech_data.start}
                    end={speech_data.end}
                    link={speech_data.link}
                    text={speech_data.text}
                    word={"pipeline".to_owned()}
                />
            </div>
            
            <p style={style.clone()}>{"In this case the results do line up with what we might expect. Many times they do not, and that's why I encourage you to explore the actual text of the speeches. If you're the kind of Ontarian that doesn't pay much attention to Quebec politics, here is a chance to check out what members of the Bloc might be saying about pipelines."}</p>

            <p style={style.clone()}>{"I mention the Bloc because this brings up another limitation: the data presented here is based on transcripts and "}<em>{"translations"}</em>{" provided by the House of Commons. That means that almost every word spoken in French has been translated into English, and you have to be careful not to make assumptions based on the specific kind of language that a French speaker is using when they don't have full control over the translated result."}</p>

            <h2 style={style.clone()}>{"More Charts"}</h2>

            <p style={style.clone()}>{"There are a few other charts included that you can add to the view by clicking them at the top left:"}</p>
            <p style={style.clone()}>{"--"}</p>
            <p style={style.clone()}>{"The gender breakdown is just like the party breakdown, but shows the words spoken by men, women, and one individual who identifies as Two-Spirit. Here's what the graph looks like for \"mental health\":"}</p>
            <div style={div_style.clone()}>
                <BreakdownPlot
                    breakdown_type={BreakdownType::Gender}
                    data={mental_gender_data()}
                    show_counts={true}
                    loading={false}
                    window_width={window_size.0} 
                    get_speeches={get_speeches("mental health".to_owned())}
                />
            </div>
            <p style={style.clone()}>{"I have chosen to place that one MP - Blake Desjarlais of the NDP - in his own bucket to respect Two-Spirit as a distinct gender identity, but doing my due diligence would involve contacting him to get his preference. So long as he is in a distinct category, the results you may try to read are going to be statistically skewed in many ways, and cannot be said to represent anything about all Two-Spirit people."}</p>

            <p style={style.clone()}>{"The breakdown by province is fairly self-explanatory. Here's what it looks like when you search \"trump\":"}</p>
            <div style={div_style.clone()}>
                <BreakdownPlot
                    breakdown_type={BreakdownType::Province}
                    data={trump_province_data()}
                    show_counts={false}
                    loading={false}
                    window_width={window_size.0} 
                    get_speeches={get_speeches("trump".to_owned())}
                />
            </div>
            <p style={style.clone()}>{"In this case, I might be curious why Manitoba tops the list and Saskatchewan comes closer to the bottom."}</p>

            <p style={style.clone()}>{"The speaker breakdown shows the same kind of information but for individual speakers. It's limited to the top ten results and might look like this:"}</p>
            <p style={style.clone()}>{"--"}</p>
            <p style={style.clone()}>{"The bars are coloured according to their party affiliation, and the results are limited to the top ten hits. This can be a very useful way of finding particular MPs who have taken on particular issues either as representatives of their parties, because of constituent demand, or because of private interest in the subject."}</p>

            <p style={style.clone()}>{"The population density scatterplot shows you how the word usage correlates with how dense an MPs riding is. Large remote ridings appear on the left, followed by rural and then increasingly urban ridings toward the right. Toronto Center, the most population dense riding, will always appear on the far right of the chart."}</p>
            
            <p style={style.clone()}>{"Organizing this information by population density doesn't usually show anything statistically meaningful, but you can use it to check out any outliers. One word that gives an interesting result is \"gaza\":"}</p>
            <div style={div_style.clone()}>
                <PopulationPlot
                    data={gaza_pop_data()}
                    show_counts={false}
                    loading={false}
                    window_width={window_size.0} 
                    get_speeches={get_speeches("pipeline".to_owned())}
                />
            </div>
            <p style={style.clone()}>{"Note that almost all of the people who have actually spoken about Gaza in the house come from urban ridings. Of the few exceptions, way at the left side of the graph, none are Conservative. Each of these dots can be clicked to bring up the speeches made by the MP in that riding."}</p>
            <h2 style={style.clone()}>{"More Limitations"}</h2>
            
            if (*selection).id != 0 {
                if (*loading) == false {
                    <SpeechOverlay
                        selection={(*selection).clone()}
                        word={(*speech_overlay_word).clone()}
                        visible={*speech_overlay_visible}
                        hide={hide_speech_overlay}
                        speakers={Rc::clone((*speakers).as_ref().unwrap())}
                    />
                }
            }
        </div>
    }
}
