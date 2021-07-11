use gcode_interpreter::GcodeInterpreter;
use gcode_parser::{tokens::Motion, GcodeProgram};
use kiss3d::light::Light;
use kiss3d::window::Window;
use nalgebra::Point3;

fn main() {
    let mut window = Window::new("Gcode Viewer");

    window.set_light(Light::StickToCamera);

    let rapid_color = Point3::new(1.0, 0.0, 0.0);
    let feed_color = Point3::new(0.0, 1.0, 1.0);

    let program = GcodeProgram::from_str(
        r#"G0 X0 Y0 Z0
        X10 Y20 Z30
        Z10
        G1 Z-2
        X0 Y0
        Z5
        G0 Z30"#,
    )
    .unwrap();

    let interp = GcodeInterpreter::new(&program);

    while window.render() {
        // Floor
        {
            for y in (-25..=25).step_by(5) {
                window.draw_line(
                    &Point3::new(-25.0, 0.0, y as f32),
                    &Point3::new(25.0, 0.0, y as f32),
                    &Point3::new(0.5, 0.5, 0.5),
                );
            }

            for x in (-25..=25).step_by(5) {
                window.draw_line(
                    &Point3::new(x as f32, 0.0, -25.0),
                    &Point3::new(x as f32, 0.0, 25.0),
                    &Point3::new(0.5, 0.5, 0.5),
                );
            }
        }

        for parts in interp.block_iter().collect::<Vec<_>>().windows(2) {
            match parts {
                [a, b] => {
                    let a = a.as_ref().unwrap();
                    let b = b.as_ref().unwrap();

                    let color = match a.motion {
                        Motion::Rapid => rapid_color,
                        Motion::Feed => feed_color,
                    };

                    let a_pos = a.next_position;
                    let b_pos = b.next_position;

                    window.draw_line(
                        &Point3::new(a_pos[0], a_pos[1], a_pos[2]),
                        &Point3::new(b_pos[0], b_pos[1], b_pos[2]),
                        &color,
                    );
                }
                _ => unreachable!(),
            }
        }
    }
}
