Block diff?

If group one is already `Some(Change)` error?

M2 is an outlier as it checks final program state

---

m3 s400 m4

- Set M group Spindle to clockwise rotation
- Set spindle speed to 400 RPM
- Set M group Spindle to off - Errors because MSpindle() is already Some

---

Cooant (bit of a special case)

m7 m8 m9

- Set mist(?) on
- Set flood(?) on
- Set all coolant off - Errors because either mist status or flood status has changed

---

t5 m6 g43

- Set next tool to t5
- Change tool command
  - (when tool is changed, current = next, next = none)
- Enable tool offset comp or whatever g43 is
