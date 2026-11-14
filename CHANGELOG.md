# Changelog

## 0.3.0 (2026-11-14)

The first release since 0.2.53 (February 2025), and a large one: the CLI has been rebuilt on clap's derive API, the expression engine rewritten to match what the GoaT API accepts, and the report commands now print tables. Binaries no longer need OpenSSL.

### Changes that may affect scripts

- **Reports print tables.** `hist`, `scatter`, `arc` and `sources` used to print GoaT's raw JSON; they now print TSV tables (see below). Use `-F json` for the full response.
- **`taxon arc -t` proportions are different, and now right.** The reference population (`-y`) was not limited to `--taxon`, so proportions were divided by the whole tree of life. For example, chromosome-level species in Magnoliopsida were 1996/29192 and are now 1996/3673.
- **Failures exit with a non-zero status**, including GoaT API errors that GoaT returns with HTTP 200.
- **Lineage counts are correct.** `taxon count -l` printed the size of the whole tree (~2.5 million) and `search -l` warned about it, because GoaT's `/count` ignores `tax_lineage()`. They now report the real number, e.g. 30 for *Homo sapiens*.
- **`assembly search --lineage` is a flag.** It used to require a value, and then panicked.
- **Spaces in URLs are `%20`, not `+`,** in the URLs printed by `-u`/`-U`.
- **`--help` is reordered:** query options first, then field flags, then output options.
- **Fields:** `sequencing_status_hk-ebp` and `sequencing_status_phyloalps` are gone, following GoaT.

### Added

- **Output formats:** `-F, --format tsv|csv|json` on `search`, `count`, `lookup`, `record`, `sources`, `hist`, `scatter` and `arc`. With several taxa, `search -F json` gives one response with all their results.
- **Report tables:**
  - `hist`: a row per bin (`<field>_from`, `<field>_to`, `count`), or per value for keyword fields, with a column per `-c` category;
  - `scatter`: a row per x/y cell;
  - `arc`: both counts and the proportion;
  - `sources`: a row per data source.
- **New report commands:** `taxon arc` (`--x-filter`, `--y-filter`, `--exclude-missing`, `--exclude-ancestral`, and global arcs without `-t`), plus full options for `hist`, `scatter` and `sources` (`-c`, `-f`, `-n`, `--x-opts`/`--y-opts`).
- **Records:** `taxon record` and `assembly record` show every field of a record, with its value, count, aggregation and sources. `-n` shows names (taxon) or identifiers (assembly) instead, and `-l` the lineage. Taxon names, including common names, are resolved to IDs.
- **Sorting:** `search --sort-by <field> --sort-order asc|desc`. The field is checked, and typos get a suggestion, because GoaT silently ignores unknown fields.
- **Depth:** `--tax-depth <n>` (with `-d`) gives only taxa exactly `n` levels below each taxon.
- **Shell completions:** `goat-cli completions bash|zsh|fish|elvish|powershell`.
- **The full GoaT expression syntax in `-e`,** documented in [EXPRESSIONS.md](EXPRESSIONS.md):
  - `OR`, with optional parentheses around each branch;
  - searching without a taxon, when `-e` is given;
  - case-insensitive `and`/`or`, field names and keyword values;
  - scientific notation and negative numbers;
  - dates as `yyyy`, `yyyy-mm` or `yyyy-mm-dd`;
  - `null`;
  - `field:subset` modifiers;
  - the functions `count`, `sp_count` and `range`;
  - identifier terms such as `taxon_id = 9606,9598`;
  - field synonyms and hyphenated names;
  - clearer errors for `&&`, `||`, `contains`, nesting and unknown functions;
  - no 100-character limit.
- **New fields:** `-v`, the report `-x`/`-y` and `-e` accept any field GoaT has, not only the ones built into goat-cli. If a name isn't built in, goat-cli fetches GoaT's field list and caches it on disk for a day.
- **Field data refreshed from GoaT:** 19 new taxon fields (e.g. `first_assembly_date`, `redlist_category`, `total_assemblies`), 4 new assembly fields, and `assembly search --gc-percent`.
- **Input files:** `-f` files may have blank lines and `#` comments. Report commands read taxa from `-f`; before, they ignored the file.
- **Warnings:** a search with no results now gives one.
- **Upstream issues:** [ISSUES.md](ISSUES.md) lists problems in the GoaT API found along the way, with URLs to reproduce them and what goat-cli does in the meantime.

### Fixed

- **`hist -c <rank>`** headed its columns with taxon IDs. They are now names (`Draba`, `Cardamine`, ...).
- **`--x-opts`/`--y-opts`** rejected `1e8` and overflowed above ~2.1e9, so genome-size ranges couldn't be set. Min and max now take any finite number, and the scale may be left blank before an axis title (`",,5,,Title"`).
- **`scatter` keyword axes** (e.g. `assembly_level`) are a single column of values.
- **`newick -n`** was accepted but had no effect; only the given taxa are now tips. `newick --threshold -1` is accepted, as the help said.
- **`lookup`:** a name with no hits no longer aborts the batch, the header prints once, and results keep the input order.
- **`record`:** names that match no taxon, or several, are reported and skipped, so the rest still print.
- **Crashes and hangs:**
  - several argument combinations panicked;
  - `goat-cli ... | head` panicked on the broken pipe;
  - `-t ","` hung forever, and is now an error;
  - a report with more counts than buckets panicked.
- **Progress bar:** it started at a made-up `0/512` and ended on `0/0`. It now shows only real totals and finishes at total/total. It also polls less often, and with `-u` it no longer prints the URLs twice.
- **Dates:** validation of year-only and full-date inputs is fixed.
- **Errors:** GoaT API errors are reported once and clearly, rather than wrapped in a "format TSV error".

### Performance

- **Concurrency cap:** at most 8 requests are in flight at a time. A 500-line `-f` file used to open 500 connections at once.
- **Count:** `search` fetches the count it needs for warnings alongside the results, saving a round trip (~0.44 s to ~0.27 s for one taxon).
- **gzip:** responses are requested gzipped, about 5.6× smaller for large TSVs.
- **Retries:** failed requests are retried on server errors (5xx) as well as connection errors, with a connect timeout.

### Packaging

- **No OpenSSL:** TLS is now rustls, with bundled root certificates. The 0.2.53 Linux binary failed on current distributions for want of `libssl.so.1.1`.
- **Release builds:** the release workflow now uses an updated cargo-dist, on supported GitHub runners.
- **Crate:** the crate no longer includes the `docs/` folder.
- **Docs:** the API documentation at https://genomehubs.github.io/goat-cli/ is rebuilt (it was from 0.2.5), and the [wiki](https://github.com/genomehubs/goat-cli/wiki) is rewritten for this release.

### Known issues in GoaT

These are in the GoaT API, not goat-cli. See [ISSUES.md](ISSUES.md).

- `--toggle-direct` adds no columns, because GoaT ignores `:direct`/`:ancestor`/`:descendant` in fields.
- `mitochondrion_assembly_span` and `plastid_assembly_span` return no values, and numeric filters on them give HTTP 500.
- `/lookup` finds nothing for some well-known names, e.g. *Arabidopsis thaliana*. Use the taxon ID or `search`.
- `OR` is rejected in `arc` filters, as GoaT's report endpoint miscounts it.

**Full diff:** https://github.com/genomehubs/goat-cli/compare/v0.2.53...v0.3.0
