# Data sources

`catalog.json` is an offline fallback snapshot, fetched on 12 September 2026.
Live data and a validated local cache take precedence.

- Boss names and categories are extracted from the OSRS Wiki's
  [Template:Bosses](https://oldschool.runescape.wiki/w/Template:Bosses), by
  [OSRS Wiki contributors](https://oldschool.runescape.wiki/w/Template:Bosses?action=history).
  Wiki text is available under [CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/).
  The extraction keeps encounter names, collapses grouped fights and raids, and
  sorts and deduplicates entries. It excludes images and descriptive text.
- Skill names are extracted from
  [Jagex's OSRS HiScores](https://secure.runescape.com/m=hiscore_oldschool/overall),
  excluding Overall and activities. RuneScape is a trademark of Jagex Ltd.

To regenerate the snapshot using the application's validated parser, run
`cargo run --locked -- refresh-data` and copy the `catalog` object from the
resulting cache JSON into `data/catalog.json`. Check the changes and update the
snapshot date here. Normal data updates do not require refreshing this fallback
or releasing a new application build.
