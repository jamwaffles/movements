extern crate criterion;

use criterion::*;
use gcode_parser::GcodeProgram;

macro_rules! bench_file {
    ($fn_name:ident, $suite_name:expr, $file_name:expr) => {
        fn $fn_name(c: &mut Criterion) {
            let program = include_str!(concat!("../../test_files/", $suite_name, "/", $file_name,));

            c.bench(
                $suite_name,
                Benchmark::new($file_name, move |b| {
                    b.iter(|| GcodeProgram::from_str(black_box(program)).unwrap())
                })
                .throughput(Throughput::Bytes(program.len() as u64)),
            );
        }
    };
}

bench_file! {
    universal_gcode_sender_serial_stress_test,
    "universal_gcode_sender",
    "serial_stress_test.gcode"
}

bench_file!(
    universal_gcode_sender_buffer_stress_test,
    "universal_gcode_sender",
    "buffer_stress_test.gcode"
);

bench_file!(huge_tiger, "tinyg", "tiger.gcode");

bench_file!(tinyg_mudflap_100in, "tinyg", "mudflap_100in.gcode");

bench_file!(tinyg_zoetrope, "tinyg", "zoetrope.gcode");

bench_file!(linuxcnc_skeleton_ngc, "linuxcnc", "skeleton.ngc");

bench_file!(linuxcnc_b_index_ngc, "linuxcnc", "b-index.ngc");

bench_file!(linuxcnc_smartprobe, "linuxcnc", "smartprobe.ngc");

criterion_group!(
    file,
    universal_gcode_sender_serial_stress_test,
    universal_gcode_sender_buffer_stress_test,
    huge_tiger,
    tinyg_mudflap_100in,
    tinyg_zoetrope,
    linuxcnc_skeleton_ngc,
    linuxcnc_b_index_ngc,
    linuxcnc_smartprobe,
);
criterion_main!(file);
