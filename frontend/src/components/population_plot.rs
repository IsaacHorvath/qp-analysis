use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use yew::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use gloo::utils::window;
use common::models::{BreakdownType, PopulationResponse};
use crate::components::speech_overlay::OverlaySelection;
use std::cmp::{min, max};
use wasm_bindgen::JsCast;
use log::info;

pub enum PopulationPlotMsg {
    Redraw,
    Clicked(MouseEvent),
    Hover(MouseEvent),
}

#[derive(Properties, PartialEq)]
pub struct PopulationPlotProps {
    pub data: Option<Result<Vec<PopulationResponse>, String>>,
    pub show_counts: bool,
    pub loading: bool,
    pub window_width: f64,
    pub get_speeches: Callback<OverlaySelection>,
}

#[derive(Default, Clone)]
struct CoordMapping {
    x: i32,
    y: i32,
    id: i32,
    name: String,
}

struct PopDensity {
    pub id: i32,
    pub name: String,
    pub pop_density: f64,
    pub colour: String,
    pub count: i32,
    pub score: f32,
}

pub struct PopulationPlot {
    canvas: NodeRef,
    inter_canvas: NodeRef,
    dpr: f64,
    coord_mappings: Vec<CoordMapping>,
    hover_id: i32,
}

fn canvas_context(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap()
}

fn get_width(window_width: &f64) -> u32 {
    min(max(900, (window_width * 0.97) as u32), 1800)
}

fn get_height() -> u32 { 400 } // todo make this responsive or something

impl PopulationPlot {
    fn mouse_mapping(&self, e: MouseEvent) -> CoordMapping {
        let ps = (5.0*self.dpr) as i32;
        let x = (e.offset_x() as f64 * self.dpr) as i32;
        let y = (e.offset_y() as f64 * self.dpr) as i32;
        for m in &self.coord_mappings {
            if x > m.x - ps && x < m.x + ps && y > m.y - ps && y < m.y + ps {
                return m.clone()
            }
        }
        CoordMapping::default()
    }
}

impl Component for PopulationPlot {

    type Message = PopulationPlotMsg;
    type Properties = PopulationPlotProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(PopulationPlotMsg::Redraw);
        PopulationPlot {
            canvas: NodeRef::default(),
            inter_canvas: NodeRef::default(),
            dpr: 1.0,
            coord_mappings: vec![],
            hover_id: 0,
        }       
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|e: MouseEvent| PopulationPlotMsg::Clicked(e));
        let onmousemove = ctx.link().callback(|e: MouseEvent| PopulationPlotMsg::Hover(e));
        let mut canvas_style = "display: none".to_string();
        let mut inter_canvas_style = "display: none".to_string();
        let mut message_style = "display: none";
        let mut message = "no results found".to_string();
        let loader_style = if ctx.props().loading {"display: flex"} else {"display: none"};
        
        let props = &ctx.props();
        match &props.data {
            None => {},
            Some(Err(e)) => {
                message = format!("error: {}", e);
                message_style = "";
            },
            Some(Ok(data)) => {
                let segs = data.len() as u32;
                if segs == 0 || data.iter().filter(|d| d.score > 0.0).count() == 0 { message_style = ""; }
                else {
                    canvas_style = format!("opacity: {}", if ctx.props().loading {"0.25"} else {"1"});
                    inter_canvas_style = "".to_string();
                }
            }
        }
        
        html! (
            <div class="plot" >
                <div class="loader-wrapper" style={loader_style}>
                    <div class="loader"/>
                </div>
                <h2 class="plot-heading">{"population density plot"}</h2>
                <h3 class="plot-message" style={message_style}>{message}</h3>
                <canvas
                    class="inter-canvas"
                    style={inter_canvas_style}
                    {onclick}
                    {onmousemove}
                    ref={self.inter_canvas.clone()}
                />
                <canvas
                    class="canvas"
                    style={canvas_style}
                    ref={self.canvas.clone()}
                />
            </div>
        )
    }
  
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let inter_canvas: HtmlCanvasElement = self.inter_canvas.cast().unwrap();
        
        if let Some(Ok(data)) = &ctx.props().data {
            let segs = data.len();
            if segs > 0 && data.iter().filter(|d| d.score > 0.0).count() > 0 {
                self.dpr = window().device_pixel_ratio().max(1.0);
                let point_size = (5.0*self.dpr) as u32;
                
                match msg {
                    PopulationPlotMsg::Redraw => {
                        let canvas_width = self.dpr * get_width(&ctx.props().window_width) as f64;
                        let canvas_height = self.dpr * get_height() as f64;
                        canvas.set_height(canvas_height as u32);
                        inter_canvas.set_height(canvas_height as u32);
                        canvas.set_width(canvas_width as u32);
                        inter_canvas.set_width(canvas_width as u32);

                        let backend = CanvasBackend::with_canvas_object(canvas).unwrap();
                        let drawing_area = backend.into_drawing_area();
                        let mut label_size = (canvas_width.sqrt() / 2.5) as u32;
                        
                        let data = data.iter().map(|r| { PopDensity {
                            id: r.id,
                            name: r.name.clone(),
                            pop_density: r.population as f64 / r.area,
                            colour: r.colour.clone(),
                            score: r.score,
                            count: r.count,
                        }}).collect::<Vec<PopDensity>>();
                        
                        let show_counts = ctx.props().show_counts;
                        let x_max = data.iter().map(|r| { r.pop_density }).max_by(|a, b| {a.total_cmp(b)}).unwrap();
                        let y_max = data.iter().map(|r| { r.score }).max_by(|a, b| {a.total_cmp(b)}).unwrap();
                        let c_max = data.iter().map(|r| { r.count }).max_by(|a, b| a.cmp(b)).unwrap();

                        let bold_line = hex::decode("97948f").unwrap();
                        let light_line = hex::decode("67635c").unwrap();

                        label_size = max((label_size as f64 * (1.0 + (self.dpr * 0.1))) as u32, (8.0 * self.dpr) as u32);
                        let desc_style = TextStyle::from(("sans-serif", (16.0 * self.dpr) as u32).into_font()).color(&WHITE);
                        self.coord_mappings = vec![];
                        
                        if !show_counts {
                            let mut chart = ChartBuilder::on(&drawing_area)
                                .x_label_area_size((50.0 * self.dpr) as u32)
                                .y_label_area_size((70.0 * self.dpr) as u32)
                                .build_cartesian_2d((0.02..x_max).log_scale(), 0.0..y_max).unwrap();
                            
                            chart.configure_mesh()
                                .x_desc("population per square kilometer") 
                                .x_label_style(TextStyle::from(("sans-serif", label_size).into_font()).color(&WHITE))   
                                .y_label_style(TextStyle::from(("sans-serif", label_size).into_font()).color(&WHITE))
                                .axis_desc_style(desc_style)
                                .bold_line_style(RGBColor(bold_line[0], bold_line[1], bold_line[2]))
                                .light_line_style(RGBColor(light_line[0], light_line[1], light_line[2]))
                                .y_desc("word count per 100,000")
                                .draw()
                                .unwrap();
                                
                            chart.draw_series(data.iter().map(|r| {
                                let rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                                Circle::new((r.pop_density, r.score), point_size, RGBColor(rgb[0], rgb[1], rgb[2]).filled())
                            }))
                            .unwrap();
                            
                            for r in data.iter() {
                                let p = chart.backend_coord(&(r.pop_density, r.score));
                                self.coord_mappings.push(CoordMapping { x: p.0, y: p.1, id: r.id, name: r.name.clone() });
                            };
                        }
                        else {
                            let mut chart= ChartBuilder::on(&drawing_area)
                                .x_label_area_size((50.0 * self.dpr) as u32)
                                .y_label_area_size((70.0 * self.dpr) as u32)
                                .build_cartesian_2d((0.02..x_max).log_scale(), 0..c_max).unwrap();
                            
                            chart.configure_mesh()
                                .x_desc("population per square kilometer") 
                                .x_label_style(TextStyle::from(("sans-serif", label_size).into_font()).color(&WHITE))   
                                .y_label_style(TextStyle::from(("sans-serif", label_size).into_font()).color(&WHITE))
                                .axis_desc_style(desc_style)
                                .bold_line_style(RGBColor(bold_line[0], bold_line[1], bold_line[2]))
                                .light_line_style(RGBColor(light_line[0], light_line[1], light_line[2]))
                                .y_desc("total word count")
                                .y_label_formatter(&|v| { format!("{}", *v as u32) })
                                .draw()
                                .unwrap();
                                
                            chart.draw_series(data.iter().map(|r| {
                                let rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                                Circle::new((r.pop_density, r.count), point_size, RGBColor(rgb[0], rgb[1], rgb[2]).filled())
                            }))
                            .unwrap();
                            
                            for r in data.iter() {
                                let p = chart.backend_coord(&(r.pop_density, r.count));
                                self.coord_mappings.push(CoordMapping { x: p.0, y: p.1, id: r.id, name: r.name.clone() });
                            };
                        }
                    },
                    PopulationPlotMsg::Clicked(e) => {
                        let cm = self.mouse_mapping(e);
                        
                        if cm.id > 0 {
                            let heading = data.iter().filter(|r| r.id == cm.id).next().unwrap().name.clone();
                            ctx.props().get_speeches.emit(OverlaySelection {breakdown_type: BreakdownType::Speaker, id: cm.id, heading});
                        }
                    },
                    PopulationPlotMsg::Hover(e) => {
                        if !ctx.props().loading {
                            let cm = self.mouse_mapping(e);
                            
                            if cm.id != self.hover_id {
                                self.hover_id = cm.id;
                                let context = canvas_context(&inter_canvas);
                                context.clear_rect(0.0, 0.0, inter_canvas.width() as f64, inter_canvas.width() as f64);
                                
                                let left = (cm.x - 10) as f64;
                                let bottom = (cm.y - 10) as f64;
                                if cm.id != 0  {
                                    context.begin_path();
                                    context.arc(cm.x as f64, cm.y as f64, point_size as f64, 0.0, 2.0*std::f64::consts::PI).unwrap();
                                    context.set_line_width(3.0);
                                    context.set_stroke_style_str("#fee17d");
                                    context.stroke();
                                    
                                    context.set_line_width(0.5);
                                    info!("{}px sans-serif", (12.0 * self.dpr) as i32);
                                    context.set_font(&format!("{}px sans-serif", (12.0 * self.dpr) as i32));
                                    let ts = context.measure_text(&cm.name).unwrap();
                                    let h = ts.font_bounding_box_ascent() + 2.0;
                                    context.set_fill_style_str("#121212");
                                    context.fill_rect(left - 2.0, bottom - h, ts.width() + 4.0, h + 4.0);
                                    context.set_fill_style_str("#fee17d");
                                    context.fill_text(&cm.name, left, bottom).unwrap();
                                }
                            }
                        }
                    },
                };
            }
        };
        false
    }
    
    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        ctx.link().send_message(PopulationPlotMsg::Redraw);
        true
    }
}
