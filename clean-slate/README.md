# Experiment: use const generics to parse fixed gcodes

- [x] Investigate a general purpose word parser using const fns

Findings

- Works very nicely with various inputs
- API is elegant
- Won't work with variables and expressions, but would do well in the fast path as part of an `alt()` list

# Experiment: parse `G0` and `G4 Pn` dwell

- [x] Parse these two items into their two modal group enums, `Motion` (group 1) and `NonModal` (group 0)
- [x] Collect the spans in the input for where they exist

Findings

- `insta` needs `std`
- `LocatedSpan`s can be created from `&str`s with `.into()`
- `cargo-insta` must be installed to use `insta`

# Experiment: add comment parsing to previous experiment

- [x] Parse comments and represent them in the `Word` struct.

# Experiment: extract block/word parsing into a no_std core and provide another crate to do span matching

> Just stick to what _I_ need - I don't mind std!

- [ ] Make input type generic over `LocatedSpan` or `&str`.
- [ ] A different approach would be feature flagging as I need to turn `String` comments on and off. Ugh.

# Experiment: replace nom_locate with something custom and the `consumed()` function

# Experiment: use nom_supreme's error tree

Findings

- ErrorTree is kind of annoying to use
- Perhaps I can do well enough using nom's builtin `context`.

# Experiment: Add `context` and print error stacks

Findings

- Very annoying to use

# Experiment: Remove `Span` from most of parser tree

# Experiment: Non-failing parser

- Parse a program with multiple errors _to the end_ and log/print all the errors and their locations.

# Experiment: dynamically load actix actors

- <https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html>
- <https://adventures.michaelfbryan.com/posts/plugins-in-rust/>

Another approach

- <https://fasterthanli.me/articles/so-you-want-to-live-reload-rust>

# Stage 1 Goals

- [-] ~Parser itself must be no_std by default~
- [x] Parse only these tokens (and valid NGC274 variants):
  - [x] `G0` rapid
  - [x] `G4 Pn` dwell
- [x] Expose functions to parse:
  - [x] A single word
  - [x] A block (line) of words
  - [x] A program
- [x] Use nom_locate's `LocatedSpan` everywhere, even if only parsing a block
- [-] ~Swappable error type~
  - [-] ~Parser can be no_std if desired, using Nom's builtin error type-
  - [-] ~If in a std environment, nom_supreme's ErrorTree can be used to give nicer output-
- [x] Tests use `insta` to assert against a parsed tree
