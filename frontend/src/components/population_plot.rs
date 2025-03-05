use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use yew::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use gloo::utils::window;
use common::models::{BreakdownType, PopulationResponse};
use crate::components::speech_overlay::OverlaySelection;
use std::cmp::{max, min};
use wasm_bindgen::JsCast;
use log::info;

pub enum PopulationPlotMsg {
    Redraw,
    Clicked(MouseEvent),
    Hover(MouseEvent),
}

#[derive(Properties, PartialEq)]
pub struct PopulationPlotProps {
    pub data: Vec<PopulationResponse>,
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
        
        html! (
            <div style="margin: 0.5%; overflow: auto; image-rendering: pixelated; border: 2px solid #fee17d; border-radius: 20px; padding: 1%; width: fit-content; display: grid" >
                <canvas style="grid-column: 1; grid-row: 1; z-index: 10; width: 99%; height: 99%" {onclick} {onmousemove} ref = {self.inter_canvas.clone()}/>
                <canvas style="grid-column: 1; grid-row: 1; width: 99%; height: 99%" ref = {self.canvas.clone()}/>
            </div>
        )
    }
  
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let inter_canvas: HtmlCanvasElement = self.inter_canvas.cast().unwrap();
        
        let window_width = ctx.props().window_width - 40.0;
        let width = min(max(900, window_width as u32), 1800);
        let height: u32 = 500;
        
        self.dpr = window().device_pixel_ratio();
        let mut canvas_width = width;
        let mut canvas_height = height;
        if self.dpr >= 1.0 {
            canvas_width = (self.dpr * canvas_width as f64) as u32;
            canvas_height = (self.dpr * canvas_height as f64) as u32;
        }
        
        let point_size = (5.0*self.dpr) as u32;
        
        match msg {
            PopulationPlotMsg::Redraw => {
                if ctx.props().loading {
                    canvas.set_attribute("style", &format!("opacity: 0.25; grid-column: 1; grid-row: 1; width: {}px; height: {}px", width, height)).expect("couldn't set opacity");
                }
                else {
                    canvas.set_attribute("style", &format!("opacity: 1; grid-column: 1; grid-row: 1; width: {}px; height: {}px", width, height)).expect("couldn't set opacity");
                }
                inter_canvas.set_attribute("style", &format!("grid-column: 1; grid-row: 1; z-index: 10; width: {}px; height: {}px", width, height)).expect("couldn't set dimensions");
                
                canvas.set_height(canvas_height);
                inter_canvas.set_height(canvas_height);
                canvas.set_width(canvas_width);
                inter_canvas.set_width(canvas_width);

                let backend = CanvasBackend::with_canvas_object(canvas).unwrap();
                let drawing_area = backend.into_drawing_area();
                let mut label_size = (window_width.sqrt() / 2.5 * self.dpr) as u32;
                
                let data = ctx.props().data.iter().map(|r| { PopDensity {
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

                let mut chart= ChartBuilder::on(&drawing_area)
                    .x_label_area_size((40.0 * self.dpr) as u32)
                    .y_label_area_size((60.0 * self.dpr) as u32)
                    .right_y_label_area_size(if show_counts {(60.0 * self.dpr) as u32} else {0})
                    .caption(&format!("population density scatterplot"), ("sans-serif", (40.0 * self.dpr) as u32, &WHITE))
                    .build_cartesian_2d((0.0..x_max).log_scale(), 0.0..y_max).unwrap()
                    .set_secondary_coord((0.0..x_max).log_scale(), 0..c_max);

                let bold_line = hex::decode("97948f").unwrap();
                let light_line = hex::decode("67635c").unwrap();

                label_size = max((label_size as f64 * (1.0 + (self.dpr * 0.1))) as u32, (8.0 * self.dpr) as u32);
                let desc_style = TextStyle::from(("sans-serif", (16.0 * self.dpr) as u32).into_font()).color(&WHITE);
                chart.configure_mesh()
                    .x_desc("population per square kilometer") 
                    .x_label_style(TextStyle::from(("sans-serif", label_size).into_font()).color(&WHITE))
                    // .x_label_formatter(&|v| {
                    //     if let CenterOf(s) = v {
                    //         return format!("{}", s);
                    //     } else {
                    //         return "".to_string();
                    //     }
                    // })
                    .y_desc("word count per 100,000")
                    .y_label_style(TextStyle::from(("sans-serif", label_size).into_font()).color(&WHITE))
                    .axis_desc_style(desc_style.clone())
                    .bold_line_style(RGBColor(bold_line[0], bold_line[1], bold_line[2]))
                    .light_line_style(RGBColor(light_line[0], light_line[1], light_line[2]))
                    .draw()
                    .unwrap();
                
                if show_counts {
                    chart
                        .configure_secondary_axes()
                        .y_desc("total word count")
                        .label_style(TextStyle::from(("sans-serif", label_size).into_font()).color(&WHITE))
                        .axis_desc_style(desc_style)
                        .y_label_formatter(&|v| { format!("{}", *v as u32) })
                        .draw()
                        .unwrap();
                }
                
                self.coord_mappings = vec![];
                for r in data.iter() {
                    let p = chart.backend_coord(&(r.pop_density, r.score));
                    self.coord_mappings.push(CoordMapping { x: p.0, y: p.1, id: r.id, name: r.name.clone() });
                }

                chart.draw_series(data.iter().map(|r| {
                    let rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                    Circle::new((r.pop_density, r.score), point_size, RGBColor(rgb[0], rgb[1], rgb[2]).filled())
                }))
                .unwrap();
                
                // if show_counts {
                //     chart.draw_secondary_series(data.iter().enumerate().map(|(i, r)| {
                //         let rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                //         Circle::new((r.pop_density, r.count), size, RGBColor(rgb[0], rgb[1], rgb[2]).filled())
                //     }))
                //     .unwrap();
                // }
            },
            PopulationPlotMsg::Clicked(e) => {
                let cm = self.mouse_mapping(e);
                
                if cm.id > 0 {
                    let heading = ctx.props().data.iter().filter(|r| r.id == cm.id).next().unwrap().name.clone();
                    ctx.props().get_speeches.emit(OverlaySelection {breakdown_type: BreakdownType::Speaker, id: cm.id, heading});
                }
            },
            PopulationPlotMsg::Hover(e) => {
                if !ctx.props().loading {
                    let cm = self.mouse_mapping(e);
                    
                    if cm.id != self.hover_id {
                        self.hover_id = cm.id;
                        let context = canvas_context(&inter_canvas);
                        context.clear_rect(0.0, 0.0, canvas_width.into(), canvas_height.into());
                        
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
        false
    }
    
    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        ctx.link().send_message(PopulationPlotMsg::Redraw);
        true
    }
}
