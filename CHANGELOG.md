# Changelog

All notable changes to nanashi are recorded here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims to
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.1.0] - 2026-07-07


### Build System

- Add changelog tooling and repo housekeeping
- Exclude repo-only files from the published crate

### CI

- Modernize the workflow
- Drop the xcb libs now that arboard uses pure-rust x11

### Documentation

- Rewrite the docs and credit the original
- Tighten readme and contributing wording

### Features

- Migrate rendering from tui to ratatui and input to crossterm
- Run fetches on an async event loop
- Add a status bar with a loading spinner
- Expand board and post to the full 4chan schema
- Send a user agent and rate limit requests
- Add the catalog, threads, and archive endpoints

### Miscellaneous

- Fork tui-chan and rename the crate to nanashi

### Refactor

- Introduce action/effect architecture with pane components
- Replace clipboard with arboard and drop unused deps
- Move config to ~/.config/nanashi

### Testing

- Cover the update transitions
- Cover deserialization of the api schema and the rate limiter

### Style

- Spell out elided lifetimes
