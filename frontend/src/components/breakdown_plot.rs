use plotters::prelude::*;
use plotters::prelude::SegmentValue::CenterOf;
use plotters_canvas::CanvasBackend;
use yew::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use gloo::utils::window;
use common::models::{BreakdownType, BreakdownResponse};
use crate::components::speech_overlay::OverlaySelection;
use std::cmp::{max, min};
use wasm_bindgen::JsCast;
use log::info;

pub enum BreakdownPlotMsg {
    Redraw,
    Clicked(MouseEvent),
    Hover(MouseEvent),
}

#[derive(Properties, PartialEq)]
pub struct BreakdownPlotProps {
    pub breakdown_type: BreakdownType,
    pub data: Vec<BreakdownResponse>,
    pub show_counts: bool,
    pub loading: bool,
    pub window_width: f64,
    pub get_speeches: Callback<OverlaySelection>,
}

#[derive(Default, Clone)]
struct CoordMapping {
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
    id: i32,
}

pub struct BreakdownPlot {
    canvas: NodeRef,
    inter_canvas: NodeRef,
    dpr: f64,
    coord_mappings: Vec<CoordMapping>,
    hover_id: i32,
}

fn canvas_context(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap()
}

impl BreakdownPlot {
    fn mouse_mapping(&self, e: MouseEvent) -> CoordMapping {
        let x = (e.offset_x() as f64 * self.dpr) as i32;
        let y = (e.offset_y() as f64 * self.dpr) as i32;
        for m in &self.coord_mappings {
            if x > m.left && x < m.right && y > min(m.top, m.bottom - 20) && y < m.bottom + 30 {
                return m.clone()
            }
        }
        CoordMapping::default()
    }
}

impl Component for BreakdownPlot {

    type Message = BreakdownPlotMsg;
    type Properties = BreakdownPlotProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(BreakdownPlotMsg::Redraw);
        BreakdownPlot {
            canvas: NodeRef::default(),
            inter_canvas: NodeRef::default(),
            dpr: 1.0,
            coord_mappings: vec![],
            hover_id: 0,
        }       
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|e: MouseEvent| BreakdownPlotMsg::Clicked(e));
        let onmousemove = ctx.link().callback(|e: MouseEvent| BreakdownPlotMsg::Hover(e));
        
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
        let breakdown_type = &ctx.props().breakdown_type.clone();
        let mut data: Vec<BreakdownResponse> = ctx.props().data.clone();
                
        // todo do these transformations in database
        data.sort_by(|a, b| {b.score.total_cmp(&a.score)});
        if data.len() > 10 {
                data = data[0..10].to_vec();
        }
        
        let window_width = ctx.props().window_width - 40.0;
        let segs = data.len() as u32;
        let width = match *breakdown_type {
            BreakdownType::Speaker => min(max(segs*90, window_width as u32), segs*180),
            BreakdownType::Party => min(max(segs*80, window_width as u32), segs*160),
            BreakdownType::Gender => min(max(segs*80, window_width as u32), segs*160),
            BreakdownType::Province => min(max(segs*90, window_width as u32), segs*180),
        };
        let height: u32 = 500;
        
        self.dpr = window().device_pixel_ratio();
        let mut canvas_width = width;
        let mut canvas_height = height;
        if self.dpr >= 1.0 {
            canvas_width = (self.dpr * canvas_width as f64) as u32;
            canvas_height = (self.dpr * canvas_height as f64) as u32;
        }
        
        match msg {
            BreakdownPlotMsg::Redraw => {
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
                
                if *breakdown_type == BreakdownType::Speaker {
                    label_size = label_size - 4;
                }
                
                let show_counts = ctx.props().show_counts;
                let x_axis = data.iter().map(|r| { r.name.clone() }).collect::<Vec<String>>();
                let y_max = data.iter().map(|r| { r.score }).max_by(|a, b| {a.total_cmp(b)}).unwrap(); 
                let c_max = data.iter().map(|r| { r.count }).max_by(|a, b| a.cmp(b)).unwrap() as f32; 

                let mut chart= ChartBuilder::on(&drawing_area)
                    .x_label_area_size((40.0 * self.dpr) as u32)
                    .y_label_area_size((60.0 * self.dpr) as u32)
                    .right_y_label_area_size(if show_counts {(60.0 * self.dpr) as u32} else {0})
                    .caption(&format!("{} breakdown", *breakdown_type), ("sans-serif", (40.0 * self.dpr) as u32, &WHITE))
                    .build_cartesian_2d(x_axis.into_segmented(), 0.0..y_max).unwrap()
                    .set_secondary_coord(0.0..data.len() as f32, 0.0..c_max);

                let bold_line = hex::decode("97948f").unwrap();
                let light_line = hex::decode("67635c").unwrap();

                label_size = max((label_size as f64 * (1.0 + (self.dpr * 0.1))) as u32, (8.0 * self.dpr) as u32);
                let desc_style = TextStyle::from(("sans-serif", (16.0 * self.dpr) as u32).into_font()).color(&WHITE);
                chart.configure_mesh()
                    .disable_x_mesh()
                    .x_desc(format!("{}", *breakdown_type)) 
                    .x_label_style(TextStyle::from(("sans-serif", label_size).into_font()).color(&WHITE))
                    .x_label_formatter(&|v| {
                        if let CenterOf(s) = v {
                            return format!("{}", s);
                        } else {
                            return "".to_string();
                        }
                    })
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
                for (i, r) in data.iter().enumerate() {
                    let left = i as f32 + if show_counts {0.15} else {0.20};
                    let right = i as f32 + if show_counts {0.85} else {0.80};
                    let mut top = r.score * (c_max / y_max);
                    if show_counts { top = f32::max(r.count as f32, top) }
                    let tl = chart.borrow_secondary().backend_coord(&(left, top));
                    let br = chart.borrow_secondary().backend_coord(&(right, 0.0));
                    self.coord_mappings.push(CoordMapping { left: tl.0, top: tl.1, right: br.0, bottom: br.1, id: r.id });
                }

                // use the secondary series to allow for fine-tuned x values instead of segments
                chart.draw_secondary_series(data.iter().enumerate().map(|(i, r)| {
                    let s_height = r.score * (c_max / y_max);
                    let left = i as f32 + if show_counts {0.15} else {0.20};
                    let right = i as f32 + if show_counts {0.49} else {0.80};
                    
                    let rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                    Rectangle::new([(left, 0.0), (right, s_height)], RGBColor(rgb[0], rgb[1], rgb[2]).filled())
                }))
                .unwrap();
                
                if show_counts {
                    chart.draw_secondary_series(data.iter().enumerate().map(|(i, r)| {
                        let rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                        Rectangle::new([(i as f32 + 0.51, 0.0), (i as f32 + 0.85, r.count as f32)], RGBColor(rgb[0], rgb[1], rgb[2]).filled())
                    }))
                    .unwrap();
                }
            },
            BreakdownPlotMsg::Clicked(e) => {
                let cm = self.mouse_mapping(e);
                
                if cm.id > 0 {
                    let breakdown_type = breakdown_type.clone();
                    let heading = ctx.props().data.iter().filter(|r| r.id == cm.id).next().unwrap().name.clone();
                    ctx.props().get_speeches.emit(OverlaySelection {breakdown_type, id: cm.id, heading});
                }
            },
            BreakdownPlotMsg::Hover(e) => {
                if !ctx.props().loading {
                    let cm = self.mouse_mapping(e);
                    
                    if cm.id != self.hover_id {
                        self.hover_id = cm.id;
                        let context = canvas_context(&inter_canvas);
                        
                        let top = min(cm.top, cm.bottom - 20);
                        if cm.id != 0  {
                            info!("hovering over {}", cm.id);
                            context.set_line_width(3.0);
                            context.set_stroke_style_str("#fee17d");
                            context.stroke_rect(cm.left.into(), top.into(), (cm.right - cm.left).into(), (cm.bottom - top).into());
                        }
                        else {
                            context.clear_rect(0.0, 0.0, canvas_width.into(), canvas_height.into());
                        }
                    }
                }
            },
        };
        false
    }
    
    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        ctx.link().send_message(BreakdownPlotMsg::Redraw);
        true
    }
}
