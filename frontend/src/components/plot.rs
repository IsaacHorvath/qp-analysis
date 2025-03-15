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
    fn redraw(&mut self, canvas: HtmlCanvasElement, inter_canvas: HtmlCanvasElement);
    fn hover(&mut self, e: MouseEvent, inter_canvas: HtmlCanvasElement);
    fn clicked(&self, e: MouseEvent);
}

pub fn canvas_context(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap()
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
    let data_state: UseStateHandle<Option<Result<Rc<Vec<R>>, String>>> = use_state(||
        if let PlotSource::Json(json) = &props.source {
            Some(Ok(Rc::from(serde_json::from_str::<Vec<R>>(json).unwrap())))
        }
        else {None}
    );
    let loading = use_state(|| false);
    let word_state = use_state(|| "".to_string());
    let canvas = use_node_ref();
    let inter_canvas = use_node_ref();
    let window_width = use_window_size();
    let engine: Rc<RefCell<P>> = use_mut_ref(|| Plottable::new(props.breakdown_type.clone()));
    engine.borrow_mut().set_props(window_width.0, props.show_counts, props.get_speeches.clone());
    
    {
        let engine = engine.clone();
        let visible = props.visible.clone();
        let data_state = data_state.clone();
        let loading = loading.clone();
        let word = props.word.clone();
        let source = props.source.clone();
        let canvas = canvas.clone();
        let inter_canvas = inter_canvas.clone();
        use_effect(move || {
            if let PlotSource::Uri(uri) = source {
                if (*word_state) != word && visible {
                    loading.set(true);
                    word_state.set(word.clone());
                    spawn_local(async move {
                        let breakdown_request = DataRequest { search: word };
                        let resp = Request::put(&format!("api/{}", uri))
                            .header("Content-Type", "application/json")
                            .json(&breakdown_request).expect("couldn't create request body")
                            .send().await.unwrap();
                            
                        let mut result = resp.text().await.map_err(|err| err.to_string());
                        if !resp.ok() { 
                            result = match result {Ok(e) => Err(e), e => e};
                        }
                        
                        data_state.set(Some( match result {
                            Ok(j) => Ok(Rc::from(serde_json::from_str::<Vec<R>>(&j).unwrap())),
                            Err(e) => Err(e),
                        }));
                        loading.set(false);
                    });
                }
            };
            
            if !engine.borrow().is_empty() {
                if let (Some(canvas), Some(inter_canvas)) = (canvas.cast(), inter_canvas.cast()) {
                    engine.borrow_mut().redraw(canvas, inter_canvas);
                }
            }

            || {}
        });
    }
    
    let onclick = {
        let engine = engine.clone();
        Callback::from(move |e : MouseEvent| { engine.borrow().clicked(e); })
    };
    
    let onmousemove = {
        let engine = engine.clone();
        let inter_canvas = inter_canvas.clone();
        Callback::from(move |e : MouseEvent| { engine.borrow_mut().hover(e, inter_canvas.cast().unwrap()); })
    };
    
    let mut canvas_style = "display: none".to_string();
    let mut inter_canvas_style = "display: none".to_string();
    let mut message_style = "";
    let mut message = "no results found".to_string();
    let loader_style = if *loading {"display: flex"} else {"display: none"};
    
    match data_state.as_ref() {
        None => {},
        Some(Err(e)) => {
            message = format!("error: {}", e);
            message_style = "";
        },
        Some(Ok(d)) => {
            engine.borrow_mut().load_data(d.clone());
            if engine.borrow().is_empty() { message_style = "display: initial"; }
            else {
                let width = engine.borrow().get_width();
                let height = engine.borrow().get_height();
                
                let canvas_opacity = if *loading {"0.25"} else {"1"};
                canvas_style = format!("opacity: {}; width: {}px; height: {}px", canvas_opacity, width, height);
                inter_canvas_style = format!("width: {}px; height: {}px", width, height);
                if let Some(canvas) = canvas.clone().cast() {
                    if let Some(inter_canvas) = inter_canvas.clone().cast() {
                        engine.borrow_mut().redraw(canvas, inter_canvas);
                    }    
                }
            }
        }
    }
    
    let heading = engine.borrow().get_heading();
        
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
