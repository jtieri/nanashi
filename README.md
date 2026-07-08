# nanashi

nanashi is a terminal client for browsing 4chan. The name 名無し, "nameless",
is what anonymous posters were called on Japanese textboards.

nanashi is a fork of [tui-chan](https://github.com/tuqqu/tui-chan) that I'm rebuilding from the ground up.
It's early and rough, and plenty is going to move around while the rewrite lands,
so take anything here as a work in progress. The [roadmap](docs/ROADMAP.md) spells out where it's headed.

## Where it's going

The short version:

- vim-style navigation with remappable keybindings
- images and media drawn inline in the terminal (kitty graphics, with fallbacks
  for terminals that can't manage it)
- feature parity with the 4chan website: catalog view, working quote links,
  spoilers, archives, search, watching a thread for new replies, etc.
- saving media, and eventually posting
- other imageboards down the line, not only 4chan

## Installing

From crates.io:

```shell
cargo install nanashi
```

Or from source (you'll need [Rust](https://www.rust-lang.org/tools/install)):

```shell
git clone https://github.com/jtieri/nanashi.git
cd nanashi
cargo install --path .
```

Either way you get a `nanashi` binary in `~/.cargo/bin`. Run it with:

```shell
nanashi
```

To remove it later, `cargo uninstall nanashi`.

## Controls

Navigation is vim-style, and a count works in front of a motion, so `5j` moves
down five and `10G` jumps to the tenth item. Press `?` for the help bar.

| Action                          | Keys              |
|---------------------------------|-------------------|
| Move down / up                  | `j` / `k`         |
| Move between panes (back / in)  | `h` / `l`         |
| Jump to top / bottom            | `gg` / `G`        |
| Half page down / up             | `Ctrl-d` / `Ctrl-u` |
| Next / previous page            | `]` / `[`         |
| Reload                          | `r`               |
| Fullscreen the focused pane     | `f`               |
| Open thread/post in a browser   | `o`               |
| Open media in a browser         | `O`               |
| Copy thread/post url            | `y`               |
| Copy media url                  | `Y`               |
| Toggle the help bar             | `?`               |
| Quit                            | `q`               |

The bindings are built in for now. A config file to remap them is coming.

## Credits

nanashi started out as [tui-chan](https://github.com/tuqqu/tui-chan) by tuqqu.
See [NOTICE](NOTICE) for the attribution.

It isn't affiliated with 4chan. It just talks to the public API, so use it within
the site's rules.

## License

MIT, same as the original. See [LICENSE](LICENSE).
