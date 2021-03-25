mod traj_1d_trapezoidal;
mod traj_nd_trapezoidal;

use nalgebra::Vector3;
use std::cell::RefCell;
use std::panic;
use std::rc::Rc;
use traj_nd_trapezoidal::{Limits, Point, TrapezoidalLineSegment};
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

fn display_hover(
    container: &Element,
    (time, pos, vel, acc): (f32, Vector3<f32>, Vector3<f32>, Vector3<f32>),
) {
    container.set_inner_html(&format!(
        "Time:         {:0.2}
Position:     {:0.2}, {:0.2}, {:0.2}
Velocity:     {:0.2}, {:0.2}, {:0.2}
Acceleration: {:0.2}, {:0.2}, {:0.2}",
        time, pos[0], pos[1], pos[2], vel[0], vel[1], vel[2], acc[0], acc[1], acc[2]
    ));
}

fn draw_profiles(
    context: &CanvasRenderingContext2d,
    segment: &TrapezoidalLineSegment,
    width: u32,
    height: u32,
) {
    context.clear_rect(0.0, 0.0, width as f64, height as f64);

    for i in 0..(Vector3::<f32>::zeros().len()) {
        draw_axis_profiles(context, segment, width, height, i)
    }

    // draw_axis_profiles(context, segment, width, height, 2);
}

fn draw_axis_profiles(
    context: &CanvasRenderingContext2d,
    segment: &TrapezoidalLineSegment,
    width: u32,
    height: u32,
    index: usize,
) {
    let y_scale = 15.0;

    // let baseline = (height / 2) + (index as u32 * 10);
    let baseline = (height / 2) + (index as u32 * 0);

    // context.set_line_width((index + 1) as f64);

    context.begin_path();
    context.set_stroke_style(&("#aaa".into()));
    context.move_to(0.0, baseline as f64);
    context.line_to(width as f64, baseline as f64);
    context.stroke();
    context.close_path();

    let points = (0..width)
        .filter_map(|i| {
            let time = segment.max_duration() * i as f32 / width as f32;

            if time > segment.duration()[index as usize] {
                return None;
            }

            segment
                .position(time)
                .map(|(Point { position, velocity }, acceleration)| {
                    let position = position[index];
                    let velocity = velocity[index];
                    let acceleration = acceleration[index];

                    let position = baseline as f32 - position * y_scale;
                    let velocity = baseline as f32 - velocity * y_scale + 3.0;
                    let acceleration = baseline as f32 - acceleration * y_scale + 6.0;

                    let x = i as f64;

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

    let width = 1000;
    let height = 800;

    canvas.set_width(width);
    canvas.set_height(height);

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let segment = TrapezoidalLineSegment::new(
        Limits {
            velocity: Vector3::repeat(2.0),
            acceleration: Vector3::repeat(5.0),
        },
        Point {
            // position: Vector3::repeat(0.0),
            position: Vector3::new(2.0, 0.0, 3.0),
            velocity: Vector3::new(1.0, 0.0, 5.0),
            // velocity: Vector3::zeros(),
        },
        Point {
            position: Vector3::repeat(10.0),
            velocity: Vector3::new(0.0, 0.0, 2.0),
            // velocity: Vector3::zeros(),
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
            let max_velocity = Vector3::repeat(max_velocity);

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
            let max_acceleration = Vector3::repeat(max_acceleration);

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
            let start_velocity = Vector3::repeat(start_velocity);

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
        let controls = controls;
        let out = out;
        let segment = segment.clone();
        let context = context;

        let closure = Closure::wrap(Box::new(move |event: web_sys::InputEvent| {
            let value = event
                .target()
                .as_ref()
                .map(|t| wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlInputElement>(t))
                .expect("Unable to get value")
                .expect("Unable to get value")
                .value();

            let end_velocity = value.parse::<f32>().expect("Value is not valid f32");
            let end_velocity = Vector3::repeat(end_velocity);

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
        let hover = hover;
        let segment = segment;

        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let x = event.offset_x();

            let time = segment.borrow().max_duration() * x as f32 / width as f32;

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
