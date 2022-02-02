# GoaT CLI

A command line interface for GoaT (Genomes on a Tree). GoaT presents metadata for taxa across the tree of life. 

The CLI here builds URLs to query the <b><a href="https://goat.genomehubs.org/api-docs/">Goat API</a></b>. The underlying API is very powerful, but complex. The aim of this CLI is to remove some of the complexity for the end user. Only TSV files are returned. 

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
goat 0.1.4
Max Brown <mb39@sanger.ac.uk>
GoaTs on a terminal. Combine flags to query metadata for any species.

USAGE:
    goat [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    count     Query the GoaT count API. Return the number of hits from any search.
    help      Print this message or the help of the given subcommand(s)
    lookup    Query the GoaT lookup API.
    newick    Query the GoaT record API, and return a newick.
    search    Query the GoaT search API.
```

### Search

`goat help search` or `goat search --help` will bring up the help below:

```
goat-search 0.1.4
Query the GoaT search API.

USAGE:
    goat search [OPTIONS]

OPTIONS:
    -a, --assembly                   Print assembly data (span & level)
    -A, --all                        Print all currently implemented GoaT variables.
    -b, --busco                      Print BUSCO estimates.
    -B, --bioproject                 Print the bioproject and biosample ID of records.
    -c, --c-values                   Print c-value data.
    -C, --country-list               Print list of countries where taxon is found.
    -d, --descendents                Get information for all descendents of a common ancestor.
    -D, --date                       Print EBP & assembly dates.
    -e, --expression <expression>    Experimental expression parser.
    -f, --file <file>                A file of NCBI taxonomy ID's (tips) and/or binomial names.
                                     Each line should contain a single entry.
                                     File size is limited to 500 entries.
    -g, --genome-size                Print genome size data.
    -G, --gene-count                 Print gene count data.
    -h, --help                       Print help information
    -i, --include-estimates          Include ancestral estimates. Omitting this flag includes only
                                     direct estimates from a taxon. Cannot be used with --raw.
    -k, --karyotype                  Print karyotype data (chromosome number & haploid number).
    -l, --legislation                Print legislation data.
    -m, --mitochondria               Print mitochondrial genome size & GC%.
    -n, --names                      Print all associated name data (synonyms, Tree of Life ID, and
                                     common names).
    -N, --n50                        Print the contig & scaffold n50 of assemblies.
    -p, --plastid                    Print plastid genome size & GC%.
    -P, --ploidy                     Print ploidy estimates.
        --print-expression           Print all variables in GoaT currently, with their associated
                                     variants.
                                     Useful for construction of expressions.
    -r, --raw                        Print raw values (i.e. no aggregation/summary).
    -R, --ranks <ranks>              Choose a rank to display with the results. All ranks up to the
                                     given rank are displayed. [default: none] [possible values:
                                     none, subspecies, species, genus, family, order, class, phylum,
                                     kingdom, superkingdom]
    -s, --size <size>                The number of results to return. Max 50,000 currently.
                                     [default: 50]
    -S, --sex-determination          Print sex determination data.
        --status                     Print all data associated with how far this taxon has
                                     progressed with genomic sequencing.
                                     This includes sample collection, acquisition, progress in
                                     sequencing, and whether submitted to INSDC.
    -t, --target-lists               Print target list data associated with each taxon.
    -T, --tidy                       Print data in tidy format.
    -u, --url                        Print the underlying GoaT API URL(s). Useful for debugging.
    -V, --version                    Print version information
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

#### Expressions

As of `goat-cli` version 0.1.4, the `-e (--expression)` option allows the user to filter data on the fly. For example, to search across all flowering plants for chromosomal level assemblies:

`goat search -dat Magnoliopsida --size 300 -e"assembly_level == chromosome"`

Or all flowering plants which have fewer than (or equal to) 12 haploid chromosomes, keeping assembly stats:

`goat search -dakt Magnoliopsida --size 300 -e"haploid_number <= 12"`

If you are unsure what variables to use, print the table of them:

`goat search --print-expression`

```
 Variable names in GoaT, with functional operator annotation.                                                           
+----------------------------------------------------------------------------------------------------------------------+
|        Expression Name         |               Display Name               |            Operators/Keywords            |
+--------------------------------+------------------------------------------+------------------------------------------+
|         assembly_date          |               Last updated               |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|         assembly_level         |              Assembly level              | == complete genome, chromosome, scaffold |
|                                |                                          |                 , contig                 |
+--------------------------------+------------------------------------------+------------------------------------------+
|         assembly_span          |              Assembly span               |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|       busco_completeness       |            BUSCO completeness            |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|            c_value             |                 C value                  |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|       c_value_cell_type        |            C value cell type             | == antennae, antennal gland, blood cells |
|                                |                                          | , brain, buccal epithelium, coelomocytes |
|                                |                                          | , corneal epithelium, digestive gland, d |
|                                |                                          | orsal fin clip, egg, embyro, epidermis,  |
|                                |                                          | exopodite, fibroblasts, fin clips, germa |
|                                |                                          | rium, gills, haemocytes, heart cells, in |
|                                |                                          | dividual chromosomes, intestine, kidney  |
|                                |                                          | cells, legs, leukocytes, liver, lung (cu |
|                                |                                          | lture), mantle, midgut, muscle cells, ne |
|                                |                                          | , not specified, oocytes, ovaries, pancr |
|                                |                                          | eas, pharynx, polypide cells in suspensi |
|                                |                                          | on, red blood cells, retinal cells, sali |
|                                |                                          | vary gland, somatic cells, sperm, spleen |
|                                |                                          | , tentacles, testes, thymus, tissue cult |
|                                |                                          | ure, various, ventral hypodermal chord,  |
|                                |                                          |      whole body, whole body squash       |
+--------------------------------+------------------------------------------+------------------------------------------+
|         c_value_method         |              C value method              | == biochemical analysis, bulk fluorometr |
|                                |                                          | ic assay, complete genome sequencing, fe |
|                                |                                          | ulgen densitometry, feulgen image analys |
|                                |                                          | is densitometry, flow cytometry, flow ka |
|                                |                                          | ryotyping, fluorescence fading analysis, |
|                                |                                          | gallocyanin chrom alum densitometry, me  |
|                                |                                          | thyl green densitometry, not specified,  |
|                                |                                          | static cell fluorometry, ultraviolet mic |
|                                |                                          |             roscopy, unknown             |
+--------------------------------+------------------------------------------+------------------------------------------+
|       chromosome_number        |            Chromosome number             |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|           contig_n50           |                Contig N50                |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|          country_list          |               Country list               |                  == gb                   |
+--------------------------------+------------------------------------------+------------------------------------------+
|        ebp_metric_date         |             EBP metric date              |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|            echabs92            |        EC Habitats Directive 1992        | == echabs92_annex_iib, echabs92_annex_iv |
|                                |                                          |          b, echabs92_annex_iva           |
+--------------------------------+------------------------------------------+------------------------------------------+
|           gc_percent           |                GC percent                |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|           gene_count           |                Gene count                |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|          genome_size           |               Genome size                |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|       genome_size_draft        |            Genome size draft             |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|        genome_size_kmer        |             Genome size kmer             |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|          habreg_2017           | Conservation of Habitats and Species Reg |       == habreg-sch2, habreg-sch5        |
|                                |              ulations 2017               |                                          |
+--------------------------------+------------------------------------------+------------------------------------------+
|         haploid_number         |              Haploid number              |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|     isb_wildlife_act_1976      |  Irish Statute Book Wildlife Act, 1976   |          == iwa-nsch3, iwa-sch5          |
+--------------------------------+------------------------------------------+------------------------------------------+
|         marhabreg-2017         | Conservation of Offshore Marine Habitats |            == marhabreg-sch1             |
|                                |       and Species Regulations 2017       |                                          |
+--------------------------------+------------------------------------------+------------------------------------------+
|  mitochondrion_assembly_span   |            mitochondrion span            |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|    mitochondrion_gc_percent    |            mitochondrion GC%             |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|           n_percent            |                N percent                 |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|             nohit              |                  No hit                  |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|     plastid_assembly_span      |               plastid span               |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|       plastid_gc_percent       |               plastid GC%                |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|             ploidy             |                  Ploidy                  |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
| protection_of_badgers_act_1992 |      Protection of Badgers Act 1992      |               == badgers92               |
+--------------------------------+------------------------------------------+------------------------------------------+
|          scaffold_n50          |               Scaffold N50               |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|             target             |                  Target                  |         !=, <, <=, =, ==, >, >=          |
+--------------------------------+------------------------------------------+------------------------------------------+
|           waca_1981            |    Wildlife and Countryside Act 1981     |         == waca-sch1, waca-sch5          |
+--------------------------------+------------------------------------------+------------------------------------------+
```

*`goat count` presents an identical CLI to `goat search`.*

### Lookup

`goat lookup` is very simple currently. It only requires a taxon NCBI ID, taxon name, or common name. It returns an NCBI tax-id, along with synonyms, authorities, and common names.

```
goat-lookup 0.1.4
Query the GoaT lookup API.

USAGE:
    goat lookup [OPTIONS]

OPTIONS:
    -h, --help             Print help information
    -s, --size <size>      The number of results to return. [default: 10]
    -t, --taxon <taxon>    The taxon to search. An NCBI taxon ID, or the name of a taxon at any
                           rank.
    -u, --url              Print lookup URL.
    -V, --version          Print version information
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
goat-newick 0.1.4
Query the GoaT record API, and return a newick.

USAGE:
    goat newick [OPTIONS]

OPTIONS:
    -h, --help             Print help information
    -r, --rank <rank>      The number of results to return. [default: species] [possible values:
                           species, genus, family, order]
    -t, --taxon <taxon>    The taxon to return a newick of. Multiple taxa will return the joint
                           tree.
    -u, --url              Print lookup URL.
    -V, --version          Print version information
```

E.g. get a cladogram of the genera in the family Fabaceae:

`goat newick -t "Fabaceae" -r genus`

### Feedback, requests, notes etc

This CLI is actively being built so no guarantees are made for stability or bugs.

This README will always be up to date with the latest *working* version. The released versions may lag behind.

Please test `goat`. Please try and break `goat`. Please find bugs and tell me. Please send feature requests. Creating an issue in this repo is probably the best way to go!
