# Experiment: use const generics to parse fixed gcodes

- [x] Investigate a general purpose word parser using const fns

Findings

- Works very nicely with various inputs
- API is elegant
- Won't work with variables and expressions, but would do well in the fast path as part of an `alt()` list

# Experiment: parse `G0` and `G4 Pn` dwell

- [ ] Parse these two items into their two modal group enums, `Motion` (group 1) and `NonModal` (group 0)
- [ ] Collect the spans in the input for where they exist

Findings

- `insta` needs `std`
- `LocatedSpan`s can be created from `&str`s with `.into()`
- `cargo-insta` must be installed to use `insta`

# Experiment: add comment parsing to previous experiment

- [ ] Parse comments and represent them in the `Word` struct.

# Stage 1 Goals

- [ ] Parser itself must be no_std by default
- [ ] Parse only these tokens (and valid NGC274 variants):
  - [ ] `G0` rapid
  - [ ] `G4 Pn` dwell
- [ ] Expose functions to parse:
  - [ ] A single word
  - [ ] A block (line) of words returned as an iterator
  - [ ] A program, returned as an iterator of blocks
- [ ] Use nom_locate's `LocatedSpan` everywhere, even if only parsing a block
- [ ] Swappable error type
  - [ ] Parser can be no_std if desired, using Nom's builtin error type
  - [ ] If in a std environment, nom_supreme's ErrorTree can be used to give nicer output
- [ ] Tests use `insta` to assert against a parsed tree
