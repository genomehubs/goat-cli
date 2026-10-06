# Upstream GoaT API issues

Problems in the GoaT API ([genomehubs/genomehubs](https://github.com/genomehubs/genomehubs),
`src/genomehubs-api/src/api/v2`) found while developing `goat-cli`. They can't
be fixed in this repository; each section is written to be filed upstream as
is, and describes what `goat-cli` does in the meantime.

All were reproduced against `https://goat.genomehubs.org/api/v2/` on 2026-10-06.
Note that the API rejects `+` for spaces in `query`; use `%20` as below.

---

## 1. `OR` gives wrong results in `/report`

`/search` and `/count` handle `OR` correctly, but `/report` overcounts, and
fails for trees.

Taking the arc report for Primates and Cetacea (`x` is the numerator count):

| `x` | `x` count |
|---|---|
| [`tax_tree(Primates) AND assembly_span`](https://goat.genomehubs.org/api/v2/report?result=taxon&taxonomy=ncbi&includeEstimates=true&report=arc&rank=species&x=tax_tree%28Primates%29%20AND%20assembly_span) | 854 |
| [`tax_tree(Cetacea) AND assembly_span`](https://goat.genomehubs.org/api/v2/report?result=taxon&taxonomy=ncbi&includeEstimates=true&report=arc&rank=species&x=tax_tree%28Cetacea%29%20AND%20assembly_span) | 203 |
| [`tax_tree(Primates,Cetacea) AND assembly_span`](https://goat.genomehubs.org/api/v2/report?result=taxon&taxonomy=ncbi&includeEstimates=true&report=arc&rank=species&x=tax_tree%28Primates%2CCetacea%29%20AND%20assembly_span) | 1057 (= 854 + 203, correct) |
| [`tax_tree(Primates) AND assembly_span OR tax_tree(Cetacea) AND assembly_span`](https://goat.genomehubs.org/api/v2/report?result=taxon&taxonomy=ncbi&includeEstimates=true&report=arc&rank=species&x=tax_tree%28Primates%29%20AND%20assembly_span%20OR%20tax_tree%28Cetacea%29%20AND%20assembly_span) | **1511** |

The same happens for `histogram` and `scatter` (also 1511), and `sources`
returns 41 sources instead of 5. A `tree` report with `OR`
[returns HTTP 500](https://goat.genomehubs.org/api/v2/report?result=taxon&taxonomy=ncbi&includeEstimates=true&report=tree&treeThreshold=2000&x=tax_rank%28species%29%20AND%20tax_tree%28Primates%29%20OR%20tax_rank%28species%29%20AND%20tax_tree%28Cetacea%29).

**Expected:** `OR` combines branches as it does in `/search`
(`.../search?query=tax_tree(Primates) AND tax_rank(species) OR tax_tree(Cetacea) AND tax_rank(species)`
gives 310 = 261 + 49), or `/report` rejects `OR` with an error.

**Possible cause (not verified):** the report code paths may not go through
the `OR` handling in `generateQuery` (`functions/getResults.js`), so the
second branch's filters are lost.

**`goat-cli`:** `taxon arc -x/-y` reject `OR` with an explanation.

---

## 2. GET `/msearch` (batch download) returns different data from `/search`

The batch download takes `;`-separated queries and returns a single TSV,
which would be ideal for `goat-cli -f` batches, but the results differ from
running each query through `/search` with the same parameters.

**Values dropped.** `genome_size_kmer` for *Homo sapiens*:

- [`/search`](https://goat.genomehubs.org/api/v2/search?query=tax_name%28Homo%20sapiens%29&result=taxon&taxonomy=ncbi&fields=genome_size%2Cgenome_size_kmer):
  `"9606"  "species"  "Homo sapiens"  3423000000  3056000000`
- [`/msearch`](https://goat.genomehubs.org/api/v2/msearch?query=tax_name%28Homo%20sapiens%29%3Btax_name%28Pan%20troglodytes%29&result=taxon&taxonomy=ncbi&fields=genome_size%2Cgenome_size_kmer):
  `"9606"  "species"  "Homo sapiens"  3423000000  ` (empty)

**Different rows.** `tax_tree(Primates) AND tax_rank(species)` with
`fields=assembly_level`:
[`/search`](https://goat.genomehubs.org/api/v2/search?query=tax_tree%28Primates%29%20AND%20tax_rank%28species%29&result=taxon&taxonomy=ncbi&size=500&fields=assembly_level)
returns 248 rows,
[`/msearch`](https://goat.genomehubs.org/api/v2/msearch?query=tax_tree%28Primates%29%20AND%20tax_rank%28species%29&result=taxon&taxonomy=ncbi&limit=500&fields=assembly_level)
261. With the parameters `goat-cli` sends (`includeEstimates`,
`excludeAncestral[]`/`excludeMissing[]`, `includeRawValues`, `tidyData`,
`ranks`, `names`), the differences are larger: e.g. 16 rows instead of 307
for Primates and Cetacea with `assembly_level,assembly_span` and
`genome_size*` fields, and raw/tidy output that doesn't match.

**Also:** a query that fails is skipped silently (only logged on the
server), so its rows are just missing from the output.

**Possible cause:** `getMsearchDownload` (`routes/msearch.js`) builds the
Elasticsearch body itself from what `getResults` returns, copying `size`,
`from`, `query`, `_source` and `aggs` but not `sort`, and passes only some
of the request parameters to `getResults`. Not verified which of these
causes each difference.

**`goat-cli`:** sends one `/search` request per taxon (at most 8 at once).

---

## 3. POST `/msearch` totals don't match `/count`

For `tax_tree` queries the `total` in the POST `/msearch` response differs
from `/count`, and is capped at 10,000:

| query | `/count` | POST `/msearch` `total` |
|---|---|---|
| `tax_tree(Primates)` | 283 | 1308 |
| `tax_tree(Cetacea)` | 52 | 328 |
| `tax_tree(Mammalia)` | 2032 | 10000 |
| `tax_tree(Aves)` | 2886 | 10000 |

`tax_name(...)` queries agree. Reproduce with:

```sh
curl -s -X POST https://goat.genomehubs.org/api/v2/msearch \
  -H 'Content-Type: application/json' \
  -d '{"searches":[{"query":"tax_tree(Primates)","result":"taxon","taxonomy":"ncbi","limit":1}]}'
```

**Possible cause (not verified):** the 10,000 cap suggests `hits.total` is
read without `track_total_hits`; the other differences suggest `/count`
applies filtering that the msearch query doesn't.

**`goat-cli`:** uses `/count`.

---

## 4. New `first_*`/`latest_*` taxon fields can't be filtered, and break `/search`

Fields added recently (e.g. `first_assembly_date`, `latest_assembly_date`,
`first_ebp_standard_date`) have `type: date` but no `summary` in
`/resultFields`.

- Presence works:
  [`... AND first_assembly_date`](https://goat.genomehubs.org/api/v2/count?query=tax_tree%28Mammalia%29%20AND%20tax_rank%28species%29%20AND%20first_assembly_date&result=taxon&taxonomy=ncbi&includeEstimates=true)
  counts 1095.
- Any comparison matches nothing:
  [`... AND first_assembly_date >= 2010`](https://goat.genomehubs.org/api/v2/count?query=tax_tree%28Mammalia%29%20AND%20tax_rank%28species%29%20AND%20first_assembly_date%20%3E%3D%202010&result=taxon&taxonomy=ncbi&includeEstimates=true)
  counts 0.
- Requesting one together with another date field
  [returns HTTP 500](https://goat.genomehubs.org/api/v2/search?query=tax_tree%28Hominidae%29&result=taxon&taxonomy=ncbi&includeEstimates=true&size=4&fields=first_assembly_date%2Cassembly_date)
  (`fields=first_assembly_date,assembly_date`), although
  [`fields=first_assembly_date`](https://goat.genomehubs.org/api/v2/search?query=tax_tree%28Hominidae%29&result=taxon&taxonomy=ncbi&includeEstimates=true&size=4&fields=first_assembly_date)
  alone returns 200.

**Possible cause (not verified):** the missing `summary` metadata.

**`goat-cli`:** accepts the fields (they're in its field data), but can't
work around the server behaviour.

---

## Not bugs

Things that looked like API problems during testing, but aren't:

- **`tax_tree(A,B)` with several taxa works**, giving exactly A + B in both
  `/search` and `/report`.
- **Higher ranks return nothing without estimates.** Genera, classes, etc.
  usually have no directly measured values, so e.g.
  `tax_tree(Mammalia) AND tax_rank(genus)` returns 0 unless
  `includeEstimates=true`. The same applies to `tax_depth(n)`, which works
  with estimates.
