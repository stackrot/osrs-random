# OSRS Random Generator

A command-line boss and skill chooser for Old School RuneScape, for Linux and
Windows.

## Installation

Download `osrs-random-linux.zip` or `osrs-random-windows.zip` from the
[latest release](https://github.com/stackrot/osrs-random/releases/latest).
Extract it and run `osrs-random` or `osrs-random.exe`. Linux builds require
x86-64 and glibc 2.35 or newer; Windows builds target x86-64.

On Linux, if needed:

```sh
chmod +x osrs-random
./osrs-random
```

Existing versions need one manual download to gain the self-updater.

## Usage

Run without arguments for the interactive menu. Commands return immediately
without prompts, pauses or terminal clearing:

```sh
osrs-random boss
osrs-random boss --exclude 'The Wilderness bosses' --exclude 'Raids'
osrs-random skill
osrs-random list-bosses
osrs-random list-skills
osrs-random version
osrs-random --help
```

Category names are case-insensitive; use `list-bosses` for current names. The
interactive boss chooser lets you exclude categories by number. Selection keeps
the original behaviour: choose a category uniformly, then a boss within it.

## Live boss and skill data

Bosses and categories come from the OSRS Wiki's
[Bosses template](https://oldschool.runescape.wiki/w/Template:Bosses). Skills come
from [Jagex's OSRS HiScores](https://secure.runescape.com/m=hiscore_oldschool/overall).
New entries appear after the upstream source lists them and the cache refreshes;
no application release is needed. The Wiki determines boss categories. Grouped
encounters such as Barrows and raids are kept together; boss variants are grouped
under the first encounter name in the Wiki entry.

The catalogue refreshes on first use and when its cache is at least 24 hours old.
Each interactive session reuses its loaded catalogue. Force a refresh with:

```sh
osrs-random refresh-data
```

If a fetch fails or a source returns incomplete data, the app keeps the last valid
cache, or uses a bundled snapshot on first run. It reports when it falls back.
An explicit `refresh-data` failure returns a non-zero exit status and preserves
the existing cache.

```sh
osrs-random --offline boss
osrs-random --offline list-skills
```

`--offline` disables data fetches and the interactive update check. The cache is
stored in `$XDG_CACHE_HOME/osrs-random/catalog-v1.json` (normally
`~/.cache/osrs-random/catalog-v1.json`) on Linux, or
`%LOCALAPPDATA%\osrs-random\cache\catalog-v1.json` on Windows.

Source outages and markup changes can delay new data. See
[data sources and attribution](data/README.md) for the bundled snapshot.

## Self-updating

```sh
osrs-random update --check
osrs-random update
```

The interactive menu checks for updates at startup and offers to install them.
Declining continues normally. `update` explicitly installs the latest newer
release from this repository's GitHub Releases. It downloads the ZIP for the
compiled target, checks its size and GitHub SHA-256 digest, and replaces the
executable. Restart the app afterwards. Its directory must be writable.

Official binaries embed the timestamped release tag, so repeated checks recognise
an installed release and older releases never replace newer ones. `version`
prints the package version and embedded tag without accessing the network.
Source builds without a release tag cannot be compared with timestamped releases;
install an official release to use the updater. Other architectures and ABIs must
be built and updated from source.

## Development

Install current stable Rust, then:

```sh
cargo build --locked
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

Tests use local fixtures and do not require the live data sources. CI checks Linux
and Windows. Pushes to `master` build both targets with the same release tag and
publish only after checks and both uploads complete. Dependencies are locked in
`Cargo.lock`.

Report bugs or data-source problems on the
[issue tracker](https://github.com/stackrot/osrs-random/issues).

## Licence

Application code is [MIT licensed](LICENSE). See [data attribution](data/README.md)
for OSRS Wiki content.
