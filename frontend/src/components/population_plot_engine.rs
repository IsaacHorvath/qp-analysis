use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use yew::prelude::*;
use web_sys::HtmlCanvasElement;
use gloo::utils::window;
use common::models::PopulationResponse;
use crate::components::speech_overlay::OverlaySelection;
use std::cmp::{min, max};
use log::info;
use crate::components::plot::Plottable;
use crate::components::plot::canvas_context;
use common::models::BreakdownType;
use std::rc::Rc;

#[derive(Default, Clone)]
struct CoordMapping {
    x: i32,
    y: i32,
    id: i32,
    name: String,
}

// todo move to population component
struct PopDensity {
    pub id: i32,
    pub name: String,
    pub pop_density: f64,
    pub colour: String,
    pub count: i32,
    pub score: f32,
}

pub struct PopulationEngine {
    pub data: Rc<Vec<PopulationResponse>>,
    pub window_width: f64,
    pub dpr: f64,
    pub show_counts: bool,
    pub hover_id: i32,
    pub coord_mappings: Vec<CoordMapping>,
    pub get_speeches: Callback<OverlaySelection>,
}

impl PopulationEngine {
    fn point_size(&self) -> i32 {
        (5.0 * self.dpr) as i32
    }
    
    fn mouse_mapping(&self, e: MouseEvent) -> CoordMapping {
        let ps = self.point_size();
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

impl Plottable<PopulationResponse> for PopulationEngine {
    fn new(window_width: f64, show_counts: bool, get_speeches: Callback<OverlaySelection>) -> Self {
        PopulationEngine {
            data: Rc::from(vec![]),
            window_width,
            dpr: 1.0,
            show_counts,
            hover_id: 0,
            coord_mappings: vec![],
            get_speeches,
        }
    }
    fn load_data(&mut self, data: Rc<Vec<PopulationResponse>>) {
        self.data = data;
    }
    
    fn is_empty(&self) -> bool {
        self.data.as_ref().iter().filter(|d| d.count > 0).count() == 0
    }
    
    fn get_width(&self) -> u32 {
        min(max(900, (self.window_width * 0.97) as u32), 1800)
    }
    
    fn get_height(&self) -> u32 {
        400
    }
    
    fn get_heading(&self) -> String {
        "population density plot".to_string()
    }
    
    fn redraw(&mut self, canvas: HtmlCanvasElement, inter_canvas: HtmlCanvasElement) {
        self.dpr = window().device_pixel_ratio().max(1.0);
        let canvas_width = self.dpr * self.get_width() as f64;
        let canvas_height = self.dpr * self.get_height() as f64;
        canvas.set_height(canvas_height as u32);
        inter_canvas.set_height(canvas_height as u32);
        canvas.set_width(canvas_width as u32);
        inter_canvas.set_width(canvas_width as u32);

        let backend = CanvasBackend::with_canvas_object(canvas).unwrap();
        let drawing_area = backend.into_drawing_area();
        let mut label_size = (canvas_width.sqrt() / 2.5) as u32;
        
        let data = self.data.as_ref().iter().map(|r| { PopDensity {
            id: r.id,
            name: r.name.clone(),
            pop_density: r.population as f64 / r.area,
            colour: r.colour.clone(),
            score: r.score,
            count: r.count,
        }}).collect::<Vec<PopDensity>>();
        
        let x_max = data.iter().map(|r| { r.pop_density }).max_by(|a, b| {a.total_cmp(b)}).unwrap();
        let y_max = data.iter().map(|r| { r.score }).max_by(|a, b| {a.total_cmp(b)}).unwrap();
        let c_max = data.iter().map(|r| { r.count }).max_by(|a, b| a.cmp(b)).unwrap();

        let bold_line = hex::decode("97948f").unwrap();
        let light_line = hex::decode("67635c").unwrap();

        label_size = max((label_size as f64 * (1.0 + (self.dpr * 0.1))) as u32, (8.0 * self.dpr) as u32);
        let desc_style = TextStyle::from(("sans-serif", (16.0 * self.dpr) as u32).into_font()).color(&WHITE);
        self.coord_mappings = vec![];
        
        if !self.show_counts {
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
                Circle::new((r.pop_density, r.score), self.point_size(), RGBColor(rgb[0], rgb[1], rgb[2]).filled())
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
                Circle::new((r.pop_density, r.count), self.point_size(), RGBColor(rgb[0], rgb[1], rgb[2]).filled())
            }))
            .unwrap();
            
            for r in data.iter() {
                let p = chart.backend_coord(&(r.pop_density, r.count));
                self.coord_mappings.push(CoordMapping { x: p.0, y: p.1, id: r.id, name: r.name.clone() });
            };
        }
    }
    
    fn hover(&mut self, e: MouseEvent, inter_canvas: HtmlCanvasElement) {
        let cm = self.mouse_mapping(e);
        
        if cm.id != self.hover_id {
            self.hover_id = cm.id;
            let context = canvas_context(&inter_canvas);
            context.clear_rect(0.0, 0.0, inter_canvas.width() as f64, inter_canvas.width() as f64);
            
            let left = (cm.x - 10) as f64;
            let bottom = (cm.y - 10) as f64;
            if cm.id != 0 {
                context.begin_path();
                context.arc(cm.x as f64, cm.y as f64, self.point_size() as f64, 0.0, 2.0*std::f64::consts::PI).unwrap();
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
    
    fn clicked(&self, e: MouseEvent) {
        let cm = self.mouse_mapping(e);
        
        if cm.id > 0 {
            let heading = self.data.as_ref().iter().filter(|r| r.id == cm.id).next().unwrap().name.clone();
            self.get_speeches.emit(OverlaySelection {breakdown_type: BreakdownType::Speaker, id: cm.id, heading});
        }
    }
}
