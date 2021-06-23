# Pull parser

Declare `Number` type as `f32` to begin with. Open up possibility of using `fixed` or `f64` later.

## Case 1: No alloc, no std

Use case: 3D printers, simpler machines drip fed from other devices. Unidirectional program flow.

Procedures nor control flow are supported. Variables are supported.

1. Parse input as a bunch of blocks
2. Parse each word from each block into a `Token` struct with position in the original input and a `Word` enum.

Variable assignment is handled in the interpreter/state machine. This could be a fixed array for numeric vars and a `heapless` `Vec` for named variables.

## Case 2: Alloc

Use case: rPi/PC based machine controls with more memory and need/desire to validate/backplot the program.

- Add methods to parse entire program into a tree of `Token`s.
  - Support procedures
  - Support control flow
- Add extra items to `Word` enum, or make a new `ComplexWord` (need a better name) enum with `Word` nested inside it

# Alloc-only: `ProgramTree`

A struct which can consume the pull parser into a token tree, allowing control flow and procedures and stuff.

# Interpreter

Interpreter provides a type param for named variable storage. This would need to implement a trait, but could use a static array (const generics) or a `heapless` `HashMap` (more cargo features here?) or a normal `alloc::HashMap` depending on std or no_std.




## Case 1: No alloc, no std


## Case 2: Alloc


# Parser/interpreter/executor arch

Targets both PC/rPi with realtime and non-realtime threads, as well as RTIC applications. Use queues/message passing to communicate between components.

- Why is IO in a non-realtime thread?
  - Doesn't really need servo-timing realtime
  - But the limit switches are in the realtime thread because they're handled in the motion controller because it does homing
- How does the HAL work?
  - This is a big one. RTAPI can register functions with `hal_export_funct`. See also <http://www.linuxcnc.org/docs/html/man/man3/hal_create_thread.3hal.html>. Period is decided in `hal_create_thread` call.
- Motion and IO controllers are driven by interpreter
- Motion controller holds and reports distance to go and stuff like that
- Interpreter stores state of variables
  - Updates from HAL come up through the realtime thread and into the interpreter
- How do external realtime plugins work?
  - Dylibs?
  - Expose an API to register multiple RT threads at some tick rate each.
- Expose API to register pins.
  - Might need shared memory for this
  - Test if message passing is fast enough. Might just be with 0MQ

Take a look [here](https://github.com/rust-lang/rust/blob/99e3aef02079e9c10583638520cd0c134dc3a01d/library/std/src/sys/unix/thread.rs) for the Rust thread spawning plumbing. I could copy and paste this into my own Thread struct but with the ability to specify the priority, but using `libc` directly.

The scheduling policy is defined [here](https://wiki.linuxfoundation.org/realtime/documentation/technical_basics/sched_policy_prio/start) (from [this page](https://wiki.linuxfoundation.org/realtime/documentation/howto/applications/application_base)). Tldr - use `SCHED_FIFO`.

So, sleeping on it, I should test message passing instead of shared memory for latency/throughput.
