use common::models::BreakdownType;
use yew::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::JsCast;
use crate::components::speech_overlay::OverlaySelection;
use std::rc::Rc;

pub enum PlotMsg {
    Redraw,
    Clicked(MouseEvent),
    Hover(MouseEvent),
}

pub trait Plottable<R>
    where R: PartialEq + std::fmt::Debug + 'static
{
    fn new(data: Rc<Vec<R>>, breakdown_type: BreakdownType, window_width: f64, show_counts: bool, get_speeches: Callback<OverlaySelection>) -> Self;
    fn load_data(&mut self, data: Rc<Vec<R>>);
    fn is_empty(&self) -> bool;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_heading(&self) -> String;
    fn redraw(&mut self, canvas: HtmlCanvasElement, inter_canvas: HtmlCanvasElement);
    fn hover(&mut self, e: MouseEvent, inter_canvas: HtmlCanvasElement);
    fn clicked(&self, e: MouseEvent);
}

#[derive(Properties, PartialEq)]
pub struct PlotProps<R>
    where R: PartialEq + std::fmt::Debug + 'static
{
    pub breakdown_type: BreakdownType,
    pub data: Option<Result<Rc<Vec<R>>, String>>,
    pub loading: bool,
    pub window_width: f64,
    pub show_counts: bool,
    pub get_speeches: Callback<OverlaySelection>,
}

pub struct Plot<P, R>
    where P: Plottable<R> + 'static,
    R: PartialEq + std::fmt::Debug + 'static
{
    engine: P,
    canvas: NodeRef,
    inter_canvas: NodeRef,
    _dummy: Option<R>,
}

pub fn canvas_context(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap()
}

impl<P, R> Component for Plot<P, R>
    where P: Plottable<R> + 'static,
    R: PartialEq + std::fmt::Debug + 'static
{
    type Message = PlotMsg;
    type Properties = PlotProps<R>;

    fn create(ctx: &Context<Self>) -> Self {
        let data = if let Some(Ok(d)) = &ctx.props().data {d.clone()} else {Rc::from(vec![])};
        ctx.link().send_message(PlotMsg::Redraw);
        
        let props = ctx.props();
        Plot {
            engine: Plottable::new(data, props.breakdown_type.clone(), props.window_width, props.show_counts, props.get_speeches.clone()),
            canvas: NodeRef::default(),
            inter_canvas: NodeRef::default(),
            _dummy: None,
        }       
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|e: MouseEvent| PlotMsg::Clicked(e));
        let onmousemove = ctx.link().callback(|e: MouseEvent| PlotMsg::Hover(e));
        let mut canvas_style = "display: none".to_string();
        let mut inter_canvas_style = "display: none".to_string();
        let mut message_style = "";
        let mut message = "no results found".to_string();
        let loader_style = if ctx.props().loading {"display: flex"} else {"display: none"};
        
        let props = &ctx.props();
        match &props.data {
            None => {},
            Some(Err(e)) => {
                message = format!("error: {}", e);
                message_style = "";
            },
            Some(Ok(_)) => {
                if self.engine.is_empty() { message_style = "display: initial"; }
                else {
                    let width = self.engine.get_width();
                    let height = self.engine.get_height();
                    
                    let canvas_opacity = if ctx.props().loading {"0.25"} else {"1"};
                    canvas_style = format!("opacity: {}; width: {}px; height: {}px", canvas_opacity, width, height);
                    inter_canvas_style = format!("width: {}px; height: {}px", width, height);
                }
            }
        }
        
        let heading = self.engine.get_heading();
        
        html! (
            <div class="plot" >
                <div class="loader-wrapper" style={loader_style}>
                    <div class="loader"/>
                </div>
                <h2 class="plot-heading">{heading}</h2>
                <h3 class="plot-message" style={message_style}>{message}</h3>
                <canvas class="inter-canvas" style={inter_canvas_style} {onclick} {onmousemove} ref={self.inter_canvas.clone()} />
                <canvas class="canvas" style={canvas_style} ref={self.canvas.clone()} />
            </div>
        )
    }
  
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let inter_canvas: HtmlCanvasElement = self.inter_canvas.cast().unwrap();
        
        if !self.engine.is_empty() {
            match (msg, &ctx.props().loading) {
                (PlotMsg::Redraw, _) => self.engine.redraw(canvas, inter_canvas),
                (PlotMsg::Hover(e), false) => self.engine.hover(e, inter_canvas),
                (PlotMsg::Clicked(e), false) => self.engine.clicked(e),
                (_, _) => ()
            }
        }
        false
    }
    
    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let data = if let Some(Ok(d)) = &ctx.props().data {d.clone()} else {Rc::from(vec![])};
        //log::info!("changed: {}", data.len());
        self.engine.load_data(data);
        ctx.link().send_message(PlotMsg::Redraw);
        true
    }
}
