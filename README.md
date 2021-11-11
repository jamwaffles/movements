# Movements

[![Build Status](https://circleci.com/gh/jamwaffles/movements/tree/master.svg?style=shield)](https://circleci.com/gh/jamwaffles/movements/tree/master)

Maybe a CNC controller one day.

## GTK setup on macOS Monterey

GTK applications on macOS Monterey show a black screen on startup. To fix, remove any existing GTK deps installed with Homebrew and run:

```bash
brew install ttarhan/gtk-fix/gtk+3
```

(from <https://github.com/ttarhan/homebrew-gtk-fix>)
