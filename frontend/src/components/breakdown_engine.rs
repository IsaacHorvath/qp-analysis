use plotters::prelude::*;
use plotters::prelude::SegmentValue::CenterOf;
use plotters_canvas::CanvasBackend;
use yew::prelude::*;
use web_sys::HtmlCanvasElement;
use gloo::utils::window;
use common::models::BreakdownResponse;
use crate::components::speech_overlay::OverlaySelection;
use std::cmp::{min, max};
use crate::components::plot::{Plottable, canvas_context, PlotError};
use common::models::BreakdownType;
use std::rc::Rc;

#[derive(Default, Clone)]
struct CoordMapping {
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
    id: i32,
}

pub struct BreakdownEngine {
    data: Rc<Vec<BreakdownResponse>>,
    breakdown_type: BreakdownType,
    window_width: f64,
    dpr: f64,
    show_counts: bool,
    hover_id: i32,
    coord_mappings: Vec<CoordMapping>,
    get_speeches: Option<Callback<OverlaySelection>>,
}

impl BreakdownEngine {
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

impl Plottable<BreakdownResponse> for BreakdownEngine {
    fn new(breakdown_type: BreakdownType) -> Self {
        BreakdownEngine {
            breakdown_type: breakdown_type.clone(),
            data: Rc::from(vec![]),
            window_width: 0.0,
            dpr: 1.0,
            show_counts: false,
            hover_id: 0,
            coord_mappings: vec![],
            get_speeches: None,
        }
    }
    
    fn set_props(&mut self, window_width: f64, show_counts: bool, get_speeches: Callback<OverlaySelection>) {
        self.window_width = window_width;
        self.show_counts = show_counts;
        self.get_speeches = Some(get_speeches);
    }
    
    fn load_data(&mut self, data: Rc<Vec<BreakdownResponse>>) {
        self.data = data;
    }
    
    fn is_empty(&self) -> bool {
        self.data.as_ref().len() == 0
    }
    
    fn get_width(&self) -> u32 {
        let segs = self.data.as_ref().len() as u32;
        let ww = (self.window_width * 0.97) as u32;
        match self.breakdown_type {
            BreakdownType::Speaker => min(max(segs*90, ww), segs*160),
            BreakdownType::Party => min(max(segs*80, ww), segs*160),
            BreakdownType::Gender => min(max(segs*80, ww), segs*160),
            BreakdownType::Province => min(max(segs*90, ww), segs*160),
        }
    }
    
    fn get_height(&self) -> u32 {
        500
    }
    
    fn get_heading(&self) -> String {
        format!("{} breakdown", self.breakdown_type)
    }
    
    fn redraw(&mut self, canvas: HtmlCanvasElement, inter_canvas: HtmlCanvasElement) -> Result<(), PlotError> {
        self.dpr = window().device_pixel_ratio().max(1.0);
        let canvas_width = (self.dpr * self.get_width() as f64) as u32;
        let canvas_height = (self.dpr * self.get_height() as f64) as u32;
        canvas.set_height(canvas_height);
        inter_canvas.set_height(canvas_height);
        canvas.set_width(canvas_width);
        inter_canvas.set_width(canvas_width);
        
        // todo get rid of this clone
        let mut data = self.data.as_ref().clone();
        data.sort_by(|a, b| {b.score.total_cmp(&a.score)});

        let backend = CanvasBackend::with_canvas_object(canvas).ok_or(PlotError)?;
        let drawing_area = backend.into_drawing_area();
        let mut label_size = (self.window_width.sqrt() / 2.5 * self.dpr) as u32;
        
        if self.breakdown_type == BreakdownType::Speaker || self.breakdown_type == BreakdownType::Province {
            label_size = label_size - 4;
        }
        
        let x_axis = data.iter().map(|r| { r.name.clone() }).collect::<Vec<String>>();
        let y_max = data.iter().map(|r| { r.score }).max_by(|a, b| {a.total_cmp(b)}).ok_or(PlotError)?; 
        let c_max = data.iter().map(|r| { r.count }).max_by(|a, b| a.cmp(b)).ok_or(PlotError)? as f32;

        let mut chart= ChartBuilder::on(&drawing_area)
            .x_label_area_size((40.0 * self.dpr) as u32)
            .y_label_area_size((70.0 * self.dpr) as u32)
            .right_y_label_area_size(if self.show_counts {(60.0 * self.dpr) as u32} else {0})
            .caption(" ", ("sans-serif", (30.0 * self.dpr) as u32, &WHITE))
            .build_cartesian_2d(x_axis.into_segmented(), 0.0..y_max)?
            .set_secondary_coord(0.0..data.len() as f32, 0.0..c_max);

        let bold_line = hex::decode("97948f")?;
        let light_line = hex::decode("67635c")?;

        label_size = max((label_size as f64 * (1.0 + (self.dpr * 0.1))) as u32, (8.0 * self.dpr) as u32);
        let desc_style = TextStyle::from(("sans-serif", (16.0 * self.dpr) as u32).into_font()).color(&WHITE);
        chart.configure_mesh()
            .disable_x_mesh()
            .x_desc(format!("{}", self.breakdown_type)) 
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
            .draw()?;
        
        if self.show_counts {
            chart
                .configure_secondary_axes()
                .y_desc("total word count")
                .label_style(TextStyle::from(("sans-serif", label_size).into_font()).color(&WHITE))
                .axis_desc_style(desc_style)
                .y_label_formatter(&|v| { format!("{}", *v as u32) })
                .draw()?;
        }
        
        self.coord_mappings = vec![];
        for (i, r) in data.iter().enumerate() {
            let left = i as f32 + if self.show_counts {0.15} else {0.20};
            let right = i as f32 + if self.show_counts {0.85} else {0.80};
            let mut top = r.score * (c_max / y_max);
            if self.show_counts { top = f32::max(r.count as f32, top) }
            let tl = chart.borrow_secondary().backend_coord(&(left, top));
            let br = chart.borrow_secondary().backend_coord(&(right, 0.0));
            self.coord_mappings.push(CoordMapping { left: tl.0, top: tl.1, right: br.0, bottom: br.1, id: r.id });
        }

        // use the secondary series to allow for fine-tuned x values instead of segments
        chart.draw_secondary_series(data.iter().enumerate().map(|(i, r)| {
            let s_height = r.score * (c_max / y_max);
            let left = i as f32 + if self.show_counts {0.15} else {0.20};
            let right = i as f32 + if self.show_counts {0.49} else {0.80};
            
            let rgb = hex::decode(r.colour.clone())?;
            Ok(Rectangle::new([(left, 0.0), (right, s_height)], RGBColor(rgb[0], rgb[1], rgb[2]).filled()))
        }).collect::<Result<Vec<Rectangle<(f32, f32)>>, PlotError>>()?)?;
        
        if self.show_counts {
            chart.draw_secondary_series(data.iter().enumerate().map(|(i, r)| {
                let rgb = hex::decode(r.colour.clone())?;
                Ok(Rectangle::new([(i as f32 + 0.51, 0.0), (i as f32 + 0.85, r.count as f32)], RGBColor(rgb[0], rgb[1], rgb[2]).filled()))
            }).collect::<Result<Vec<Rectangle<(f32, f32)>>, PlotError>>()?)?;
        }
        Ok(())
    }
    
    fn hover(&mut self, e: MouseEvent, inter_canvas: HtmlCanvasElement) -> Result<(), PlotError> {
        let cm = self.mouse_mapping(e);
        
        if cm.id != self.hover_id {
            self.hover_id = cm.id;
            let context = canvas_context(&inter_canvas).ok_or(PlotError)?;
            context.clear_rect(0.0, 0.0, inter_canvas.width() as f64, inter_canvas.height() as f64);
            
            let top = min(cm.top, cm.bottom - 20);
            if cm.id != 0  {
                context.set_line_width(3.0);
                context.set_stroke_style_str("#fee17d");
                context.stroke_rect(cm.left.into(), top.into(), (cm.right - cm.left).into(), (cm.bottom - top).into());
            }
        }
        Ok(())
    }
    
    fn clicked(&self, e: MouseEvent) -> Result<(), PlotError> {
        if let Some(get_speeches) = &self.get_speeches {
            let cm = self.mouse_mapping(e);
            if cm.id > 0 {
                let heading = self.data
                    .as_ref()
                    .iter()
                    .filter(|r| r.id == cm.id)
                    .next()
                    .ok_or(PlotError)?
                    .name
                    .clone();
                get_speeches.emit(OverlaySelection {breakdown_type: self.breakdown_type.clone(), id: cm.id, heading});
            }
        }
        Ok(())
    }
}
