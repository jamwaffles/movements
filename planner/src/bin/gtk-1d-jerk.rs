use gtk::cairo::{Context, FontSlant, FontWeight};
use gtk::DrawingArea;
use gtk::MessageDialog;
use gtk::{prelude::*, Scale};
use gtk::{Adjustment, TextBuffer};
use gtk::{ApplicationWindow, TextView};
use gtk::{Builder, TextTagTable};
use planner::one_d::Vertex;
use planner::one_d_jerk::*;
use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::Arc;

struct State {
    start: Vertex,
    end: Vertex,
    limits: Limits,
    vertical_scale: f64,
    cursor: Option<(f64, f64)>,
}

impl State {
    fn make_segment(&self) -> Segment {
        Segment::new(self.start, self.end, &self.limits)
    }

    fn format_debug(&self, canvas: &DrawingArea) -> String {
        let dimensions = canvas.allocation();

        let width = f64::from(dimensions.width);
        let height = f64::from(dimensions.height);

        if let Some((cursor_x, cursor_y)) = self.cursor {
            let x_pos_norm = cursor_x / width;

            let segment = self.make_segment();

            let t = segment.duration() * x_pos_norm as f32;

            format!(
                "t   {:+02.3}\npos {:+02.3}\nvel {:+02.3}\nacc {:+02.3}",
                t,
                segment.position(t),
                segment.velocity(t),
                segment.acceleration(t)
            )
        } else {
            String::new()
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            start: Vertex {
                position: 0.0,
                velocity: 0.0,
            },
            end: Vertex {
                position: 1.0,
                velocity: 0.0,
            },
            limits: Limits {
                acceleration: 5.0,
                velocity: 1.0,
                jerk: 10.0,
            },
            vertical_scale: 40.0,
            cursor: None,
        }
    }
}

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("./gtk-1d-jerk.glade");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.object("window1").expect("Couldn't get window1");
    window.set_application(Some(application));

    let drawing_area: DrawingArea = builder.object("canvas1").expect("Couldn't find canvas");

    let mut state = Rc::new(RefCell::new(State::default()));

    let buf = TextBuffer::new::<TextTagTable>(None);
    let debug_output: TextView = builder.object("debug_output").unwrap();
    debug_output.set_buffer(Some(&buf));

    drawing_area.connect_motion_notify_event(
        glib::clone!(@weak state => @default-panic, move |canvas, event| {
            let pos = event.position();

            let mut state = state.borrow_mut();

            state.cursor = Some(pos);

            debug_output.buffer().unwrap().set_text(&state.format_debug(&canvas));

            canvas.queue_draw();

            Inhibit(false)
        }),
    );

    drawing_area.connect_leave_notify_event(
        glib::clone!(@weak state => @default-panic, move |canvas, event| {
            state.borrow_mut().cursor = None;
            canvas.queue_draw();
            Inhibit(false)
        }),
    );

    {
        let velocity_limit: Scale = builder.object("limits_velocity").expect("Velocity limit");
        let velocity_adjustment = Adjustment::new(
            state.borrow().limits.velocity.into(),
            0.01,
            5.0,
            0.01,
            0.05,
            0.0,
        );
        velocity_limit.set_adjustment(&velocity_adjustment);
        velocity_limit.connect_value_changed(
            glib::clone!(@weak state, @weak drawing_area => move |scale| {
                state.borrow_mut().limits.velocity = scale.value() as f32;
                drawing_area.queue_draw();
            }),
        );
    }

    {
        let acceleration_limit: Scale = builder
            .object("limits_acceleration")
            .expect("acceleration limit");
        let acceleration_adjustment = Adjustment::new(
            state.borrow().limits.acceleration.into(),
            0.01,
            5.0,
            0.01,
            0.05,
            0.0,
        );
        acceleration_limit.set_adjustment(&acceleration_adjustment);
        acceleration_limit.connect_value_changed(
            glib::clone!(@weak state, @weak drawing_area => move |scale| {
                state.borrow_mut().limits.acceleration = scale.value() as f32;
                drawing_area.queue_draw();
            }),
        );
    }

    {
        let jerk_limit: Scale = builder.object("limits_jerk").expect("jerk limit");
        let jerk_adjustment = Adjustment::new(
            state.borrow().limits.jerk.into(),
            0.01,
            50.0,
            0.01,
            0.05,
            0.0,
        );
        jerk_limit.set_adjustment(&jerk_adjustment);
        jerk_limit.connect_value_changed(
            glib::clone!(@weak state, @weak drawing_area => move |scale| {
                state.borrow_mut().limits.jerk = scale.value() as f32;
                drawing_area.queue_draw();
            }),
        );
    }

    {
        let start_position: Scale = builder.object("start_position").unwrap();
        let start_position_adjustment = Adjustment::new(
            state.borrow().start.position.into(),
            0.01,
            5.0,
            0.01,
            0.05,
            0.0,
        );
        start_position.set_adjustment(&start_position_adjustment);
        start_position.connect_value_changed(
            glib::clone!(@weak state, @weak drawing_area => move |scale| {
                state.borrow_mut().start.position = scale.value() as f32;
                drawing_area.queue_draw();
            }),
        );
    }

    {
        let start_velocity: Scale = builder.object("start_velocity").unwrap();
        let start_velocity_adjustment = Adjustment::new(
            state.borrow().start.velocity.into(),
            0.01,
            5.0,
            0.01,
            0.05,
            0.0,
        );
        start_velocity.set_adjustment(&start_velocity_adjustment);
        start_velocity.connect_value_changed(
            glib::clone!(@weak state, @weak drawing_area => move |scale| {
                state.borrow_mut().start.velocity = scale.value() as f32;
                drawing_area.queue_draw();
            }),
        );
    }

    {
        let end_position: Scale = builder.object("end_position").unwrap();
        let end_position_adjustment = Adjustment::new(
            state.borrow().end.position.into(),
            0.01,
            5.0,
            0.01,
            0.05,
            0.0,
        );
        end_position.set_adjustment(&end_position_adjustment);
        end_position.connect_value_changed(
            glib::clone!(@weak state, @weak drawing_area => move |scale| {
                state.borrow_mut().end.position = scale.value() as f32;
                drawing_area.queue_draw();
            }),
        );
    }

    {
        let end_velocity: Scale = builder.object("end_velocity").unwrap();
        let end_velocity_adjustment = Adjustment::new(
            state.borrow().end.velocity.into(),
            0.01,
            5.0,
            0.01,
            0.05,
            0.0,
        );
        end_velocity.set_adjustment(&end_velocity_adjustment);
        end_velocity.connect_value_changed(
            glib::clone!(@weak state, @weak drawing_area => move |scale| {
                state.borrow_mut().end.velocity = scale.value() as f32;
                drawing_area.queue_draw();
            }),
        );
    }

    {
        let vertical_scale: Scale = builder.object("vertical_scale").expect("vertical_scale");
        let vertical_scale_adjustment =
            Adjustment::new(state.borrow().vertical_scale, 0.1, 200.0, 0.1, 0.5, 0.0);
        vertical_scale.set_adjustment(&vertical_scale_adjustment);
        vertical_scale.connect_value_changed(
            glib::clone!(@weak state, @weak drawing_area => move |scale| {
                state.borrow_mut().vertical_scale = scale.value();
                drawing_area.queue_draw();
            }),
        );
    }

    drawing_area.connect_draw(
        glib::clone!(@strong state => @default-panic, move |drawing_area, cr| {
            let dimensions = drawing_area.allocation();

            let width = f64::from(dimensions.width);
            let height = f64::from(dimensions.height);
            // cr.scale(dimensions.width.into(), dimensions.height.into());

            cr.set_source_rgb(255.0, 255.0, 255.0);
            cr.paint().expect("Invalid cairo surface state");

            cr.set_line_width(1.0);

            let total = width as u32;

            let state = state.borrow();

            let segment = state.make_segment();

            let mid_y = height / 2.0;

            let y_scale = state.vertical_scale;

            // Vertical cursor line
            if let Some((cursor_x, _cursor_y)) = state.cursor {
                // Debug output
                {
                    let font_size = 14.0;
                    let x_base = 5.0;
                    let y_base = 5.0 + font_size;

                    let t = segment.duration() * cursor_x as f32 / width as f32;

                    cr.set_source_rgb(0.0, 0.0, 0.0);
                    cr.select_font_face("monospace", FontSlant::Normal, FontWeight::Normal);
                    cr.set_font_size(14.0);
                    cr.move_to(x_base, y_base);
                    cr.show_text(&format!("Duration     {:+04.2}", segment.duration())).unwrap();
                    cr.move_to(x_base, y_base + font_size);
                    cr.set_source_rgb(0.0, 0.0, 0.0);
                    cr.show_text(&format!("Position     {:+04.2}", segment.position(t))).unwrap();

                    cr.move_to(x_base, y_base + font_size * 2.0);
                    cr.set_source_rgb(1.0, 0.0, 0.0);
                    cr.show_text(&format!("Velocity     {:+04.2}", segment.velocity(t))).unwrap();

                    cr.move_to(x_base, y_base + font_size * 3.0);
                    cr.set_source_rgb(0.0, 0.0, 1.0);
                    cr.show_text(&format!("Acceleration {:+04.2}", segment.acceleration(t))).unwrap();
                }

                cr.set_source_rgb(0.5, 0.5, 0.5);

                cr.move_to(cursor_x, 0.0);
                cr.line_to(cursor_x, height);

                cr.stroke().expect("Invalid cairo surface state");
            }

            // Position
            {
                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.move_to(0.0, mid_y);
                for x in 0..total {
                    let t = segment.duration() * x as f32 / total as f32;
                    let y_pos: f64 = mid_y + f64::from(segment.position(t)) * y_scale;
                    cr.line_to(x as f64, height - y_pos);
                }
                cr.stroke().expect("Invalid cairo surface state");
            }

            // Velocity
            {
                cr.set_source_rgb(1.0, 0.0, 0.0);
                cr.move_to(0.0, mid_y);
                for x in 0..total {
                    let t = segment.duration() * x as f32 / total as f32;
                    let y_pos: f64 = mid_y + f64::from(segment.velocity(t)) * y_scale;
                    cr.line_to(x as f64, height - y_pos);
                }
                cr.stroke().expect("Invalid cairo surface state");
            }

            // Acceleration
            {
                cr.set_source_rgb(0.0, 0.0, 1.0);
                cr.move_to(0.0, mid_y);
                for x in 0..total {
                    let t = segment.duration() * x as f32 / total as f32;
                    let y_pos: f64 = mid_y + f64::from(segment.acceleration(t)) * y_scale;
                    cr.line_to(x as f64, height - y_pos);
                }
                cr.stroke().expect("Invalid cairo surface state");
            }

            Inhibit(false)


        }),
    );

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.cairotest"),
        Default::default(),
    );

    application.connect_activate(build_ui);

    application.run();
}
