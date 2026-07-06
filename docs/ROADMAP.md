# Roadmap

nanashi is a fork of [tui-chan](https://github.com/tuqqu/tui-chan) that I'm
turning into a full client for 4chan, and eventually other imageboards. The work
is split into phases so it stays usable as it goes, instead of sitting
half-broken for months.

The order is on purpose: get the foundation onto a modern footing, make moving
around feel right, catch up to what you can already do on the site, then add the
things a terminal can do that a browser can't.

## Phase 1: foundation

Get onto a modern, non-blocking stack without changing what the app does yet.

- swap the unmaintained `tui` crate for `ratatui`, and `termion` for `crossterm`
- rework the event loop so a network request doesn't freeze the UI. Fetches run
  in the background and hand their results back to the render loop.
- fill out the data model to match the full 4chan post and board schema
- add the catalog, thread list, and archive endpoints
- behave as a polite API client: a real user agent, rate limiting, and caching
  with If-Modified-Since
- update dependencies and clean up along the way

## Phase 2: vim keybindings

Modal navigation you can remap.

- proper modes (normal, command, search), with key chords and counts so `gg`,
  `G`, and `5j` all work
- `hjkl` to move, `h` and `l` to jump between panes
- a `:` command line and `/` search
- a config format that can describe all of it, with a vim default you can
  override completely

## Phase 3: read parity

Everything you can do reading 4chan in a browser, from the terminal.

- a catalog view
- comments that render properly: quote links you can follow, greentext,
  spoilers, code blocks, dead links, and backlinks that show who replied
- poster IDs, flags, tripcodes, capcodes, and thread status
- reading archived threads
- searching and filtering inside a board
- watching a thread and getting told when it has new replies
- saving a post's media, or a whole thread's worth

## Phase 4: images and media

- inline images, two ways: show everything by default, or reveal the highlighted
  post's image on a keypress
- kitty graphics first, falling back to sixel and then to text where the terminal
  can't do better
- spoilered images stay hidden until you ask for them
- hand webm and other video off to something like mpv

## Later

- posting replies and starting threads. This one is hard. 4chan has no official
  write API, and posting runs through Cloudflare and a captcha, so the realistic
  version is writing your post in the terminal and handing off to a browser to
  submit it, which is easier if you have a Pass.
- more imageboards. The client already keeps sites behind an abstraction, and a
  lot of boards share one API, so adding them shouldn't cost much.
- local caching and offline reading
