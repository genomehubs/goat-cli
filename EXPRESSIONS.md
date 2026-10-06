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
  friends: `assembly_level >= scaffold`.
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

## Known API limitations

These appear in the API source but don't work, so `goat-cli` does not offer
them:

- `tax_depth(n)` returns no results.
- `tax_tree(A,B)` with several taxa gives inconsistent results (0 hits from
  `/search`; fewer than the equivalent `OR` from `/report`). `goat-cli`
  sends one query per taxon for `search` and `count`. The report commands
  (`hist`, `scatter`, `arc`, `sources`, `newick`) still join several taxa
  into one `tax_tree(...)`, so prefer a single taxon there.
- `collate(...)`, `variable.metadata = ...` paths and newline-separated batch
  queries are accepted but don't return meaningful results.
