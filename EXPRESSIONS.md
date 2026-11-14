# Expressions (`-e`)

`goat-cli taxon search`, `taxon count`, `assembly search` and `assembly count`
take an expression with `-e/--expression` to filter results on the server.
This page describes the syntax the GoaT API accepts, as implemented in
[genomehubs-api](https://github.com/genomehubs/genomehubs/tree/main/src/genomehubs-api/src/api/v2)
(`functions/getResults.js`) and checked against the live API. Much of it is
not in the [official API docs](https://goat.genomehubs.org/api-docs/).

`goat-cli` checks expressions before sending them, so typos get a "did you
mean" suggestion instead of an empty result.

```sh
goat-cli taxon search -t Mammalia -d -e 'genome_size > 1e9 AND assembly_level = chromosome,complete genome'
```

## Clauses

An expression is one or more clauses joined by `AND` / `OR`. A clause is

```
<variable> <operator> <value>
```

or a bare `<variable>`, which matches records that have a value for it.

- **Variables** are listed by `--print-expression`. Names are
  case-insensitive, `-` may be used for `_` (`genome-size`), and synonyms are
  accepted (e.g. `ebp_metric_date` for `ebp_standard_date`).
- **Operators** are `=`, `==`, `!=`, `<`, `<=`, `>`, `>=`. Spaces around them
  are optional.
- **Values** are case-insensitive. Quotes are optional, even for values
  containing spaces.

## Combining clauses

| Syntax | Meaning |
|---|---|
| `a AND b` | both (`and` works too) |
| `a OR b` | either (`or` works too) |
| `a AND b OR c AND d` | `(a AND b) OR (c AND d)`: `OR` binds more loosely than `AND` |
| `(a AND b) OR (c AND d)` | the same; parentheses may wrap a whole `OR` branch |

Parentheses cannot be nested or used inside a branch, because GoaT splits
on `OR` before anything else. Write `a AND (b OR c)` as `a AND b OR a AND c`.

The taxon (`-t`/`-f`, `-d`, `-l`) and `--tax-rank` apply to every `OR`
branch, so

```sh
goat-cli taxon count -t Mammalia -d --tax-rank species -e 'assembly_level = chromosome OR genome_size > 5e9'
```

counts mammal species that have a chromosome-level assembly *or* a genome
size over 5 Gb.

`&&`, `||` and `contains` are not supported.

## Values

| Variable type | Accepted values | Examples |
|---|---|---|
| numeric | any number, including negatives and scientific notation; size suffixes such as `1G` are **not** accepted | `genome_size > 1e9`, `c_value > -1` |
| date | `yyyy`, `yyyy-mm` or `yyyy-mm-dd` | `assembly_date >= 2023-06` |
| keyword | text; for some variables, one of a fixed list | `bioproject = PRJNA533106` |

- **Lists:** a comma separated list matches any of its values:
  `assembly_level = chromosome,complete genome`.
- **Negation:** prefix a value with `!` to exclude it:
  `assembly_level = chromosome,!scaffold`, `bioproject = !PRJEB40665`.
  `!=` also works: `assembly_level != scaffold`.
- **Missing values:** `null` matches missing values and `!null` present ones,
  for any type: `genome_size != null`.
- **Ordered keywords:** keywords with a natural order can use `<`, `>` and
  friends: `assembly_level >= scaffold`. The `sequencing_status*` fields are
  ordered along the pipeline: `sample_available` < `sample_collected` <
  `sample_acquired` < `in_progress` < `data_generation` < `in_assembly` <
  `open` < `insdc_open` < `published`. A keyword field takes only one
  comparison per expression (see [Known API limitations](#known-api-limitations));
  for a span of stages, list them.
- **Fixed lists:** some keywords only take values from a fixed list (e.g.
  `assembly_level`, `sequencing_status*`). `goat-cli` rejects anything else.
  Other keywords list their known values in `--print-expression` (e.g.
  `long_list`, `country_list`) but accept any value; `goat-cli` warns if a
  value is not a known one, because the query will probably match nothing.

## Functions and subsets

A variable can be wrapped in a summary function, or followed by a subset,
or both:

| Syntax | Meaning |
|---|---|
| `min(x)`, `max(x)`, `range(x)`, `value(x)` | compare a summary of `x` |
| `count(x)`, `length(x)`, `sp_count(x)` | compare a count (always a number), e.g. `length(long_list) > 1` |
| `x:direct`, `x:ancestor`, `x:descendant`, `x:estimate` | only values from that source, e.g. `genome_size:direct > 1e9` |

Not every function applies to every variable; GoaT reports an error if
one doesn't. `mean`, `median` and `mode` are not supported in expressions.

## Identifiers

Identifier fields ending in `_id` can be matched directly, including in lists:

```sh
goat-cli taxon search -e 'taxon_id = 9606,9598'
goat-cli assembly search -e 'assembly_id = GCA_000001405.29'
```

## Searching without a taxon

If `-e` is given, `-t`/`-f` can be omitted to search across all taxa:

```sh
goat-cli taxon search --tax-rank species -e 'genome_size > 1e11'
```

## Set elsewhere

Taxon terms are set with flags rather than in the expression:

| Query term | Flag |
|---|---|
| `tax_name(...)` | `-t` / `-f` |
| `tax_tree(...)` | `-t` with `-d` |
| `tax_lineage(...)` | `-t` with `-l` |
| `tax_rank(...)` | `--tax-rank` |

## Report filters

`goat-cli taxon arc` takes filters (`-x/--x-filter`, `-y/--y-filter`) in the
same syntax, and they are checked the same way, except that `OR` is not
allowed: GoaT's report endpoint gives wrong results for it (inflated counts,
or a server error for trees), although `/search` handles it correctly.

## Recipes

Questions that come up when tracking genome projects, and the data behind
them. Each one uses `count`; swap in `search` to list the taxa, adding `-v`
or field flags for the columns you want. All were checked against the live
API.

Project fields use the project's acronym from `--print-expression` (e.g.
`dtol`, `erga-bge`, `vgp`, `asg`). Each project has a
`sequencing_status_<project>` field, and its stage fields (`long_list`,
`sample_collected`, `in_progress`, `insdc_open`, `published`, ...) list the
projects at that stage.

### Projects and samples through the pipeline

1. **Samples a project has collected, but not yet received.**

   ```sh
   goat-cli taxon count --tax-rank species -e 'sequencing_status_dtol = sample_collected'
   ```

2. **Samples received, but sequencing not yet started.**

   ```sh
   goat-cli taxon count --tax-rank species -e 'sequencing_status_dtol = sample_acquired'
   ```

3. **Everything currently in the pipeline**, from sample received to
   assembly. List the stages; a range such as `>= sample_acquired AND <
   insdc_open` doesn't work (see [Known API limitations](#known-api-limitations)).

   ```sh
   goat-cli taxon count --tax-rank species -e 'sequencing_status_dtol = sample_acquired,in_progress,data_generation,in_assembly'
   ```

4. **Everything that has reached at least data generation.** A single
   comparison is fine.

   ```sh
   goat-cli taxon count --tax-rank species -e 'sequencing_status_dtol >= data_generation'
   ```

5. **What a project has made public on INSDC**, and what it has published
   a paper on.

   ```sh
   goat-cli taxon count --tax-rank species -e 'sequencing_status_dtol >= insdc_open'
   goat-cli taxon count --tax-rank species -e 'published = dtol'
   ```

6. **Targets on a project's list that it hasn't started**: on the long
   list, with no status yet.

   ```sh
   goat-cli taxon count --tax-rank species -e 'long_list = dtol AND sequencing_status_dtol = null'
   ```

7. **Species targeted by three or more projects**, where effort may be
   duplicated. To list the taxa, `-v long_list` shows which projects.

   ```sh
   goat-cli taxon count --tax-rank species -e 'length(long_list) >= 3'
   ```

8. **Species that more than one project is sequencing.**

   ```sh
   goat-cli taxon count --tax-rank species -e 'length(in_progress) > 1'
   ```

9. **Samples collected by one partner**, e.g. the Natural History Museum.
   Partner codes are listed under `sample_collected_by` in
   `--print-expression`.

   ```sh
   goat-cli taxon count --tax-rank species -e 'sample_collected_by = nhm'
   ```

10. **A project's progress within a group and a country**: UK insects that
    have been collected but are still waiting to reach the lab.

    ```sh
    goat-cli taxon count -t Insecta -d --tax-rank species -e 'sequencing_status_dtol = sample_collected AND country_list = gb'
    ```

11. **A project's chromosome-level assemblies**, from the assembly index by
    BioProject (PRJEB40665 is DToL's umbrella project), and those updated
    since a date.

    ```sh
    goat-cli assembly count -t Lepidoptera -d -e 'bioproject = PRJEB40665 AND assembly_level = chromosome'
    goat-cli assembly count -e 'bioproject = PRJEB40665 AND last_updated >= 2026-06'
    ```

### Gaps and priorities

12. **Threatened species that no project has taken on**: critically
    endangered or endangered on the IUCN Red List, with no sequencing
    status. With `assembly_level = null` instead, it gives those with no
    assembly from anyone.

    ```sh
    goat-cli taxon count --tax-rank species -e 'redlist_category = cr,en AND sequencing_status = null'
    ```

13. **Threatened species in the pipeline right now.**

    ```sh
    goat-cli taxon count --tax-rank species -e 'redlist_category = cr,en AND sequencing_status = sample_collected,sample_acquired,in_progress,data_generation,in_assembly'
    ```

14. **Species from a country that no project lists**, e.g. Great Britain.
    Country codes are under `country_list` in `--print-expression`.

    ```sh
    goat-cli taxon count --tax-rank species -e 'country_list = gb AND long_list = null'
    ```

15. **Families with no assembled species at all.** Higher ranks need `-i`,
    since their values are summarised from their descendants.

    ```sh
    goat-cli taxon count -t Insecta -d --tax-rank family -i -e 'assembly_level = null'
    ```

16. **Species with a measured chromosome count but no assembly**: targets
    whose assemblies can be checked against a known karyotype.

    ```sh
    goat-cli taxon count -t Lepidoptera -d --tax-rank species -e 'chromosome_number:direct AND assembly_level = null'
    ```

17. **Very large measured genomes (over 10 Gb) that no project has
    started.**

    ```sh
    goat-cli taxon count --tax-rank species -e 'genome_size:direct > 1e10 AND sequencing_status = null'
    ```

### Assembly quality and data

18. **Assemblies worth upgrading**: species whose best assembly is at
    scaffold level, with contig N50 under 100 kb.

    ```sh
    goat-cli taxon count --tax-rank species -e 'assembly_level = scaffold AND contig_n50 < 1e5'
    ```

19. **High-quality assemblies**: chromosome level, with BUSCO completeness
    of at least 95%.

    ```sh
    goat-cli assembly count -e 'assembly_level = chromosome AND busco_completeness >= 95'
    ```

20. **Species assembled for the first time this year**, and those first
    meeting the EBP reference standard this year.

    ```sh
    goat-cli taxon count --tax-rank species -e 'first_assembly_date >= 2026'
    goat-cli taxon count --tax-rank species -e 'first_ebp_standard_date >= 2026'
    ```

21. **Polyploids with a chromosome-level assembly**, from measured ploidy
    only.

    ```sh
    goat-cli taxon count --tax-rank species -e 'ploidy:direct >= 4 AND assembly_level >= chromosome'
    ```

22. **Genome sizes in a range.** Numeric fields can take two comparisons,
    unlike keywords.

    ```sh
    goat-cli taxon count -t Lepidoptera -d --tax-rank species -e 'genome_size >= 1e9 AND genome_size <= 2e9'
    ```

`taxon arc` takes the same filters, without `OR`, to give proportions
rather than counts; see the
[wiki](https://github.com/genomehubs/goat-cli/wiki/5.-Reports-and-trees).

## Known API limitations

- **A keyword field takes only one comparison.** In
  `sequencing_status_dtol >= sample_acquired AND sequencing_status_dtol < insdc_open`,
  GoaT applies only the first comparison, and silently ignores the second.
  Use a list of values instead
  (`sequencing_status_dtol = sample_acquired,in_progress,data_generation,in_assembly`).
  Numeric ranges such as `genome_size >= 1e9 AND genome_size <= 2e9` work.
- Taxa at higher ranks (e.g. genera, or `tax_rank(class)`) usually have no
  directly measured values, only estimates, so without `-i/--include-estimates`
  queries for them return nothing.
- `tax_depth(n)` (descendants exactly `n` levels below the taxon) is exposed
  as `-d --tax-depth <n>`. It takes a single number; ranges such as
  `tax_depth(<=2)` give a server error. Like any query, levels above species
  need `-i` to return anything.
- `collate(...)`, `variable.metadata = ...` paths and newline separated batch
  queries appear in the API source, but did not return meaningful results
  when tested, so `goat-cli` does not offer them.
