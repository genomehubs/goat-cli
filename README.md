# GoaT CLI

A command line interface for GoaT (Genomes on a Tree). GoaT presents metadata for taxa across the tree of life. 

The CLI here builds URLs to query the <b><a href="https://goat.genomehubs.org/api-docs/">Goat API</a></b>. The underlying API is very powerful, but complex. The aim of this CLI is to remove some of the complexity for the end user. TSV's are returned.

This CLI is actively being built so no guarantees are made for stability or bugs. An official release will be made soon.

## Download

### Download from releases

E.g. get from the releases page.

```bash
# for mac
curl -L "https://github.com/genomehubs/goat-cli/releases/download/0.1.1/goat_mac_0.1.1" > goat && chmod +x goat
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

Currently, all functionality is through the `goat search` subcommand. `goat search --help` will bring up the help below:

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
    -d, --date                 Print EBP & assembly dates.
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
    -p, --phylogeny            Get information for all descendents of a common ancestor.
    -P, --plastid              Print plastid genome size & GC%.
        --ploidy               Print ploidy estimates.
    -r, --raw                  Print raw values (i.e. no aggregation/summary).
    -S, --sex-determination    Print sex determination data.
        --target-lists         Print target list data associated with each taxon.
    -T, --tidy                 Print data in tidy format.
    -u, --url                  Print the underlying GoaT API URL(s). Useful for debugging.
    -V, --version              Prints version information

OPTIONS:
    -f, --file <file>      A file of NCBI taxonomy ID's (tips) and/or binomial names.
                           Each line should contain a single entry.
        --ranks <ranks>    Choose a rank to display with the results. All ranks up to the given rank are displayed.
                           [default: none]  [possible values: none, subspecies, species, genus, family, order, class,
                           phylum, kingdom, superkingdom]
    -s, --size <size>      The number of results to return. Max 10,000 currently. [default: 50]
    -t, --taxon <taxon>    The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank.
```

### Flags

There are a series of flags that can be added in any combination. For example:

`goat search -abct "Arabidopsis thaliana"` (<- Note quotes needed when there are spaces in the query)

Will return a TSV for assembly statistics (`-a`, type & span), BUSCO completeness metrics (`-b`), and C-values (`-c`) for the taxon <i>Arabidopsis thaliana</i>. Add `--tidy` to format the results in tidy format (i.e. one observation per row).

Two of the flags are different. `-p` (`--phylogeny`) will return a tax tree search. This means that a node on the tree and all of its descendents will be included in the query. For example:

`goat search -pat Poaceae` 

Will return aggregated results for every species in the family <i>Poaceae</i>. If the default number of return hits is lower than the possible number of results, there will be a warning (in this case):

`[-]     For search query Poaceae, size specified (50) was less than the number of results returned, (1116).`

So up the `--size` option to `1116`.

You may not want aggregated results, instead requiring raw values. This is where the `-r --raw` flag is handy. Adding this flag will return all results for a taxon. For example:

`goat search -part Poaceae --size 1116` 

Will display each direct result in turn for the assembly statistics implemented so far. Note adding `raw` coerces results into tidy format.

### Options

As for the options, we have already seen `size`. Next up is `--ranks`. Adding one of `subspecies, species, genus, family, order, class, phylum, kingdom, superkingdom` to this option will prepend a tab separated taxonomy for each record. Use it like this:

`goat search -pat Arabidopsis --ranks genus` 

Another way of searching for multiple species (related or not) is to use `-t` with quotes around a comma separated list. Like this:

`goat search -rat "Arabidopsis thaliana, Zea mays, Solanum tuberosum"`

This search will return all the assembly spans and levels for the three taxa specified.

Large numbers of requests are probably best submitted using a file, e.g:

`goat search -kf examples/taxids.txt > taxids_karyotype_data.tsv`

### Feedback, requests, etc

Please test `goat`. Please find bugs and tell me. Please send feature requests. Creating an issue in this repo is probably the best way to go!