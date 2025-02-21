use plotters::prelude::*;
use plotters::prelude::SegmentValue::CenterOf;
use plotters_canvas::CanvasBackend;
use yew::prelude::*;
use yew_hooks::prelude::*;
use web_sys::{HtmlCanvasElement, Window};
use common::{BreakdownType, BreakdownResponse};
use std::cmp::{max, min};
use log::info;


pub enum PlotMsg {
    Redraw,
    Nothing,
}

#[derive(Properties, PartialEq)]
pub struct PlotProps {
    pub breakdown_type: BreakdownType,
    pub data: Vec<BreakdownResponse>,
    pub loading: bool,
    pub window_width: f64,
}

pub struct Plot {
    canvas: NodeRef,
}

impl Component for Plot {

    type Message = PlotMsg;
    type Properties = PlotProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(PlotMsg::Redraw);
        Plot {
            canvas : NodeRef::default(),
        }       
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! (
            <div style="border: 2px solid #fee17d; border-radius: 20px; margin: 5px; padding: 5px">
                <canvas ref = {self.canvas.clone()}/>
            </div>
        )
    }
  
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlotMsg::Redraw => {
                let element : HtmlCanvasElement = self.canvas.cast().unwrap();
                let breakdown_type = &ctx.props().breakdown_type;
                
                let width = ctx.props().window_width;
                // //let rect = element.get_bounding_client_rect();
                element.set_height(500);
                element.set_width(match *breakdown_type {
                    BreakdownType::Speaker => min(max(800, width as u32), 1800),
                    BreakdownType::Party => min(max(300, width as u32), 900),
                    BreakdownType::Gender => min(max(200, width as u32), 700),
                });
                //info!("{} {}", height, width);
                // element.set_height(500);
                // element.set_width(match *breakdown_type {
                //      BreakdownType::Speaker => 1800,
                //      _ => 900,
                //  }); 
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
                if *breakdown_type == BreakdownType::Speaker {
                    data = data.into_iter().filter(|r| r.count > 0).collect();
                }
                data.sort_by(|a, b| {b.score.total_cmp(&a.score)});
                if data.len() > 10 {
                     data = data[0..10].to_vec();
                }
                data = data.iter().map(|r| BreakdownResponse {
                    name: format!("{} - {}", r.name, r.count),
                    colour: r.colour.clone(),
                    count: r.count,
                    score: r.score,
                }).collect();
                
                let x_axis = data.iter().map(|r| { r.name.clone() }).collect::<Vec<String>>();
                let y_max = data.iter().map(|r| { r.score }).max_by(|a, b| {a.total_cmp(b)}).unwrap(); 

                let mut chart= ChartBuilder::on(&drawing_area)
                    .set_left_and_bottom_label_area_size(50)
                    .caption(&format!("{} breakdown", *breakdown_type), ("sans-serif", 40, &WHITE))
                    .build_cartesian_2d(x_axis.into_segmented(), 0f32..y_max)
                    .unwrap();

                let bold_line = hex::decode("97948f").expect("decoding colour failed");
                let light_line = hex::decode("67635c").expect("decoding colour failed");

                let label_size = (width.sqrt() / 2.5) as u32;
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
                    .y_desc("words per 100,000")
                    .y_label_style(&WHITE)
                    .bold_line_style(RGBColor(bold_line[0], bold_line[1], bold_line[2]))
                    .light_line_style(RGBColor(light_line[0], light_line[1], light_line[2]))
                    .draw()
                    .unwrap();

                chart.draw_series(data.iter().map(|r| {
                    let x0 = SegmentValue::Exact(&r.name);
                    let x1 = SegmentValue::CenterOf(&r.name);
                    let rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                    let mut bar = Rectangle::new([(x0, 0.0), (x1, r.score)], RGBColor(rgb[0], rgb[1], rgb[2]).filled());
                    bar.set_margin(0, 0, 5, 5);
                    bar
                }))
                .unwrap();
                
                //let test = String::from("NDP");
                //drawing_area.draw(&Text::new("Test".to_string(), (1, 1), ("sans-serif", 10))).unwrap();
                
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
