use plotters::prelude::*;
use plotters::prelude::SegmentValue::CenterOf;
use plotters_canvas::CanvasBackend;
use yew::prelude::*;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use common::{BreakdownType, BreakdownResponse};
use std::cmp::{max, min};
use wasm_bindgen::JsCast;
use log::info;

pub enum PlotMsg {
    Redraw,
    Clicked(MouseEvent),
    Hover(MouseEvent),
}

#[derive(Properties, PartialEq)]
pub struct PlotProps {
    pub breakdown_type: BreakdownType,
    pub data: Vec<BreakdownResponse>,
    pub show_counts: bool,
    pub loading: bool,
    pub window_width: f64,
    pub get_speeches: Callback<i32>,
}

#[derive(Default, Clone)]
struct CoordMapping {
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
    id: i32,
}

pub struct Plot {
    canvas: NodeRef,
    inter_canvas: NodeRef,
    coord_mappings: Vec<CoordMapping>,
}

impl Component for Plot {

    type Message = PlotMsg;
    type Properties = PlotProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(PlotMsg::Redraw);
        Plot {
            canvas: NodeRef::default(),
            inter_canvas: NodeRef::default(),
            coord_mappings: vec![],
        }       
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|e: MouseEvent| PlotMsg::Clicked(e));
        let onmousemove = ctx.link().callback(|e: MouseEvent| PlotMsg::Hover(e));
        
        html! (
            <div style="margin: 5px; overflow: auto">
                <div style="border: 2px solid #fee17d; border-radius: 20px; padding: 5px; width: fit-content">
                    <canvas style="position: absolute; z-index: 20" {onclick} {onmousemove} ref = {self.inter_canvas.clone()}/>
                    <canvas ref = {self.canvas.clone()}/>
                </div>
            </div>
        )
    }
  
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let breakdown_type = &ctx.props().breakdown_type;
        match msg {
            PlotMsg::Redraw => {
                let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
                let inter_canvas: HtmlCanvasElement = self.inter_canvas.cast().unwrap();
                
                let width = ctx.props().window_width - 40.0;
                // //let rect = canvas.get_bounding_client_rect();
                canvas.set_height(500);
                inter_canvas.set_height(500);
                let canvas_width = match *breakdown_type {
                    BreakdownType::Speaker => min(max(900, width as u32), 1800), //todo width dependent on num speakers
                    BreakdownType::Party => min(max(600, width as u32), 900),
                    BreakdownType::Gender => min(max(300, width as u32), 700),
                };
                canvas.set_width(canvas_width);
                inter_canvas.set_width(canvas_width);
                
                if ctx.props().loading {
                    canvas.set_attribute("style", "opacity: 0.25").expect("couldn't set opacity");
                }
                else {
                    canvas.set_attribute("style", "opacity: 1").expect("couldn't set opacity");
                }

                let backend = CanvasBackend::with_canvas_object(canvas).unwrap();
                let drawing_area = backend.into_drawing_area();
                //drawing_area.fill(&WHITE).unwrap();
                
                let mut data: Vec<BreakdownResponse> = ctx.props().data.clone();
                let mut label_size = (width.sqrt() / 2.5) as u32;
                if *breakdown_type == BreakdownType::Speaker {
                    data = data.into_iter().filter(|r| r.count > 0).collect();
                    label_size = label_size - 4;
                }
                data.sort_by(|a, b| {b.score.total_cmp(&a.score)});
                if data.len() > 10 {
                     data = data[0..10].to_vec();
                }
                
                let show_counts = ctx.props().show_counts;
                let x_axis = data.iter().map(|r| { r.name.clone() }).collect::<Vec<String>>();
                let y_max = data.iter().map(|r| { r.score }).max_by(|a, b| {a.total_cmp(b)}).unwrap(); 
                let c_max = data.iter().map(|r| { r.count }).max_by(|a, b| a.cmp(b)).unwrap() as f32; 

                let mut chart= ChartBuilder::on(&drawing_area)
                    .x_label_area_size(40)
                    .y_label_area_size(60) //todo use log or text length to find out how much space we need for these
                    .right_y_label_area_size(if show_counts {60} else {0})
                    .caption(&format!("{} breakdown", *breakdown_type), ("sans-serif", 40, &WHITE))
                    .build_cartesian_2d(x_axis.into_segmented(), 0.0..y_max).unwrap()
                    .set_secondary_coord(0.0..data.len() as f32, 0.0..c_max);

                let bold_line = hex::decode("97948f").unwrap();
                let light_line = hex::decode("67635c").unwrap();

                label_size = max(label_size, 8);
                let desc_style = TextStyle::from(("sans-serif", 16).into_font()).color(&WHITE);
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
                    .y_label_style(&WHITE)
                    .axis_desc_style(desc_style.clone())
                    .bold_line_style(RGBColor(bold_line[0], bold_line[1], bold_line[2]))
                    .light_line_style(RGBColor(light_line[0], light_line[1], light_line[2]))
                    .draw()
                    .unwrap();
                
                if show_counts {
                    chart
                        .configure_secondary_axes()
                        .y_desc("total word count")
                        .label_style(&WHITE)
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
                
                false
            },
            PlotMsg::Clicked(e) => {
                if *breakdown_type == BreakdownType::Speaker {
                    let mut id: i32 = 0;
                    for cm in &self.coord_mappings {
                        let x = e.offset_x();
                        let y = e.offset_y();
                        if x > cm.left && x < cm.right && y > cm.top && y < cm.bottom {
                            id = cm.id;
                            break
                        }
                    }
                    if id > 0 {
                        ctx.props().get_speeches.emit(id);
                    }
                }
                
                false
            },
            PlotMsg::Hover(e) => {
                if !ctx.props().loading {
                    if *breakdown_type == BreakdownType::Speaker {
                        let mut cm = CoordMapping::default();
                        for coord_mapping in &self.coord_mappings {
                            let x = e.offset_x();
                            let y = e.offset_y();
                            if x > coord_mapping.left &&
                                x < coord_mapping.right &&
                                y > coord_mapping.top &&
                                y < coord_mapping.bottom + 20
                            {
                                cm = coord_mapping.clone();
                                break
                            }
                        }
                        if cm.id != 0 {
                            info!("hovering over {}", cm.id);
                            let context: CanvasRenderingContext2d = self
                                .inter_canvas.cast::<HtmlCanvasElement>()
                                .unwrap()
                                .get_context("2d")
                                .unwrap()
                                .unwrap()
                                .dyn_into::<CanvasRenderingContext2d>()
                                .unwrap();

                            context.set_stroke_style_str("#fee17d");
                            context.stroke_rect(cm.left.into(), cm.top.into(), (cm.right - cm.left).into(), (cm.bottom + 20 - cm.top).into());
                        }
                    }
                }
                
                false
            },
        }
    }
    
    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        ctx.link().send_message(PlotMsg::Redraw);
        true
    }
}
