# GoaT CLI

A command line interface for GoaT (Genomes on a Tree). GoaT presents metadata for taxa across the tree of life. 

The CLI here builds URLs to query the <b><a href="https://goat.genomehubs.org/api-docs/">Goat API</a></b>. The underlying API is very powerful, but complex. The aim of this CLI is to remove some of the complexity for the end user. TSV's are returned.

This CLI is actively being built so no guarantees are made for stability or bugs. An official release will be made soon.

## Download

### Download from releases

E.g. get from the releases page.

```bash
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
    -z, --all             This flag indicates all data should be printed.
    -a, --assembly        This flag indicates assembly data should be printed.
    -b, --busco           Include BUSCO estimates.
    -c, --c-values        This flag indicates C-value data should be printed.
    -g, --genome-size     This flag indicates genome size data should be printed.
    -h, --help            Prints help information
    -k, --karyotype       This flag indicates karyotype data should be printed.
        --mitochondria    Include mitochondrial genome size & GC%.
    -p, --phylogeny       Get information for all descendents of a common ancestor.
        --plastid         Include plastid genome size & GC%.
    -r, --raw             This flag indicates raw values should be all listed out. So you can do your own aggregations
                          for example.
    -u, --url             Print the underlying GoaT API URL(s). Useful for debugging.
    -V, --version         Prints version information

OPTIONS:
    -f, --file <file>      A file of NCBI taxonomy ID's (tips) and/or binomial names.
                           Each line should contain a single entry.
        --ranks <ranks>    Choose a rank to display with the results. All ranks up to the given rank are displayed.
                           [default: none]  [possible values: none, subspecies, species, genus, family, order, class,
                           phylum, kingdom, superkingdom]
        --size <size>      The number of results to return. [default: 50]
    -t, --taxon <taxon>    The taxon to search. An NCBI taxon ID, or the name of a taxon at any rank.
```

### Flags

There are a series of flags that can be added in any combination. For example:

`goat search -abc -t "Arabidopsis thaliana"` (<- Note quotes needed when there are spaces in the query)

Will return TSV's for assembly statistics (`-a`, type & span), BUSCO completeness metrics (`-b`), and C-values (`-c`) for the taxon <i>Arabidopsis thaliana</i>.

Two of the flags are different. `-p` (`--phylogeny`) will return a tax tree search. This means that a node on the tree and all of its descendents will be included in the query. For example:

`goat search -pa -t Arabidopsis` 

Will return aggregated results for every species in the genus <i>Arabidopsis</i>. If the default number of return hits is lower than the possible number of results, there will be a warning (in this case):

`[-]     For search query Arabidopsis, size specified (50) was less than the number of results returned, (51).`

So up the `--size` option to `51`.

You may not want aggregated results, instead requiring direct values. This is where the `-r --raw` flag is handy. Adding this flag will return all direct results. For example:

`goat search -pra -t Arabidopsis --size 51` 

Will display each direct result in turn for the assembly statistics implemented so far.

### Options

As for the options, we have already seen `size`. Next up is `--ranks`. Adding one of `subspecies, species, genus, family, order, class, phylum, kingdom, superkingdom` to this option will prepend a tab separated taxonomy for each record. Use it like this:

`goat search -pa -t Arabidopsis --ranks genus` 

Another way of searching for multiple species (related or not) is to use `-t` with quotes around a comma separated list. Like this:

`goat search -ra -t "Arabidopsis thaliana, Zea mays, Solanum tuberosum"`

This search will return all the assembly spans and levels for the three taxa specified.

Large numbers of requests are probably best submitted using a file, e.g:

`goat search -k -f examples/brassicaceae.txt > brassicaceae_karyotype_data.tsv`

### Feedback, requests, etc

Please test `goat`. Please find bugs and tell me. Please send feature requests. Creating an issue in this repo is probably the best way to go!