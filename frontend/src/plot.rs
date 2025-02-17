use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use yew::prelude::*;
use web_sys::HtmlCanvasElement;
use common::{BreakdownType, BreakdownResponse};

pub enum PlotMsg {
    Redraw,
    Nothing,
}

#[derive(Properties, PartialEq)]
pub struct PlotProps {
    pub breakdown_type: BreakdownType,
    pub data: Vec<BreakdownResponse>,
    pub loading: bool,
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
            <div>
                <canvas ref = {self.canvas.clone()}/>
            </div>
        )
    }
  
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlotMsg::Redraw => {
                let element : HtmlCanvasElement = self.canvas.cast().unwrap();
                let breakdown_type = &ctx.props().breakdown_type;
                            
                //let rect = element.get_bounding_client_rect();
                element.set_height(500);
                element.set_width(match *breakdown_type {
                    BreakdownType::Speaker => 1400,
                    _ => 700,
                });
                if ctx.props().loading {
                    element.set_attribute("style", "opacity: 0.25").expect("couldn't set opacity");
                }
                else {
                    element.set_attribute("style", "opacity: 1").expect("couldn't set opacity");
                }

                let backend = CanvasBackend::with_canvas_object(element).unwrap();
                let root = backend.into_drawing_area();
                
                root.fill(&WHITE).unwrap();
                let mut data: Vec<BreakdownResponse> = ctx.props().data.clone();
                if *breakdown_type == BreakdownType::Speaker {
                    data = data.into_iter().filter(|r| r.count > 0).collect();
                }
                data.sort_by(|a, b| {b.score.total_cmp(&a.score)});
                if data.len() > 10 {
                     data = data[0..10].to_vec();
                }
                
                let x_axis = data.iter().map(|r| { r.name.clone() }).collect::<Vec<String>>();
                let y_max = data.iter().map(|r| { r.score }).max_by(|a, b| {a.total_cmp(b)}).unwrap(); 

                let mut chart= ChartBuilder::on(&root)
                    .set_label_area_size(LabelAreaPosition::Left, 40)
                    .set_label_area_size(LabelAreaPosition::Bottom, 40)
                    .caption(&format!("{} word usage", *breakdown_type), ("sans-serif", 40))
                    .build_cartesian_2d(x_axis.into_segmented(), 0f32..y_max)
                    .unwrap();

                chart.configure_mesh().draw().unwrap();

                chart.draw_series(data.iter().map(|r| {
                    let x0 = SegmentValue::Exact(&r.name);
                    let x1 = SegmentValue::CenterOf(&r.name);
                    let rgb = hex::decode(r.colour.clone()).expect("decoding colour failed");
                    let mut bar = Rectangle::new([(x0, 0.0), (x1, r.score)], RGBColor(rgb[0], rgb[1], rgb[2]).filled());
                    bar.set_margin(0, 0, 5, 5);
                    bar
                }))
                .unwrap();
                
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
