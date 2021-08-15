use gtk::cairo::{Context, FontSlant, FontWeight};
use gtk::DrawingArea;
use gtk::MessageDialog;
use gtk::{prelude::*, Scale};
use gtk::{Adjustment, TextBuffer};
use gtk::{ApplicationWindow, TextView};
use gtk::{Builder, TextTagTable};
use planner::one_d::*;
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
            },
            vertical_scale: 40.0,
            cursor: None,
        }
    }
}

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("./gtk-1d-trapezoidal.glade");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.object("window1").expect("Couldn't get window1");
    window.set_application(Some(application));

    let dialog: MessageDialog = builder
        .object("messagedialog1")
        .expect("Couldn't get messagedialog1");
    dialog.connect_delete_event(|dialog, _| {
        dialog.hide();
        gtk::Inhibit(true)
    });

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

    // builder.connect_signals(move |_, handler_name| {
    //     // This is the one-time callback to register signals.
    //     // Here we map each handler name to its handler.

    //     if handler_name == "button1_clicked" {
    //         // Return the signal handler.
    //         Box::new(
    //             glib::clone!(@weak dialog => @default-return None, move |_| {
    //                 dialog.show_all();
    //                 None
    //             }),
    //         )
    //     } else {
    //         panic!("Unknown handler name {}", handler_name)
    //     }
    // });

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
            if let Some((cursor_x, _cursor_y)) = state.cursor{
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
                    cr.line_to(x as f64, y_pos);
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
                    cr.line_to(x as f64, y_pos);
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
                    cr.line_to(x as f64, y_pos);
                }
                cr.stroke().expect("Invalid cairo surface state");
            }

            Inhibit(false)

            // // border
            // cr.set_source_rgb(0.3, 0.3, 0.3);
            // cr.rectangle(0.0, 0.0, 1.0, 1.0);
            // cr.stroke().expect("Invalid cairo surface state");

            // cr.set_line_width(0.03);

            // // draw circle
            // cr.arc(0.5, 0.5, 0.4, 0.0, PI * 2.);
            // cr.stroke().expect("Invalid cairo surface state");

            // // mouth
            // let mouth_top = 0.68;
            // let mouth_width = 0.38;

            // let mouth_dx = 0.10;
            // let mouth_dy = 0.10;

            // cr.move_to(0.50 - mouth_width / 2.0, mouth_top);
            // cr.curve_to(
            //     0.50 - mouth_dx,
            //     mouth_top + mouth_dy,
            //     0.50 + mouth_dx,
            //     mouth_top + mouth_dy,
            //     0.50 + mouth_width / 2.0,
            //     mouth_top,
            // );

            // println!("Extents: {:?}", cr.fill_extents());

            // cr.stroke().expect("Invalid cairo surface state");

            // let eye_y = 0.38;
            // let eye_dx = 0.15;
            // cr.arc(0.5 - eye_dx, eye_y, 0.05, 0.0, PI * 2.);
            // cr.fill().expect("Invalid cairo surface state");

            // cr.arc(0.5 + eye_dx, eye_y, 0.05, 0.0, PI * 2.);
            // cr.fill().expect("Invalid cairo surface state");
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
