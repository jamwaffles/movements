use gtk::cairo::{Context, FontSlant, FontWeight};
use gtk::Adjustment;
use gtk::ApplicationWindow;
use gtk::Builder;
use gtk::DrawingArea;
use gtk::MessageDialog;
use gtk::{prelude::*, Scale};
use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::Arc;

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("./gtk-demo.glade");
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

    // drawing_area.connect_configure_event(|drawing_area, _| {
    //     println!("I am great");

    //     dbg!(drawing_area.allocation());

    //     drawing_area.show();

    //     true
    // });

    let mut value = Rc::new(Cell::new(0.0f64));

    let slider: Scale = builder.object("slider1").expect("Slider 1");
    let adjustment1 = Adjustment::new((*value).get(), 0.0, 2.0, 0.01, 0.05, 0.0);
    slider.set_adjustment(&adjustment1);

    slider.connect_value_changed(
        glib::clone!(@weak value, @weak drawing_area => move |scale| {
            value.set(scale.value());

            drawing_area.queue_draw();
        }),
    );

    builder.connect_signals(move |_, handler_name| {
        // This is the one-time callback to register signals.
        // Here we map each handler name to its handler.

        if handler_name == "button1_clicked" {
            // Return the signal handler.
            Box::new(
                glib::clone!(@weak dialog => @default-return None, move |_| {
                    dialog.show_all();
                    None
                }),
            )
        } else {
            panic!("Unknown handler name {}", handler_name)
        }
    });

    drawing_area.connect_draw(move |drawing_area, cr| {
        println!("Value {:?}", value);

        let dimensions = drawing_area.allocation();

        let width = f64::from(dimensions.width);
        let height = f64::from(dimensions.height);
        // cr.scale(dimensions.width.into(), dimensions.height.into());

        cr.set_source_rgb(250.0 / 255.0, 224.0 / 255.0, 55.0 / 255.0);
        cr.paint().expect("Invalid cairo surface state");

        cr.set_line_width(0.5);
        cr.set_source_rgb(0.0, 0.0, 0.0);

        cr.move_to(10.0, 10.0);
        cr.line_to(
            dimensions.width as f64 - 10.0,
            10.0 + (value.get() * (height - 20.0)),
        );
        cr.stroke().expect("Invalid cairo surface state");

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
    });

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

// pub fn drawable<F>(drawing_area: &gtk::DrawingArea, width: i32, height: i32, draw_fn: F)
// where
//     F: Fn(&DrawingArea, &Context) -> Inhibit + 'static,
// {
//     // let drawing_area = Box::new(DrawingArea::new)();

//     drawing_area.connect_draw(draw_fn);

//     // window.set_default_size(width, height);

//     // window.add(&drawing_area);
//     // window.show_all();
// }
