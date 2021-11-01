# GoaT

A command line interface for GoaT (Genomes on a Tree). GoaT presents metadata for taxa across the tree of life. 

The CLI here builds URLs to query the <b><a href="https://goat.genomehubs.org/api-docs/">Goat API</a></b> and parses the returned JSON files. The underlying API is very powerful, but complex. The aim of this CLI is to remove some of the complexity for the end user. Currently TSV's are returned, with variables separated by a delimeter line (e.g. `"--- Variable 1 ---"`).

This CLI is actively being built so no guarantees are made for stability or bugs. An official release will be made soon.

## Download

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

Executables for common systems will be released soon.

## Usage

Currently, all functionality is through the `goat search` subcommand. `goat search --help` will bring up the help below:

```
goat-search 
Query the search API.

USAGE:
    goat search [FLAGS] [OPTIONS] --file <file> --taxon <taxon>

FLAGS:
    -z, --all            This flag indicates all data should be printed.
    -a, --assembly       This flag indicates assembly data should be printed.
    -b, --busco          Include BUSCO estimates?
    -c, --c-values       This flag indicates C-value data should be printed.
    -g, --genome-size    This flag indicates genome size data should be printed.
    -h, --help           Prints help information
    -k, --karyotype      This flag indicates karyotype data should be printed.
    -p, --phylogeny      Get information for all descendents of a common ancestor.
    -r, --raw            This flag indicates raw values should be all listed out. So you can do your own aggregations
                         for example.
    -u, --url            Print the underlying GoaT API URL. Useful for debugging.
    -V, --version        Prints version information

OPTIONS:
    -f, --file <file>      A file of NCBI taxonomy ID's (tips) and/or binomial names.
                           Each line should contain a single entry.
        --ranks <ranks>    Choose a rank to display with the results. All values up to the given rank are displayed.
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

`[-]	Only 50 results are displayed, but there are 51 hits from GoaT.`

So up the `--size` option to `51`.

You may not want aggregated results, instead requiring direct values. This is where the `-r --raw` flag is handy. Adding this flag will return all direct results. For example:

`goat search -pra -t Arabidopsis --size 51` 

Will display each direct result in turn for the assembly statistics implemented so far.

### Options

As for the options, we have already seen `size`. Next up is `--ranks`. Adding one of `subspecies, species, genus, family, order, class, phylum, kingdom, superkingdom` to this option will prepend a tab separated taxonomy for each record. Use it like this:

`goat search -pa -t Arabidopsis --ranks genus` 

The `file` option is in progress. You can pass a file with one taxon per line to do concurrent searches. See `exampes/taxids.txt`.

Another way of searching for multiple species (related or not) is to use `-t` with quotes around a comma separated list. Like this:

`goat search -ra -t "Arabidopsis thaliana, Zea mays, Solanum tuberosum"`

This search will return all the assembly spans and levels for the three taxa specified.

### Feedback, requests, etc

Please test `goat`. Please find bugs and tell me. Please send feature requests. Creating an issue in this repo is probably the best way to go!