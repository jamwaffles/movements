mod traj_1d_trapezoidal;

use console_error_panic_hook;
use std::cell::RefCell;
use std::panic;
use std::rc::Rc;
use traj_1d_trapezoidal::{Limits, Point, TrapezoidalLineSegment};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use web_sys::Element;

struct Controls {
    max_velocity: Element,
    max_acceleration: Element,
    start_velocity: Element,
    end_velocity: Element,
}

fn display_config(container: &Element, config: &TrapezoidalLineSegment) {
    container.set_inner_html(&format!("{:#?}", config));
}

fn display_hover(container: &Element, (time, pos, vel, acc): (f32, f32, f32, f32)) {
    container.set_inner_html(&format!(
        "Time:         {}\nPosition:     {}\nVelocity:     {}\nAcceleration: {}",
        time, pos, vel, acc
    ));
}

fn draw_profiles(
    context: &CanvasRenderingContext2d,
    segment: &TrapezoidalLineSegment,
    width: u32,
    height: u32,
) {
    context.clear_rect(0.0, 0.0, width as f64, height as f64);

    let y_scale = 10.0;

    // // Position
    // context.begin_path();
    // context.set_stroke_style(&("#000".into()));

    // for i in 0..width {
    //     let time = segment.duration() * i as f32 / width as f32;

    //     let y = (height / 2) as f32 - segment.position(time) * y_scale;

    //     if i == 0 {
    //         context.move_to(0.0, y as f64);
    //     } else {
    //         context.line_to((width / width) as f64 * i as f64, y as f64);
    //     }
    // }

    // context.stroke();
    // context.close_path();

    let points = (0..width)
        .filter_map(|i| {
            let time = segment.duration() * i as f32 / width as f32;

            segment
                .position(time)
                .map(|(Point { position, velocity }, acceleration)| {
                    let position = (height / 2) as f32 - position * y_scale;
                    let velocity = (height / 2) as f32 - velocity * y_scale;
                    let acceleration = (height / 2) as f32 - acceleration * y_scale;

                    let x = (width / width) as f64 * i as f64;

                    (x, position, velocity, acceleration)
                })
        })
        .collect::<Vec<_>>();

    context.begin_path();
    context.set_stroke_style(&("#f00".into()));
    context.move_to(0.0, points.get(0).map(|p| p.1).unwrap_or(0.0) as f64);
    for (x, position, _, _) in points.iter() {
        context.line_to(*x, *position as f64);
    }
    context.stroke();
    context.close_path();

    context.begin_path();
    context.set_stroke_style(&("darkgreen".into()));
    context.move_to(0.0, points.get(0).map(|p| p.2).unwrap_or(0.0) as f64);
    for (x, _, velocity, _) in points.iter() {
        context.line_to(*x, *velocity as f64);
    }
    context.stroke();
    context.close_path();

    context.begin_path();
    context.set_stroke_style(&("#00f".into()));
    context.move_to(0.0, points.get(0).map(|p| p.3).unwrap_or(0.0) as f64);
    for (x, _, _, acceleration) in points.iter() {
        context.line_to(*x, *acceleration as f64);
    }
    context.stroke();
    context.close_path();
}

#[wasm_bindgen]
pub fn start(container: web_sys::HtmlDivElement) -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Trace).ok();

    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    container.prepend_with_node_1(&canvas)?;

    let control_inputs = container
        .query_selector(".demo-controls")?
        .expect("Element .demo-controls does not exist");

    let out = container
        .query_selector(".out")?
        .expect("Element .out is missing");

    let hover = container
        .query_selector(".hover")?
        .expect("Element .hover is missing");

    let controls = Controls {
        max_velocity: control_inputs
            .query_selector("[name=max_velocity]")?
            .expect("Required input name max_velocity missing"),
        max_acceleration: control_inputs
            .query_selector("[name=max_acceleration]")?
            .expect("Required input name max_velocity missing"),
        start_velocity: control_inputs
            .query_selector("[name=start_velocity]")?
            .expect("Required input name start_velocity missing"),
        end_velocity: control_inputs
            .query_selector("[name=end_velocity]")?
            .expect("Required input name end_velocity missing"),
    };

    let width = 640;
    let height = 480;

    canvas.set_width(width);
    canvas.set_height(height);

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let segment = TrapezoidalLineSegment::new(
        Limits {
            velocity: 2.0,
            acceleration: 5.0,
        },
        Point {
            position: 0.0,
            velocity: 0.0,
        },
        Point {
            position: 10.0,
            velocity: 0.0,
        },
    );

    draw_profiles(&context, &segment, width, height);
    display_config(&out, &segment);

    let controls = Rc::new(controls);
    let out = Rc::new(out);
    let segment = Rc::new(RefCell::new(segment));
    let context = Rc::new(RefCell::new(context));

    // Velocity limit handler
    {
        let controls = controls.clone();
        let out = out.clone();
        let segment = segment.clone();
        let context = context.clone();

        let closure = Closure::wrap(Box::new(move |event: web_sys::InputEvent| {
            let value = event
                .target()
                .as_ref()
                .map(|t| wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlInputElement>(t))
                .expect("Unable to get value")
                .expect("Unable to get value")
                .value();

            let max_velocity = value.parse::<f32>().expect("Value is not valid f32");

            segment.borrow_mut().set_velocity_limit(max_velocity);

            display_config(&out, &segment.borrow());
            draw_profiles(&context.borrow(), &segment.borrow(), width, height);
        }) as Box<dyn FnMut(_)>);

        controls
            .max_velocity
            .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Acceleration limit handler
    {
        let controls = controls.clone();
        let out = out.clone();
        let segment = segment.clone();
        let context = context.clone();

        let closure = Closure::wrap(Box::new(move |event: web_sys::InputEvent| {
            let value = event
                .target()
                .as_ref()
                .map(|t| wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlInputElement>(t))
                .expect("Unable to get value")
                .expect("Unable to get value")
                .value();

            let max_acceleration = value.parse::<f32>().expect("Value is not valid f32");

            segment
                .borrow_mut()
                .set_acceleration_limit(max_acceleration);

            display_config(&out, &segment.borrow());
            draw_profiles(&context.borrow(), &segment.borrow(), width, height);
        }) as Box<dyn FnMut(_)>);

        controls
            .max_acceleration
            .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Start velocity handler
    {
        let controls = controls.clone();
        let out = out.clone();
        let segment = segment.clone();
        let context = context.clone();

        let closure = Closure::wrap(Box::new(move |event: web_sys::InputEvent| {
            let value = event
                .target()
                .as_ref()
                .map(|t| wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlInputElement>(t))
                .expect("Unable to get value")
                .expect("Unable to get value")
                .value();

            let start_velocity = value.parse::<f32>().expect("Value is not valid f32");

            segment.borrow_mut().set_start_velocity(start_velocity);

            display_config(&out, &segment.borrow());
            draw_profiles(&context.borrow(), &segment.borrow(), width, height);
        }) as Box<dyn FnMut(_)>);

        controls
            .start_velocity
            .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // End velocity handler
    {
        let controls = controls.clone();
        let out = out.clone();
        let segment = segment.clone();
        let context = context.clone();

        let closure = Closure::wrap(Box::new(move |event: web_sys::InputEvent| {
            let value = event
                .target()
                .as_ref()
                .map(|t| wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlInputElement>(t))
                .expect("Unable to get value")
                .expect("Unable to get value")
                .value();

            let end_velocity = value.parse::<f32>().expect("Value is not valid f32");

            segment.borrow_mut().set_end_velocity(end_velocity);

            display_config(&out, &segment.borrow());
            draw_profiles(&context.borrow(), &segment.borrow(), width, height);
        }) as Box<dyn FnMut(_)>);

        controls
            .end_velocity
            .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Mousemove handler
    {
        let hover = hover.clone();
        let segment = segment.clone();

        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let x = event.offset_x();

            let time = segment.borrow().duration() * x as f32 / width as f32;

            if let Some((Point { position, velocity }, acceleration)) =
                segment.borrow().position(time)
            {
                display_hover(&hover, (time, position, velocity, acceleration));
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(())
}
