use plotters::prelude::*;
use plotters::prelude::SegmentValue::CenterOf;
use plotters_canvas::CanvasBackend;
use yew::prelude::*;
use web_sys::HtmlCanvasElement;
use common::{BreakdownType, BreakdownResponse};
use std::cmp::{max, min};
use log::info;

pub enum PlotMsg {
    Redraw,
    Clicked(MouseEvent),
    Nothing,
}

#[derive(Properties, PartialEq)]
pub struct PlotProps {
    pub breakdown_type: BreakdownType,
    pub data: Vec<BreakdownResponse>,
    pub show_counts: bool,
    pub loading: bool,
    pub window_width: f64,
}

struct CoordMapping {
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
    id: i32
}

pub struct Plot {
    canvas: NodeRef,
    coord_mappings: Vec<CoordMapping>,
}

impl Component for Plot {

    type Message = PlotMsg;
    type Properties = PlotProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(PlotMsg::Redraw);
        Plot {
            canvas: NodeRef::default(),
            coord_mappings: vec![],
        }       
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_click = ctx.link().callback(|e: MouseEvent| PlotMsg::Clicked(e));
        
        html! (
            <div style="margin: 5px; overflow-y: auto">
                <div style="border: 2px solid #fee17d; border-radius: 20px; padding: 5px; width: fit-content">
                    <canvas onclick={on_click} ref = {self.canvas.clone()}/>
                </div>
            </div>
        )
    }
  
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlotMsg::Redraw => {
                let element : HtmlCanvasElement = self.canvas.cast().unwrap();
                let breakdown_type = &ctx.props().breakdown_type;
                
                let width = ctx.props().window_width - 40.0;
                // //let rect = element.get_bounding_client_rect();
                element.set_height(500);
                element.set_width(match *breakdown_type {
                    BreakdownType::Speaker => min(max(900, width as u32), 1800), //todo width dependent on num speakers
                    BreakdownType::Party => min(max(600, width as u32), 900),
                    BreakdownType::Gender => min(max(300, width as u32), 700),
                });
                if ctx.props().loading {
                    element.set_attribute("style", "opacity: 0.25").expect("couldn't set opacity");
                }
                else {
                    element.set_attribute("style", "opacity: 1").expect("couldn't set opacity");
                }

                let backend = CanvasBackend::with_canvas_object(element).unwrap();
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

                // use the secondary series to allow for fine-tuned x values despite segmentation
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
                for cm in &self.coord_mappings {
                    let x = e.offset_x();
                    let y = e.offset_y();
                    if x > cm.left && x < cm.right && y > cm.top && y < cm.bottom {
                        info!("{} {} {}", x, y, cm.id);
                    }
                }
                false
            },
            _ => true,
        }
    }
    
    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        ctx.link().send_message(PlotMsg::Redraw);
        true
    }
}
