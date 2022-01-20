# GoaT CLI

A command line interface for GoaT (Genomes on a Tree). GoaT presents metadata for taxa across the tree of life. 

The CLI here builds URLs to query the <b><a href="https://goat.genomehubs.org/api-docs/">Goat API</a></b>. The underlying API is very powerful, but complex. The aim of this CLI is to remove some of the complexity for the end user. Only TSV files are returned. 

This CLI is actively being built so no guarantees are made for stability or bugs.

## Download

### Download from releases

E.g. get from the releases page.

```bash
# for mac
curl -L "https://github.com/genomehubs/goat-cli/releases/download/0.1.3/goat_mac_0.1.3" > goat && chmod +x goat
# and linux (ubuntu)
curl -L "https://github.com/genomehubs/goat-cli/releases/download/0.1.3/goat_ubuntu_0.1.3" > goat && chmod +x goat
```

### Build from source

First download Rust if you haven't already.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then clone this repo, and build.

```bash
git clone https://github.com/genomehubs/goat-cli
cd goat-cli && cargo build --release
# executable is in ./target/release/goat
./target/release/goat search --help
```

## Usage

### Overall

There are three implemented commands:

- `goat search` is the main program, returning metadata for any species.
- `goat lookup` gives you a **full** list of synonyms, authorities, and Tree of Life identifier where applicable. Also includes spell-check which is neat. Used internally in `goat search`.
- `goat count` exposes the <a href="https://goat.genomehubs.org/api-docs/#/GoaT%20API/getSearchResultCount">count API</a>. It tells you how many results you would get for any given `goat search` query. It's used internally in `goat search` but presented here for interest.

```
goat 0.1.3
Max Brown <mb39@sanger.ac.uk>
GoaTs on a terminal. Combine flags to query metadata for any species.

USAGE:
    goat [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    count     Query the GoaT count API. Return the number of hits from any search.
    help      Prints this message or the help of the given subcommand(s)
    lookup    Query the GoaT lookup API.
    newick    Query the GoaT record API, and return a newick.
    search    Query the GoaT search API.
```

### Search

`goat help search` or `goat search --help` will bring up the help below:

```
goat-search 
Query the GoaT search API.

USAGE:
    goat search [FLAGS] [OPTIONS] --file <file> --taxon <taxon>

FLAGS:
    -A, --all                  Print all currently implemented GoaT variables.
    -a, --assembly             Print assembly data (span & level)
    -B, --bioproject           Print the bioproject and biosample ID of records.
    -b, --busco                Print BUSCO estimates.
    -c, --c-values             Print c-value data.
    -C, --country-list         Print list of countries where taxon is found.
    -D, --date                 Print EBP & assembly dates.
    -d, --descendents          Get information for all descendents of a common ancestor.
    -G, --gene-count           Print gene count data.
    -g, --genome-size          Print genome size data.
    -h, --help                 Prints help information
    -i, --include-estimates    Include ancestral estimates. Omitting this flag includes only direct estimates from a
                               taxon. Cannot be used with --raw.
    -k, --karyotype            Print karyotype data (chromosome number & haploid number).
    -l, --legislation          Print legislation data.
    -m, --mitochondria         Print mitochondrial genome size & GC%.
    -N, --n50                  Print the contig & scaffold n50 of assemblies.
    -n, --names                Print all associated name data (synonyms, Tree of Life ID, and common names).
    -p, --plastid              Print plastid genome size & GC%.
    -P, --ploidy               Print ploidy estimates.
    -r, --raw                  Print raw values (i.e. no aggregation/summary).
    -S, --sex-determination    Print sex determination data.
        --status               Print all data associated with how far this taxon has progressed with genomic sequencing.
                               This includes sample collection, acquisition, progress in sequencing, and whether
                               submitted to INSDC.
    -t, --target-lists         Print target list data associated with each taxon.
    -T, --tidy                 Print data in tidy format.
    -u, --url                  Print the underlying GoaT API URL(s). Useful for debugging.
    -V, --version              Prints version information

OPTIONS:
    -f, --file <file>      A file of NCBI taxonomy ID's (tips) and/or binomial names.
                           Each line should contain a single entry.
    -R, --ranks <ranks>    Choose a rank to display with the results. All ranks up to the given rank are displayed.
                           [default: none]  [possible values: none, subspecies, species, genus, family, order, class,
                           phylum, kingdom, superkingdom]
    -s, --size <size>      The number of results to return. Max 10,000 currently. [default: 50]
    -t, --taxon <taxon>    The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank.
```

#### Flags

There are a series of flags that can be added in any combination. For example:

`goat search -abct "Arabidopsis thaliana"` (<- Note quotes needed when there are spaces in the query)

Will return a TSV for assembly statistics (`-a`, type & span), BUSCO completeness metrics (`-b`), and C-values (`-c`) for the taxon <i>Arabidopsis thaliana</i>. Add `--tidy` to format the results in tidy format (i.e. one observation per row).

Two of the flags are different. `-d` (`--descendents`) will return a tax tree search. This means that a node on the tree and all of its descendents will be included in the query. For example:

`goat search -dat Poaceae` 

Will return aggregated results for every species in the family <i>Poaceae</i>. If the default number of return hits is lower than the possible number of results, there will be a warning (in this case):

`[-]     For search query Poaceae, size specified (50) was less than the number of results returned, (800).`

So up the `--size` option to `800`. To find all (direct) chromsomal level assembly spans for grasses:

`goat search -dat Poaceae --size 800 | grep "Chromosome"`

You may not want aggregated results, instead requiring raw values. This is where the `-r --raw` flag is handy. Adding this flag will return all results for a taxon. For example:

`goat search -dart Poaceae --size 800` 

Will display each direct result in turn for the assembly statistics implemented so far. Note adding `raw` coerces results into tidy format.

#### Options

As for the options, we have already seen `size`. Next up is `-R, --ranks`. Adding one of `subspecies, species, genus, family, order, class, phylum, kingdom, superkingdom` to this option will prepend a tab separated taxonomy for each record. Use it like this:

`goat search -pat Arabidopsis --ranks genus` 

Another way of searching for multiple species (related or not) is to use `-t` with quotes around a comma separated list. Like this:

`goat search -rat "Arabidopsis thaliana, Zea mays, Solanum tuberosum"`

This search will return all the assembly spans and levels for the three taxa specified.

Larger numbers of requests are probably best submitted using a file (only up to 500 spp), e.g:

`goat search -kf examples/taxids.txt > taxids_karyotype_data.tsv`

*`goat count` presents an identical CLI to `goat search`.*

### Lookup

`goat lookup` is very simple currently. It only requires a taxon NCBI ID, taxon name, or common name. It returns an NCBI tax-id, along with synonyms, authorities, and common names.

```
goat-lookup 
Query the GoaT lookup API.

USAGE:
    goat lookup [FLAGS] --taxon <taxon>

FLAGS:
    -h, --help       Prints help information
    -u, --url        Print lookup URL.
    -V, --version    Prints version information

OPTIONS:
    -t, --taxon <taxon>    The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank.
```

Examples:

Spell-check if no results found
- `goat lookup -t Pterophorsa` -> `Did you mean: Pterophorus, Petrophora, Pterophora?`

Query multiple taxon ID's
- `goat lookup -t "38942, 2405"`

Get authorities only
- `goat lookup -t "Quercus robur" | grep "authority"`

### Newick

GoaT can also return trees. Given a clade, or a string of clades, a newick tree will be returned. It's actually a cladogram as there are no branch lengths, but this may change with future GoaT versions.

```
goat-newick 
Query the GoaT record API, and return a newick.

USAGE:
    goat newick [FLAGS] [OPTIONS] --taxon <taxon>

FLAGS:
    -h, --help       Prints help information
    -u, --url        Print lookup URL.
    -V, --version    Prints version information

OPTIONS:
    -r, --rank <rank>      The number of results to return. [default: species]  [possible values: species, genus,
                           family, order]
    -t, --taxon <taxon>    The taxon to return a newick of. Multiple taxa will return the joint tree.
```

E.g. get a cladogram of the genera in the family Fabaceae:

`goat newick -t "Fabaceae" -r genus`

### Feedback, requests, notes etc

This README will always be up to date with the latest *working* version. The released versions may lag behind.

Please test `goat`. Please try and break `goat`. Please find bugs and tell me. Please send feature requests. Creating an issue in this repo is probably the best way to go!
