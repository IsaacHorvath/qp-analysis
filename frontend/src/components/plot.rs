use common::models::{BreakdownType, DataRequest};
use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::JsCast;
use crate::components::speech_overlay::OverlaySelection;
use std::rc::Rc;
use std::cell::RefCell;
use yew_hooks::prelude::use_window_size;
use std::error::Error;

pub fn canvas_context(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()
}

pub struct PlotError;

impl<E: Error> From<E> for PlotError {
    fn from(_: E) -> Self {
        PlotError
    }
}

pub trait Plottable<R>
    where R: PartialEq + std::fmt::Debug + 'static
{
    fn new(breakdown_type: BreakdownType) -> Self;
    fn set_props(&mut self, window_width: f64, show_counts: bool, get_speeches: Callback<OverlaySelection>);
    fn load_data(&mut self, data: Rc<Vec<R>>);
    fn is_empty(&self) -> bool;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_heading(&self) -> String;
    fn redraw(&mut self, canvas: HtmlCanvasElement, inter_canvas: HtmlCanvasElement) -> Result<(), PlotError>;
    fn hover(&mut self, e: MouseEvent, inter_canvas: HtmlCanvasElement) -> Result<(), PlotError>;
    fn clicked(&self, e: MouseEvent) -> Result<(), PlotError>;
}

#[derive(Clone, PartialEq)]
pub enum PlotSource {
    Uri(String),
    Json(String)
}

#[derive(Properties, PartialEq)]
pub struct PlotProps
{
    pub breakdown_type: BreakdownType,
    pub source: PlotSource,
    pub visible: bool,
    pub word: String,
    pub show_counts: bool,
    pub get_speeches: Callback<OverlaySelection>,
}

#[function_component(Plot)]
pub fn plot<P, R>(props: &PlotProps) -> Html
    where
        P: Plottable<R> + 'static,
        R: PartialEq + std::fmt::Debug + for<'a> serde::de::Deserialize<'a> + 'static
{
    let failed = use_state(|| false);
    let data_state: UseStateHandle<Option<Rc<Vec<R>>>> = use_state(||
        if let PlotSource::Json(json) = &props.source {
            if let Ok(data) = serde_json::from_str::<Vec<R>>(json) {
                Some(Rc::from(data))
            } else {
                failed.set(true);
                None
            }
        }
        else {None}
    );
    let loading = use_state(|| false);
    let word_state = use_state(|| "".to_string());
    let canvas = use_node_ref();
    let inter_canvas = use_node_ref();
    let window_width = use_window_size();
    let engine: Rc<RefCell<P>> = use_mut_ref(|| Plottable::new(props.breakdown_type.clone()));
    
    let heading = if let Ok(mut eng) = engine.try_borrow_mut() {
        eng.set_props(window_width.0, props.show_counts, props.get_speeches.clone());
        eng.get_heading()
    } else {
        failed.set(true);
        "".to_string()
    };
    
    {
        let engine = engine.clone();
        let visible = props.visible.clone();
        let data_state = data_state.clone();
        let loading = loading.clone();
        let failed = failed.clone();
        let word = props.word.clone();
        let source = props.source.clone();
        let canvas = canvas.clone();
        let inter_canvas = inter_canvas.clone();
        use_effect(move || {
            || -> Result<(), PlotError> {
                if !engine.try_borrow()?.is_empty() {
                    engine.try_borrow_mut()?
                        .redraw(canvas.cast().ok_or(PlotError)?, inter_canvas.cast().ok_or(PlotError)?)?;
                }
                Ok(())
            }().unwrap_or(failed.set(true));
            
            if let PlotSource::Uri(uri) = source {
                if (*word_state) != word && visible {
                    loading.set(true);
                    word_state.set(word.clone());
                    let failed = failed.clone();
                    spawn_local(async move {
                        let breakdown_request = DataRequest { search: word };
                        let Ok(resp) = Request::put(&format!("api/{}", uri))
                            .header("Content-Type", "application/json")
                            .json(&breakdown_request).expect("couldn't create request body")
                            .send().await else { failed.set(true); return };
                            
                        if !resp.ok() { failed.set(true); return }
                        let Ok(result) = resp.text().await else { failed.set(true); return };
                        let Ok(data) = serde_json::from_str::<Vec<R>>(&result) else { failed.set(true); return };
                        
                        data_state.set(Some(Rc::from(data)));
                        loading.set(false);
                    });
                }
            };

            || {}
        });
    }
    
    let onclick = {
        let engine = engine.clone();
        let failed = failed.clone();
        Callback::from(move |e : MouseEvent| {
            if let Ok(eng) = engine.try_borrow() {
                eng.clicked(e).unwrap_or(failed.set(true));
            } else {
                failed.set(true)
            }
        })
    };
    
    let onmousemove = {
        let engine = engine.clone();
        let failed = failed.clone();
        let inter_canvas = inter_canvas.clone();
        Callback::from(move |e : MouseEvent| {
            if let Some(ic) = inter_canvas.cast() {
                if let Ok(mut eng) = engine.try_borrow_mut() {
                    eng.hover(e, ic).unwrap_or(failed.set(true));
                } else {
                    failed.set(true);
                }
            } else {
                failed.set(true);
            }
        })
    };
    
    let mut canvas_style = "display: none".to_string();
    let mut inter_canvas_style = "display: none".to_string();
    let mut message_style = "";
    let mut message = "no results found".to_string();
    let loader_style = if *loading {"display: flex"} else {"display: none"};
    
    match data_state.as_ref() {
        None => {},
        Some(d) => {
            if let Ok(mut eng) = engine.try_borrow_mut() {
                eng.load_data(d.clone());
                if eng.is_empty() { message_style = "display: initial"; }
                else {
                    let width = eng.get_width();
                    let height = eng.get_height();
                    
                    let canvas_opacity = if *loading {"0.25"} else {"1"};
                    canvas_style = format!("opacity: {}; width: {}px; height: {}px", canvas_opacity, width, height);
                    inter_canvas_style = format!("width: {}px; height: {}px", width, height);
                    if let (Some(canvas), Some(inter_canvas)) = (canvas.clone().cast(), inter_canvas.clone().cast()) {
                        eng.redraw(canvas, inter_canvas).unwrap_or(failed.set(true));
                    } else {
                        failed.set(true);
                    }
                }
            } else {
                failed.set(true)
            }
        }
    }
    
    if *failed {
        canvas_style = "display: none".to_string();
        inter_canvas_style = "display: none".to_string();
        message = "an error occured".to_string();
        message_style = "display: initial";
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
