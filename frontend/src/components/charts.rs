use crate::components::breakdown_engine::BreakdownEngine;
use crate::components::plot::{Plot, PlotSource};
use crate::components::population_engine::PopulationEngine;
use crate::pages::error_page::error_page;
use crate::util::OverlaySelection;
use crate::State;
use common::models::{BreakdownResponse, BreakdownType, PopulationResponse};
use yew::prelude::*;

/// Properties for the plot container component.

#[derive(Properties, PartialEq)]
pub struct ChartsProps {
    /// The word the user last searched.
    pub word: String,

    /// Whether the plots are showing total counts or not.
    pub show_counts: bool,

    /// Whether the party breakdown chart is showing.
    pub show_party: bool,

    /// Whether the gender breakdown chart is showing.
    pub show_gender: bool,

    /// Whether the province breakdown chart is showing.
    pub show_province: bool,

    /// Whether the class breakdown chart is showing.
    pub show_class: bool,

    /// Whether the speaker breakdown chart is showing.
    pub show_speaker: bool,

    /// Whether the population density graph is showing.
    pub show_pop: bool,

    /// A callback to bring up the speech overlay for a plot bar or point.
    pub get_speeches: Callback<OverlaySelection>,
}

/// A simple plot container component.

#[function_component(Charts)]
pub fn charts(props: &ChartsProps) -> Html {
    let Some(app_state) = use_context::<State>() else {
        return error_page();
    };

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

            if !app_state.provincial {
                <Plot<BreakdownEngine, BreakdownResponse>
                    breakdown_type={BreakdownType::Province}
                    source={PlotSource::Uri("breakdown/province".to_string())}
                    visible={props.show_province}
                    word={props.word.clone()}
                    show_counts={props.show_counts}
                    get_speeches={&props.get_speeches}
                />
                <Plot<BreakdownEngine, BreakdownResponse>
                    breakdown_type={BreakdownType::Class}
                    source={PlotSource::Uri("breakdown/class".to_string())}
                    visible={props.show_class}
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

            if !app_state.provincial {
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
