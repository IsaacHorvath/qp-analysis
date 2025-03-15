use yew::prelude::*;
use common::models::{BreakdownType, BreakdownResponse, PopulationResponse};
use crate::components::plot::{Plot, PlotSource};
use crate::components::speech_overlay::OverlaySelection;
use crate::components::population_engine::PopulationEngine;
use crate::components::breakdown_engine::BreakdownEngine;

#[derive(Properties, PartialEq)]
pub struct ChartsProps {
    pub provincial: bool,
    pub word: String,
    pub show_counts: bool,
    pub show_party: bool,
    pub show_gender: bool,
    pub show_province: bool,
    pub show_speaker: bool,
    pub show_pop: bool,
    pub get_speeches: Callback<OverlaySelection>,
}

#[function_component(Charts)]
pub fn charts(props: &ChartsProps) -> Html {
    html! {
        <div class="charts">
            <Plot<BreakdownEngine, BreakdownResponse>
                breakdown_type={BreakdownType::Party}
                source={PlotSource::Uri("breakdown/party".to_string())}
                visible={props.show_party}
                word={props.word.clone()}
                show_counts={props.show_counts}
                get_speeches={&props.get_speeches}
            />
            <Plot<BreakdownEngine, BreakdownResponse>
                breakdown_type={BreakdownType::Gender}
                source={PlotSource::Uri("breakdown/gender".to_string())}
                visible={props.show_gender}
                word={props.word.clone()}
                show_counts={props.show_counts}
                get_speeches={&props.get_speeches}
            />
            if !props.provincial {
                <Plot<BreakdownEngine, BreakdownResponse>
                    breakdown_type={BreakdownType::Province}
                    source={PlotSource::Uri("breakdown/province".to_string())}
                    visible={props.show_province}
                    word={props.word.clone()}
                    show_counts={props.show_counts}
                    get_speeches={&props.get_speeches}
                />
            }
            <Plot<BreakdownEngine, BreakdownResponse>
                breakdown_type={BreakdownType::Speaker}
                source={PlotSource::Uri("breakdown/speaker".to_string())}
                visible={props.show_speaker}
                word={props.word.clone()}
                show_counts={props.show_counts}
                get_speeches={&props.get_speeches}
            />
            if !props.provincial {
                <Plot<PopulationEngine, PopulationResponse>
                    breakdown_type={BreakdownType::Speaker}
                    source={PlotSource::Uri("population".to_string())}
                    visible={props.show_pop}
                    word={props.word.clone()}
                    show_counts={props.show_counts}
                    get_speeches={&props.get_speeches}
                />
            }
        </div>
    }
}
