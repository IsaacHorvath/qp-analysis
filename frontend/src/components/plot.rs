use common::models::{BreakdownType, DataRequest};
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::JsCast;
use crate::util::{put, OverlaySelection};
use crate::State;
use std::rc::Rc;
use std::cell::RefCell;
use yew_hooks::prelude::use_window_size;
use std::error::Error;

// todo replace with anyhow

pub struct PlotError;

impl<E: Error> From<E> for PlotError {
    fn from(_: E) -> Self {
        PlotError
    }
}

/// Describes a valid plottable engine when implemented.

pub trait Plottable<R>
    where R: PartialEq + std::fmt::Debug + 'static
{
    
    /// Returns a new plot engine.
    
    fn new(breakdown_type: BreakdownType) -> Self;
    
    /// Sets the dynamic properties for this engine. These may need to be reset on rerender.
    
    fn set_props(&mut self, window_width: f64, show_counts: bool, get_speeches: Callback<OverlaySelection>);
    
    /// Loads data into the engine.
    
    fn load_data(&mut self, data: Rc<Vec<R>>);
    
    /// Whether the engine is empty of data.
    
    fn is_empty(&self) -> bool;
    
    /// Returns a sane calculated width for the plot.
    
    fn get_width(&self) -> u32;
    
    /// Returns a sane calculated height for the plot.
    
    fn get_height(&self) -> u32;
    
    /// Returns a heading for the plot.
    
    fn get_heading(&self) -> String;
    
    /// Draws the plot on the given canvas element using plotters.
    
    fn redraw(&mut self, canvas: HtmlCanvasElement, inter_canvas: HtmlCanvasElement) -> Result<(), PlotError>;
    
    /// Handle a mouse hover event. If the user is hovering over a bar/point, this
    /// means drawing an outline around it.
    
    fn hover(&mut self, e: MouseEvent, inter_canvas: HtmlCanvasElement) -> Result<(), PlotError>;
    
    /// Handle a mouse click event. If the user clicked on a bar/point, this means
    /// bringing up the speech overlay for that party/riding/etc.
    
    fn clicked(&self, e: MouseEvent) -> Result<(), PlotError>;
}

/// A source of plot data - either a uri to request data from, or a json string.

#[derive(Clone, PartialEq)]
pub enum PlotSource {
    Uri(String),
    Json(String)
}

/// Properties for the plot component.

#[derive(Properties, PartialEq)]
pub struct PlotProps
{
    
    /// The breakdown type of the plot. Ignored for population graphs.
    
    pub breakdown_type: BreakdownType,
    
    /// The source of the plot data - a uri to request or a json string.
    
    pub source: PlotSource,
    
    /// Whether the plot is currently visible.
    
    pub visible: bool,
    
    /// The word that was most recently searched.
    
    pub word: String,
    
    /// Whether we are showing total counts on this plot. The engine determines
    /// how they will be displayed if this is set to true.
    
    pub show_counts: bool,
    
    /// A callback to bring up the speech overlay when a bar/point is clicked on
    /// the plot
    
    pub get_speeches: Callback<OverlaySelection>,
}

/// A fail state the plot can be in: one of generic, too many requests, or busy.

#[derive(Clone, Copy, PartialEq)]
enum FailState {
    Generic,
    TooMany,
    Busy,
}
use FailState::*;

/// A state the plot can be in: one of showing, loading, and several failure states.

#[derive(PartialEq)]
enum PlotState {
    Showing,
    Loading,
    Failed(FailState),
}
use PlotState::*;

/// A flexible plot component that can request data and create a plot engine to
/// render it.
///
/// If the data request fails (or other errors occur) the plot enters a fail state
/// and the page will need to be refreshed. If a status `204 No Content` is
/// received, this represents a user-cancelled request, and the plot will silently
/// remain loading, awaiting one of the two conditions that would have triggered
/// a cancellation - destruction (the user leaving the page) or a new word.

#[function_component(Plot)]
pub fn plot<P, R>(props: &PlotProps) -> Html
    where
        P: Plottable<R> + 'static,
        R: PartialEq + std::fmt::Debug + for<'a> serde::de::Deserialize<'a> + 'static
{
    let state = use_state(|| Showing);
    let data_state: UseStateHandle<Option<Rc<Vec<R>>>> = use_state(||
        if let PlotSource::Json(json) = &props.source {
            if let Ok(data) = serde_json::from_str::<Vec<R>>(json) {
                Some(Rc::from(data))
            } else {
                state.set(Failed(Generic));
                None
            }
        }
        else {None}
    );
    let word_state = use_state(|| "".to_string());
    let canvas = use_node_ref();
    let inter_canvas = use_node_ref();
    let window_width = use_window_size();
    let app_state = use_context::<State>();
    let engine: Rc<RefCell<P>> = use_mut_ref(|| Plottable::new(props.breakdown_type.clone()));
    
    let heading = if let Ok(mut eng) = engine.try_borrow_mut() {
        eng.set_props(window_width.0, props.show_counts, props.get_speeches.clone());
        eng.get_heading()
    } else {
        state.set(Failed(Generic));
        "".to_string()
    };
    
    {
        let engine = engine.clone();
        let app_state = app_state.clone();
        let visible = props.visible.clone();
        let data_state = data_state.clone();
        let state = state.clone();
        let word = props.word.clone();
        let source = props.source.clone();
        let canvas = canvas.clone();
        let inter_canvas = inter_canvas.clone();
        use_effect(move || {
            if let Ok(mut eng) = engine.try_borrow_mut() {
                if !eng.is_empty() {
                    if let (Some(canvas), Some(inter_canvas)) = (canvas.cast(), inter_canvas.cast()) {
                        eng.redraw(canvas, inter_canvas).unwrap_or_else(|_| { state.set(Failed(Generic)); });
                    }
                }
            }
            
            if let PlotSource::Uri(uri) = source {
                if *word_state != word && visible && *state != Failed(Generic) {
                    state.set(Loading);
                    word_state.set(word.clone());
                    spawn_local(async move {
                        let Some(app_state) = app_state
                            else { state.set(Failed(Generic)); return };
                        let breakdown_request = DataRequest { uuid: app_state.uuid, search: word };
                        let Ok(resp) = put(&format!("api/{}", uri), breakdown_request).await
                            else { state.set(Failed(Generic)); return };
                        
                        state.set(match resp.status() {
                            200 => {
                                let Ok(result) = resp.text().await
                                    else { state.set(Failed(Generic)); return };
                                let Ok(data) = serde_json::from_str::<Vec<R>>(&result)
                                    else { state.set(Failed(Generic)); return };
                                
                                data_state.set(Some(Rc::from(data)));
                                Showing
                            },
                            204 => Loading,
                            429 => Failed(TooMany),
                            503 => Failed(Busy),
                            _ => Failed(Generic),
                        });
                    });
                }
            };

            || {}
        });
    }
    
    let onclick = {
        let engine = engine.clone();
        let state = state.clone();
        Callback::from(move |e : MouseEvent| {
            if let Ok(eng) = engine.try_borrow() {
                eng.clicked(e).unwrap_or_else(|_| { state.set(Failed(Generic)); });
            } else {
                state.set(Failed(Generic));
            }
        })
    };
    
    let onmousemove = {
        let engine = engine.clone();
        let state = state.clone();
        let inter_canvas = inter_canvas.clone();
        Callback::from(move |e : MouseEvent| {
            if let Some(ic) = inter_canvas.cast() {
                if let Ok(mut eng) = engine.try_borrow_mut() {
                    eng.hover(e, ic).unwrap_or_else(|_| { state.set(Failed(Generic)); });
                } else {
                    state.set(Failed(Generic));
                }
            } else {
                state.set(Failed(Generic));
            }
        })
    };
    
    let mut canvas_style = "display: none".to_string();
    let mut inter_canvas_style = "display: none".to_string();
    let mut message_style = "";
    let mut message = "no results found";
    let mut loader_style = "display: none";
    
    match data_state.as_ref() {
        None => {},
        Some(d) => {
            if let Ok(mut eng) = engine.try_borrow_mut() {
                eng.load_data(d.clone());
                if eng.is_empty() { message_style = "display: initial"; }
                else {
                    let width = eng.get_width();
                    let height = eng.get_height();
                    
                    let canvas_opacity = if *state == Loading {"0.25"} else {"1"};
                    canvas_style = format!("opacity: {}; width: {}px; height: {}px", canvas_opacity, width, height);
                    inter_canvas_style = format!("width: {}px; height: {}px", width, height);
                    if let (Some(canvas), Some(inter_canvas)) = (canvas.clone().cast(), inter_canvas.clone().cast()) {
                        eng.redraw(canvas, inter_canvas).unwrap_or_else(|_| { state.set(Failed(Generic)); });
                    }
                }
            } else {
                state.set(Failed(Generic));
            }
        }
    }
    
    match *state {
        Showing => {},
        Loading => {
            loader_style = "display: flex";
        },
        Failed(e) => {
            canvas_style = "display: none".to_string();
            inter_canvas_style = "display: none".to_string();
            message_style = "display: initial";
            message = match e {
                Generic => "an error occurred - please try refreshing",
                TooMany => "too many requests - please slow down",
                Busy => "servers busy - please try again later",
            };
        },
    }
        
    html! {
        if props.visible {
            <div class="plot" >
                <div class="loader-wrapper" style={loader_style}>
                    <div class="loader"/>
                </div>
                <h2 class="plot-heading">{heading}</h2>
                <h3 class="plot-message" style={message_style}>{message}</h3>
                <canvas class="inter-canvas" style={inter_canvas_style} {onclick} {onmousemove} ref={inter_canvas} />
                <canvas class="canvas" style={canvas_style} ref={canvas} />
            </div>
        }
        else {
            <div />
        }
    }
}

/// Returns the CanvasRenderingContext2d object for the given HtmlCanvasElement

pub fn canvas_context(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()
}
