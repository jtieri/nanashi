# Contributing

Thanks for looking. nanashi is in the middle of a rewrite, so if you're planning
anything big it's worth opening an issue first so we can talk it over before you
sink time into it.

## Commits

Commits follow [Conventional Commits](https://www.conventionalcommits.org/). Put
a type at the front of the subject, an optional scope in parentheses, then a
short description written as an instruction:

```
feat(keybinds): add gg and G motions
fix(client): treat a missing tim as no attachment
docs: explain the config file
refactor(app): pull pane state out of App
```

The types we use:

- `feat` for a new feature
- `fix` for a bug fix
- `refactor` when behavior stays the same
- `docs` for documentation
- `test` for tests
- `ci` for CI and workflow changes
- `build` for dependencies and build tooling
- `chore` for anything that doesn't fit the rest

The changelog is built from these messages, so a clear subject line earns its
keep.

## History

Keep it linear. Rebase your branch onto `main` instead of merging `main` into it,
and clean up the "wip" and typo commits before you open a PR. The goal is that
the log reads like a straight account of how the project got here.

## Before you push

Run the same three things CI does:

```shell
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

Anything you contribute goes in under MIT, same as the rest of the project.
