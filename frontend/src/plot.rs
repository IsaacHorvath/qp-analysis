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
            <div style="margin: 5px; overflow-y: auto">
                <div style="border: 2px solid #fee17d; border-radius: 20px; padding: 5px; width: fit-content">
                    <canvas ref = {self.canvas.clone()}/>
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
                    BreakdownType::Speaker => min(max(900, width as u32), 1800),
                    BreakdownType::Party => min(max(600, width as u32), 900),
                    BreakdownType::Gender => min(max(300, width as u32), 700),
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
                let mut label_size = (width.sqrt() / 2.5) as u32;
                if *breakdown_type == BreakdownType::Speaker {
                    data = data.into_iter().filter(|r| r.count > 0).collect();
                    label_size = label_size - 4;
                }
                data.sort_by(|a, b| {b.score.total_cmp(&a.score)});
                if data.len() > 10 {
                     data = data[0..10].to_vec();
                }
                data = data.iter().map(|r| BreakdownResponse {
                    id: r.id,
                    name: format!("{} - {}", r.name, r.count),
                    colour: r.colour.clone(),
                    count: r.count,
                    score: r.score,
                }).collect();
                
                let x_axis = data.iter().map(|r| { r.name.clone() }).collect::<Vec<String>>();
                let y_max = data.iter().map(|r| { r.score }).max_by(|a, b| {a.total_cmp(b)}).unwrap(); 
                let c_max = data.iter().map(|r| { r.count }).max_by(|a, b| a.cmp(b)).unwrap(); 

                let mut chart= ChartBuilder::on(&drawing_area)
                    .set_left_and_bottom_label_area_size(50)
                    .caption(&format!("{} breakdown", *breakdown_type), ("sans-serif", 40, &WHITE))
                    .build_cartesian_2d(x_axis.into_segmented(), 0.0..y_max).unwrap()
                    .set_secondary_coord(0.0..data.len() as f32, 0.0..y_max as f32);

                let bold_line = hex::decode("97948f").unwrap();
                let light_line = hex::decode("67635c").unwrap();

                label_size = max(label_size, 8);
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

                chart.draw_secondary_series(data.iter().enumerate().map(|(i, r)| {
                    let rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                    let bar = Rectangle::new([(i as f32 + 0.15, 0.0), (i as f32 + 0.49, r.score)], RGBColor(rgb[0], rgb[1], rgb[2]).filled());
                    bar
                }))
                .unwrap();
                
                chart.draw_secondary_series(data.iter().enumerate().map(|(i, r)| {
                    let mut rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                    brighten(&mut rgb);
                    let c_height = r.count as f32 * (y_max / (c_max as f32));
                    let bar = Rectangle::new([(i as f32 + 0.51, 0.0), (i as f32 + 0.85, c_height)], RGBColor(rgb[0], rgb[1], rgb[2]).filled());
                    bar
                }))
                .unwrap();

                chart.draw_secondary_series(data.iter().enumerate().map(|(i, r)| {
                    let count = Text::new(r.name.clone(), ((i as f32) + 0.5, 10.0), ("sans-serif", 10));
                    //let mut bar = EmptyElement::at((x1, 0.0))
                    //    + Rectangle::new([(-10, 0), (10, 150)], RGBColor(rgb[0], rgb[1], rgb[2]).filled());
                    //bar.set_margin(0, 0, 5, 5);
                    count
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

fn brighten(rgb: &mut [u8]) {
    rgb[0] = min((rgb[0] as u32) - 40, 255) as u8;
    rgb[1] = min((rgb[1] as u32) - 40, 255) as u8;
    rgb[2] = min((rgb[2] as u32) - 40, 255) as u8;
}
