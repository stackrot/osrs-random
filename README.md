# OSRS Random Generator

A command-line boss and skill chooser for Old School RuneScape, for Linux and Windows.

## Installation

Download and extract the [latest release](https://github.com/stackrot/osrs-random/releases/latest):

- **Windows:** use `osrs-random-windows.zip` and run `osrs-random.exe`.
- **Linux:** use `osrs-random-linux.zip`, then run:

  ```sh
  chmod +x osrs-random
  ./osrs-random
  ```

Both builds require x86-64; Linux also requires glibc 2.35 or newer.

## Usage

Run without arguments for the interactive menu, or use these commands:

| Command | Action |
| --- | --- |
| `osrs-random boss` | Choose a boss |
| `osrs-random skill` | Choose a skill |
| `osrs-random list-bosses` | List bosses by category |
| `osrs-random list-skills` | List skills |
| `osrs-random version` | Show the installed version |
| `osrs-random --help` | Show all options |

Exclude categories by name, using `list-bosses` to find the current names:

```sh
osrs-random boss --exclude "The Wilderness bosses" --exclude "Raids"
```

Category names are case-insensitive; the interactive menu also lets you exclude them by number.

## Boss and skill data

Bosses and categories come from the [OSRS Wiki](https://oldschool.runescape.wiki/w/Template:Bosses), and skills from [Jagex's HiScores](https://secure.runescape.com/m=hiscore_oldschool/overall).

Lists refresh on first use and after 24 hours, so new entries need no app update.

Cached or bundled data is used when the sources are unavailable.

| Command | Action |
| --- | --- |
| `osrs-random refresh-data` | Refresh the lists now |
| `osrs-random --offline boss` | Choose a boss without network access |

## Updates

The interactive menu offers newer releases at startup, or you can update manually:

```sh
osrs-random update --check
osrs-random update
```

Updates verify the download's SHA-256 digest before replacing the executable.

Restart afterwards; the installation directory must be writable.

Older installations need one manual download to gain the updater; source builds must be updated from source.

## Development

With current stable Rust installed:

```sh
cargo build --locked
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

CI checks Linux and Windows; tests run without network access.

Report bugs on the [issue tracker](https://github.com/stackrot/osrs-random/issues).

## Licence

[MIT](LICENSE) for application code; see [data attribution](data/README.md) for OSRS Wiki content.
