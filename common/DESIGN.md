# no_std parser and interpreter

use [rstest](https://crates.io/crates/rstest) for parser/case testing.

3 crates:

1. `parser` - no_std Nom-based parser that can parse a single block (line) into multiple tokens
2. `interpreter` - interpret multiple commands and set some state
3. `common` - common stuff passed between `parser` and `interpreter`.

In the future, there will be a `validator` crate which would be called on an entire block before it's queued into the interpreter.

Interpreter must accept commands to queue up to change the state. It should have the ability to get the current state, as well as pop a command off the queue and mutate the state based on it.

```rust
// --- common

enum ModalGroup {
    Motion(Motion)
}

enum Motion {
    Rapid,
    Feed,
}

enum Units {
    Mm,
    Inch
}

// --- interpreter

struct Interp {
    modal_groups: ModalGroupState,
    command_queue: Vec<ModalGroup>
}

struct ModalGroupState {
// Modal group 1
motion: Motion,
    units: Units,
}

impl Interp {
    //
}
```
