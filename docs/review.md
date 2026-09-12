# Project review — 12 September 2026

Reviewed all tracked source, dependencies, release automation, README and issue
templates at `245cc48`, then implemented the requested live data and self-updater.

## Findings addressed

| Severity | Finding | Resolution |
| --- | --- | --- |
| High | Every timestamped GitHub release was treated as up to date, so the existing update check could never detect a release. String inequality also classified older semantic versions as updates. | Embed the exact release tag in both binaries; compare ordered timestamps or semantic versions and reject incompatible version formats. |
| Medium | Bosses and skills were compiled into the executable and had already fallen behind the live game. | Fetch boss categories and encounters from the OSRS Wiki and skills from Jagex; validate and cache data for 24 hours, with an explicit refresh and offline fallback. |
| Medium | CLI commands prompted or paused for input; EOF could leave the interactive loop running indefinitely. | Keep prompts in terminal sessions, return help when launched without a terminal, handle EOF and propagate input errors. |
| Medium | Network requests had no application timeouts; release downloads had no existing validation or installation path. | Add connect/request timeouts, HTTP status and size checks, exact platform asset selection, SHA-256 verification and platform-aware executable replacement. |
| Medium | Releases ignored `Cargo.lock`, ran no tests and did not associate a version identity with the built executable. | Track the lockfile, test both supported platforms, check the embedded tag and package both binaries before publishing a completed release for the exact commit. |
| Low | HashMap iteration changed category numbering between runs. | Sort categories and entries consistently. |
| Low | Screen clearing spawned external commands and several input/output paths used unchecked unwraps. | Use crossterm directly and handle interactive input/terminal errors. |
| Low | Dependency versions and action versions were old; the referenced MIT licence file was absent. | Update dependencies/actions, remove unused dependencies, add the declared licence and document data attribution. |

## Validation

- Unit tests cover parsing unseen entries and categories, grouped encounters,
  cache freshness/corruption/offline fallback, failed refresh preservation,
  exclusions, version ordering, target selection, digest verification and ZIP
  extraction.
- Live refresh retrieved 64 boss encounters across 10 categories and 24 skills,
  including Sailing; the validated result is the bundled offline snapshot.
- A disposable Linux executable completed the actual GitHub download,
  verification and self-replacement flow and ran successfully afterwards.
- Formatting, strict Clippy and the dependency audit pass. No vulnerable or
  unmaintained dependencies were reported by the audit.
- Release automation tests and builds both Linux GNU x86-64 and Windows MSVC
  x86-64; the local end-to-end replacement check covers Linux.

## Remaining constraints

- New content becomes available when the Wiki or Jagex lists it and the cache
  refreshes. Their HTML structure is an external dependency; failures retain
  valid cached data and produce a visible fallback message.
- Categories follow the Wiki. Variant links stay grouped under the first
  encounter name; Barrows, Dagannoth Kings, Moons of Peril and raids remain grouped.
  Random selection still weights each category equally, as the original did.
- The self-updater requires an official build, an available SHA-256 digest and a
  writable installation directory. Existing installations need one manual
  download. Other CPU architectures and ABIs require source builds.
- A GitHub asset digest verifies a download against the release metadata; it is
  not an independently signed release manifest.
