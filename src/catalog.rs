use anyhow::{ensure, Context, Result};
use directories::ProjectDirs;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const WIKI_URL: &str = "https://oldschool.runescape.wiki/api.php";
const SKILLS_URL: &str = "https://secure.runescape.com/m=hiscore_oldschool/overall";
const CACHE_AGE: u64 = 24 * 60 * 60;
const MAX_DATA_SIZE: u64 = 4 * 1024 * 1024;

#[derive(Debug, Deserialize, Serialize)]
pub struct Catalog {
    pub bosses: BTreeMap<String, Vec<String>>,
    pub skills: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct Cache {
    fetched_at: u64,
    catalog: Catalog,
}

impl Catalog {
    fn validate(&self) -> Result<()> {
        ensure!(self.skills.len() >= 23, "Incomplete skill list");
        ensure!(self.bosses.len() >= 5, "Incomplete boss categories");
        ensure!(
            self.bosses.values().map(Vec::len).sum::<usize>() >= 40,
            "Incomplete boss list"
        );
        ensure!(
            self.bosses.values().all(|bosses| !bosses.is_empty()),
            "Empty boss category"
        );
        for name in self
            .skills
            .iter()
            .chain(self.bosses.keys())
            .chain(self.bosses.values().flatten())
        {
            ensure!(
                !name.trim().is_empty() && name.len() <= 200 && !name.chars().any(char::is_control),
                "Invalid catalogue entry"
            );
        }
        Ok(())
    }

    fn fetch() -> Result<Self> {
        let client = crate::net::client()?;
        let response = client
            .get(WIKI_URL)
            .query(&[
                ("action", "parse"),
                ("page", "Template:Bosses"),
                ("prop", "text"),
                ("format", "json"),
                ("formatversion", "2"),
            ])
            .send()
            .context("Could not fetch bosses from the OSRS Wiki")?;
        let data = crate::net::read_response(response, MAX_DATA_SIZE)?;
        let wiki: serde_json::Value = serde_json::from_slice(&data)?;
        let html = wiki["parse"]["text"]
            .as_str()
            .context("Missing OSRS Wiki boss data")?;
        let bosses = parse_bosses(html)?;
        let response = client
            .get(SKILLS_URL)
            .send()
            .context("Could not fetch skills from Jagex")?;
        let data = crate::net::read_response(response, MAX_DATA_SIZE)?;
        let skills = parse_skills(std::str::from_utf8(&data)?)?;
        let catalog = Self { bosses, skills };
        catalog.validate()?;
        Ok(catalog)
    }
}

pub fn load(refresh: bool, offline: bool) -> Result<Catalog> {
    load_from(cache_path().as_deref(), refresh, offline, Catalog::fetch)
}

fn load_from(
    path: Option<&Path>,
    refresh: bool,
    offline: bool,
    fetch: impl FnOnce() -> Result<Catalog>,
) -> Result<Catalog> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let cached = path.and_then(read_cache);
    if let Some(cache) = &cached {
        if offline || (!refresh && is_fresh(cache.fetched_at, now)) {
            return Ok(cached.context("Missing cache")?.catalog);
        }
    }
    if !offline {
        match fetch() {
            Ok(catalog) => {
                let cache = Cache {
                    fetched_at: now,
                    catalog,
                };
                if let Some(path) = path {
                    if let Err(error) = write_cache(path, &cache) {
                        eprintln!("Could not cache catalogue: {error}");
                    }
                }
                return Ok(cache.catalog);
            }
            Err(error) if refresh => return Err(error),
            Err(error) => eprintln!("Could not refresh catalogue: {error}"),
        }
    }
    if let Some(cache) = cached {
        eprintln!("Using cached boss and skill data.");
        return Ok(cache.catalog);
    }
    eprintln!("Using bundled boss and skill data; it may be out of date.");
    let catalog: Catalog = serde_json::from_str(include_str!("../data/catalog.json"))?;
    catalog.validate()?;
    Ok(catalog)
}

fn cache_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "osrs-random").map(|dirs| dirs.cache_dir().join("catalog-v1.json"))
}

fn is_fresh(fetched_at: u64, now: u64) -> bool {
    now.checked_sub(fetched_at)
        .is_some_and(|age| age < CACHE_AGE)
}

fn read_cache(path: &Path) -> Option<Cache> {
    if fs::metadata(path).ok()?.len() > MAX_DATA_SIZE {
        return None;
    }
    let cache: Cache = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    cache.catalog.validate().ok()?;
    Some(cache)
}

fn write_cache(path: &Path, cache: &Cache) -> Result<()> {
    let parent = path.parent().context("Invalid cache path")?;
    fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(&mut file, cache)?;
    file.flush()?;
    file.persist(path)?;
    Ok(())
}

fn selector(css: &str) -> Result<Selector> {
    Selector::parse(css).map_err(|error| anyhow::anyhow!("Invalid selector: {error}"))
}

fn text(element: ElementRef<'_>) -> String {
    element
        .text()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_skills(html: &str) -> Result<Vec<String>> {
    let document = Html::parse_document(html);
    let links = selector("#contentCategory a[href]")?;
    let base = reqwest::Url::parse(SKILLS_URL)?;
    let mut skills = BTreeSet::new();
    for link in document.select(&links) {
        let url = base.join(link.value().attr("href").unwrap_or_default())?;
        let params: BTreeMap<_, _> = url.query_pairs().collect();
        let skill = params
            .get("table")
            .is_some_and(|id| id.parse::<u32>().is_ok_and(|id| id > 0));
        let activity = params.get("category_type").is_some_and(|kind| kind != "0");
        if skill && !activity {
            skills.insert(text(link));
        }
    }
    ensure!(skills.len() >= 23, "Could not find Jagex's skill list");
    Ok(skills.into_iter().collect())
}

fn parse_bosses(html: &str) -> Result<BTreeMap<String, Vec<String>>> {
    let document = Html::parse_document(html);
    let navbox = selector("table[data-navbox-name='Bosses']")?;
    let table = document
        .select(&navbox)
        .next()
        .context("Missing boss navigation table")?;
    let mut bosses = BTreeMap::new();
    parse_groups(table, "", &mut bosses)?;
    ensure!(!bosses.is_empty(), "Could not find OSRS Wiki bosses");
    for entries in bosses.values_mut() {
        entries.sort();
        entries.dedup();
    }
    Ok(bosses)
}

fn parse_groups(
    table: ElementRef<'_>,
    parent: &str,
    bosses: &mut BTreeMap<String, Vec<String>>,
) -> Result<()> {
    let rows = selector(":scope > tbody > tr.navbox-group")?;
    let heading = selector(":scope > th.navbox-group-title")?;
    let contents = selector(":scope > td.navbox-list")?;
    let nested = selector(":scope > table")?;
    let links = selector(":scope > ul > li > a[href^='/w/']")?;
    for row in table.select(&rows) {
        let title = row.select(&heading).next().map(text);
        let category = title.as_deref().unwrap_or(parent);
        let Some(cell) = row.select(&contents).next() else {
            continue;
        };
        if parent == "Raids"
            || matches!(
                category,
                "The Barrows Brothers" | "The Dagannoth Kings" | "Moons of Peril"
            )
        {
            bosses
                .entry(parent.to_owned())
                .or_default()
                .push(category.to_owned());
        } else if let Some(child) = cell.select(&nested).next() {
            parse_groups(child, category, bosses)?;
        } else {
            let entries: Vec<_> = cell
                .select(&links)
                // The first direct link names the encounter; later links are variants.
                .filter(|link| {
                    !link
                        .prev_siblings()
                        .filter_map(ElementRef::wrap)
                        .any(|sibling| sibling.value().name() == "a")
                })
                .map(text)
                .collect();
            if !entries.is_empty() {
                bosses
                    .entry(category.to_owned())
                    .or_default()
                    .extend(entries);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled() -> Catalog {
        serde_json::from_str(include_str!("../data/catalog.json")).unwrap()
    }

    #[test]
    fn discovers_new_skills_and_excludes_activities_and_overall() {
        let mut html = String::from("<div id='contentCategory'>");
        for id in 1..=25 {
            html.push_str(&format!("<a href='overall?table={id}'>Skill {id}</a>"));
        }
        html.push_str("<a href='overall?table=0'>Overall</a>");
        html.push_str("<a href='overall?category_type=1&amp;table=26'>Clues</a></div>");
        let skills = parse_skills(&html).unwrap();
        assert_eq!(skills.len(), 25);
        assert!(skills.contains(&"Skill 25".into()));
    }

    #[test]
    fn parses_nested_categories_encounters_and_new_raids() {
        let bosses = parse_bosses(include_str!("../tests/fixtures/bosses.html")).unwrap();
        assert_eq!(bosses["World bosses"], ["New boss", "The Barrows Brothers"]);
        assert_eq!(bosses["God Wars Dungeon"], ["Kree'arra"]);
        assert_eq!(bosses["Raids"], ["New raid"]);
        assert_eq!(bosses["New category"], ["Another boss"]);
        assert!(!bosses.values().flatten().any(|name| name == "Minion"));
    }

    #[test]
    fn rejects_changed_or_incomplete_sources() {
        assert!(parse_bosses("<html>Unavailable</html>").is_err());
        assert!(parse_skills("<html>Unavailable</html>").is_err());
        let mut catalog = bundled();
        catalog.skills.clear();
        assert!(catalog.validate().is_err());
    }

    #[test]
    fn fresh_and_offline_caches_do_not_fetch() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.json");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        write_cache(
            &path,
            &Cache {
                fetched_at: now,
                catalog: bundled(),
            },
        )
        .unwrap();
        load_from(Some(&path), false, false, || panic!("Unexpected fetch")).unwrap();
        load_from(Some(&path), false, true, || panic!("Unexpected fetch")).unwrap();
    }

    #[test]
    fn failed_refresh_preserves_stale_cache() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.json");
        let mut catalog = bundled();
        catalog.skills.push("Future skill".into());
        write_cache(
            &path,
            &Cache {
                fetched_at: 0,
                catalog,
            },
        )
        .unwrap();
        let before = fs::read(&path).unwrap();
        let loaded = load_from(Some(&path), false, false, || anyhow::bail!("Offline")).unwrap();
        assert!(loaded.skills.contains(&"Future skill".into()));
        assert!(load_from(Some(&path), true, false, || anyhow::bail!("Offline")).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    #[test]
    fn corrupt_cache_falls_back_offline_and_refresh_replaces_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.json");
        fs::write(&path, "broken").unwrap();
        load_from(Some(&path), false, true, || panic!("Unexpected fetch")).unwrap();
        load_from(Some(&path), true, false, || Ok(bundled())).unwrap();
        assert!(read_cache(&path).is_some());
    }

    #[test]
    fn cache_expiry_handles_clock_changes() {
        assert!(is_fresh(100, 100));
        assert!(!is_fresh(100, 99));
        assert!(!is_fresh(0, CACHE_AGE));
    }
}
