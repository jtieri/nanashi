# nanashi

A terminal client for reading 4chan. Boards on the left, threads in the middle,
replies on the right, all driven from the keyboard.

The name is 名無し, "nameless", which is what anonymous posters were called on the
old Japanese textboards. moot translated it as "Anonymous" when he started 4chan,
so it seemed like a good fit for a client that isn't meant to stay tied to any
one site.

nanashi is a fork of [tui-chan](https://github.com/tuqqu/tui-chan) that I'm
rebuilding from the ground up. It's early and rough, and plenty is going to move
around while the rewrite lands, so take anything here as a work in progress. The
[roadmap](docs/ROADMAP.md) spells out where it's headed.

## Where it's going

The short version:

- vim-style navigation you can remap to taste
- images and media drawn inline in the terminal (kitty graphics, with fallbacks
  for terminals that can't manage it)
- the rest of what you'd do reading the site in a browser: catalog view, working
  quote links, spoilers, archives, search, watching a thread for new replies
- saving media, and eventually posting
- other imageboards down the line, not only 4chan

## Building

You'll need [Rust](https://www.rust-lang.org/tools/install).

```shell
git clone https://github.com/jtieri/nanashi.git
cd nanashi
cargo install --path .
```

That puts a `nanashi` binary in `~/.cargo/bin`. Run it with:

```shell
nanashi
```

To get rid of it later, `cargo uninstall nanashi`.

## Controls

These are the defaults inherited from tui-chan, and they'll change once the vim
keybindings land. They live in `~/.config/tui-chan/keybinds.conf` and can be
remapped.

Press `h` for the help bar. `d` opens the highlighted board or thread, and `a`
steps back a pane.

| Action                                        | Keys                      |
|-----------------------------------------------|---------------------------|
| Move around                                   | `w`, `a`, `s`, `d`        |
| Move quickly                                  | control + `w`/`a`/`s`/`d` |
| Toggle the help bar                           | `h`                       |
| Next / previous page                          | `p` / control + `p`       |
| Reload the page                               | `r`                       |
| Fullscreen the selected panel                 | `z`                       |
| Copy the selected thread or post url          | `c`                       |
| Copy the selected post's media url            | control + `c`             |
| Open the selected thread or post in a browser | `o`                       |
| Open the selected post's media in a browser   | control + `o`             |
| Quit                                          | `q`                       |

## Credits

nanashi started out as [tui-chan](https://github.com/tuqqu/tui-chan) by tuqqu.
See [NOTICE](NOTICE) for the attribution.

It isn't affiliated with 4chan. It just talks to the public API, so use it within
the site's rules.

## License

MIT, same as the original. See [LICENSE](LICENSE).
