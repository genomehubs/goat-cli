use crate::utils::expression::{Function, TypeOf, Variable};
use lazy_static::lazy_static;
use std::collections::BTreeMap;

// https://stackoverflow.com/questions/27582739/how-do-i-create-a-hashmap-literal

/// Makes a static [`BTreeMap`] from the input of `GOAT_VARIABLE_DATA`.
macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
}

// this is data for the `taxon` index.

lazy_static! {
    /// Automatically generated GoaT variable data from a Bash script in the
    /// `/vars` directory.
    pub static ref GOAT_TAXON_VARIABLE_DATA: BTreeMap<&'static str, Variable<'static>> = collection!(
        // automated input start taxon
        "c_value" => Variable { display_name: "C value", type_of: TypeOf::HalfFloat, functions: Function::Some(vec!["min", "max"]) },
        "c_value_method" => Variable { display_name: "C value method", type_of: TypeOf::Keyword(vec!["biochemical analysis", "bulk fluorometric assay", "complete genome sequencing", "feulgen densitometry", "feulgen image analysis densitometry", "flow cytometry", "flow karyotyping", "fluorescence fading analysis", "gallocyanin chrom alum densitometry", "methyl green densitometry", "not specified", "static cell fluorometry", "ultraviolet microscopy", "unknown", "biochemical analysis", "feulgen image analysis densitometry", "flow cytometry", "feulgen densitometry", "feulgen densitometry & flow cytometry", "microdensitometry", "pulse field gel electrophoresis", "reassociation kinetics", "whole genome sequencing"]), functions: Function::None },
        "c_value_cell_type" => Variable { display_name: "C value cell type", type_of: TypeOf::Keyword(vec!["antennae", "antennal gland", "blood cells", "brain", "buccal epithelium", "coelomocytes", "corneal epithelium", "digestive gland", "dorsal fin clip", "egg", "embyro", "epidermis", "exopodite", "fibroblasts", "fin clips", "germarium", "gills", "haemocytes", "heart cells", "individual chromosomes", "intestine", "kidney cells", "legs", "leukocytes", "liver", "lung (culture)", "mantle", "midgut", "muscle cells", "ne", "not specified", "oocytes", "ovaries", "pancreas", "pharynx", "polypide cells in suspension", "red blood cells", "retinal cells", "salivary gland", "somatic cells", "sperm", "spleen", "tentacles", "testes", "thymus", "tissue culture", "various", "ventral hypodermal chord", "whole body", "whole body squash"]), functions: Function::None },
        "genome_size" => Variable { display_name: "Genome size", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "genome_size_kmer" => Variable { display_name: "Genome size kmer", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "genome_size_draft" => Variable { display_name: "Genome size draft", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "contig_n50" => Variable { display_name: "Contig N50", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "assembly_date" => Variable { display_name: "Last updated", type_of: TypeOf::Date, functions: Function::Some(vec!["min", "max"]) },
        "gene_count" => Variable { display_name: "Gene count", type_of: TypeOf::Integer, functions: Function::Some(vec!["min", "max"]) },
        "ebp_standard_criteria" => Variable { display_name: "EBP standard criteria", type_of: TypeOf::Keyword(vec!["6.c", "6.7", "5.c", "5.6"]), functions: Function::None },
        "ebp_standard_date" => Variable { display_name: "EBP metric date", type_of: TypeOf::Date, functions: Function::Some(vec!["min", "max"]) },
        "mitochondrion_assembly_span" => Variable { display_name: "mitochondrion span", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "mitochondrion_gc_percent" => Variable { display_name: "mitochondrion gc percent", type_of: TypeOf::TwoDP, functions: Function::None },
        "plastid_assembly_span" => Variable { display_name: "plastid span", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "plastid_gc_percent" => Variable { display_name: "plastid gc percent", type_of: TypeOf::TwoDP, functions: Function::None },
        "chromosome_number" => Variable { display_name: "Chromosome number", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
        "haploid_number" => Variable { display_name: "Haploid number", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
        "sex_determination" => Variable { display_name: "Sex karyotype features", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "ploidy" => Variable { display_name: "Ploidy", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
        "ploidy_inferred" => Variable { display_name: "Estimated ploidy", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
        "ploidy_descriptive" => Variable { display_name: "Ploidy descriptive", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "programmed_dna_elimination" => Variable { display_name: "Programmed DNA elimination present", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "assembly_level" => Variable { display_name: "Assembly level", type_of: TypeOf::Keyword(vec!["complete genome", "chromosome", "scaffold", "contig"]), functions: Function::None },
        "assembly_span" => Variable { display_name: "Assembly span", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "biosample" => Variable { display_name: "biosample", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "scaffold_n50" => Variable { display_name: "Scaffold N50", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "sample_sex" => Variable { display_name: "Sample sex", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "sample_location" => Variable { display_name: "location", type_of: TypeOf::None, functions: Function::None },
        "total_runs" => Variable { display_name: "total_runs", type_of: TypeOf::Integer, functions: Function::None },
        "total_reads" => Variable { display_name: "total_reads", type_of: TypeOf::Long, functions: Function::None },
        "library_source" => Variable { display_name: "library_source", type_of: TypeOf::Keyword(vec!["transcriptomic", "single cell"]), functions: Function::None },
        "platform" => Variable { display_name: "platform", type_of: TypeOf::Keyword(vec!["illumina", "oxford_nanopore", "pacbio_smrt"]), functions: Function::None },
        "sra_accession" => Variable { display_name: "sra_accession", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "run_accession" => Variable { display_name: "run_accession", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "reads" => Variable { display_name: "reads", type_of: TypeOf::Long, functions: Function::None },
        "country_list" => Variable { display_name: "Country list", type_of: TypeOf::Keyword(vec!["af", "al", "dz", "as", "ad", "ao", "ai", "aq", "ag", "ar", "am", "aw", "au", "at", "az", "bs", "bh", "bd", "bb", "by", "be", "bz", "bj", "bm", "bt", "bo", "bq", "ba", "bw", "bv", "br", "io", "bn", "bg", "bf", "bi", "cv", "kh", "cm", "ca", "ky", "cf", "td", "cl", "cn", "cx", "cc", "co", "km", "cd", "cg", "ck", "cr", "hr", "cu", "cw", "cy", "cz", "ci", "dk", "dj", "dm", "do", "ec", "eg", "sv", "gq", "er", "ee", "sz", "et", "fk", "fo", "fj", "fi", "fr", "gf", "pf", "tf", "ga", "gm", "ge", "de", "gh", "gi", "gr", "gl", "gd", "gp", "gu", "gt", "gg", "gn", "gw", "gy", "ht", "hm", "va", "hn", "hk", "hu", "is", "in", "id", "ir", "iq", "ie", "im", "il", "it", "jm", "jp", "je", "jo", "kz", "ke", "ki", "kp", "kr", "kw", "kg", "la", "lv", "lb", "ls", "lr", "ly", "li", "lt", "lu", "mo", "mg", "mw", "my", "mv", "ml", "mt", "mh", "mq", "mr", "mu", "yt", "mx", "fm", "md", "mc", "mn", "me", "ms", "ma", "mz", "mm", "na", "nr", "np", "nl", "nc", "nz", "ni", "ne", "ng", "nu", "nf", "mp", "no", "om", "pk", "pw", "ps", "pa", "pg", "py", "pe", "ph", "pn", "pl", "pt", "pr", "qa", "mk", "ro", "ru", "rw", "re", "bl", "sh", "kn", "lc", "mf", "pm", "vc", "ws", "sm", "st", "sa", "sn", "rs", "sc", "sl", "sg", "sx", "sk", "si", "sb", "so", "za", "gs", "ss", "es", "lk", "sd", "sr", "sj", "se", "ch", "sy", "tw", "tj", "tz", "th", "tl", "tg", "tk", "to", "tt", "tn", "tr", "tm", "tc", "tv", "ug", "ua", "ae", "gb", "um", "us", "uy", "uz", "vu", "ve", "vn", "vg", "vi", "wf", "eh", "ye", "zm", "zw", "ax"]), functions: Function::None },
        "btk_nohit" => Variable { display_name: "BTK no hit", type_of: TypeOf::OneDP, functions: Function::None },
        "btk_target" => Variable { display_name: "BTK_target", type_of: TypeOf::OneDP, functions: Function::None },
        "busco_completeness" => Variable { display_name: "BUSCO completeness", type_of: TypeOf::OneDP, functions: Function::None },
        "busco_lineage" => Variable { display_name: "BUSCO lineage", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "busco_string" => Variable { display_name: "BUSCO string", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "echabs92" => Variable { display_name: "EC Habitats Directive 1992", type_of: TypeOf::Keyword(vec!["echabs92_annex_iib", "echabs92_annex_ivb", "echabs92_annex_iva"]), functions: Function::None },
        "habreg_2017" => Variable { display_name: "Conservation of Habitats and Species Regulations 2017", type_of: TypeOf::Keyword(vec!["habreg-sch2", "habreg-sch5"]), functions: Function::None },
        "marhabreg-2017" => Variable { display_name: "Conservation of Offshore Marine Habitats and Species Regulations 2017", type_of: TypeOf::Keyword(vec!["marhabreg-sch1"]), functions: Function::None },
        "waca_1981" => Variable { display_name: "Wildlife and Countryside Act 1981", type_of: TypeOf::Keyword(vec!["waca-sch1", "waca-sch5"]), functions: Function::None },
        "isb_wildlife_act_1976" => Variable { display_name: "Irish Statute Book Wildlife Act, 1976", type_of: TypeOf::Keyword(vec!["iwa-nsch3", "iwa-sch5"]), functions: Function::None },
        "protection_of_badgers_act_1992" => Variable { display_name: "Protection of Badgers Act 1992", type_of: TypeOf::Keyword(vec!["badgers92"]), functions: Function::None },
        "bioproject" => Variable { display_name: "bioproject", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "contributing_project_lab" => Variable { display_name: "Contributing project-lab", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "sample_collected_by" => Variable { display_name: "Sample collected by", type_of: TypeOf::Keyword(vec!["dalu", "ghc", "mba", "nhm", "nsu", "psu", "qmul", "rbge", "kew", "san", "ubc", "derb", "oxf", "vien"]), functions: Function::None },
        "number_acquired" => Variable { display_name: "Number acquired", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
        "sequencing_status_africabp" => Variable { display_name: "Sequencing status AfricaBP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_ag100pest" => Variable { display_name: "Sequencing status Ag100Pest", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_agi" => Variable { display_name: "Sequencing status AGI", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_arg" => Variable { display_name: "Sequencing status AusARG", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_asg" => Variable { display_name: "Sequencing status ASG", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_atlasea" => Variable { display_name: "Sequencing status AtlaSea", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_b10k" => Variable { display_name: "Sequencing status B10K", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_bat1k" => Variable { display_name: "Sequencing status Bat1K", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_canbp" => Variable { display_name: "Sequencing status CanBP - CBP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_cbp" => Variable { display_name: "Sequencing status CBP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_ccgp" => Variable { display_name: "Sequencing status CCGP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_cfgp" => Variable { display_name: "Sequencing status CFGP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_cgp" => Variable { display_name: "Sequencing status CGP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_dtol" => Variable { display_name: "Sequencing status DToL", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_ebpn" => Variable { display_name: "Sequencing status EBPN - EBP-Nor", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_endemixit" => Variable { display_name: "Sequencing status ENDEMIXIT", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_erga-bge" => Variable { display_name: "Sequencing status ERGA-BGE", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_erga-ch" => Variable { display_name: "Sequencing status ERGA-CH", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_erga-pil" => Variable { display_name: "Sequencing status ERGA-PIL", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_eurofish" => Variable { display_name: "Sequencing status EUROFISH", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_gaga" => Variable { display_name: "Sequencing status GAGA", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_gap" => Variable { display_name: "Sequencing status GAP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_gbr" => Variable { display_name: "Sequencing status GBR", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_giga" => Variable { display_name: "Sequencing status GIGA", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_i5k" => Variable { display_name: "Sequencing status i5K", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_ilebp" => Variable { display_name: "Sequencing status IlEBP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_1kfg" => Variable { display_name: "Sequencing status 1KFG", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_lmgp" => Variable { display_name: "Sequencing status LMGP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_loewe-tbg" => Variable { display_name: "Sequencing status LOEWE-TBG", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_metainvert" => Variable { display_name: "Sequencing status METAINVERT", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_og" => Variable { display_name: "Sequencing status OG", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_ogg" => Variable { display_name: "Sequencing status OGG", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_omg" => Variable { display_name: "Sequencing status OMG", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_other" => Variable { display_name: "Sequencing status OTHER", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_pgp" => Variable { display_name: "Sequencing status PGP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_phyloalps" => Variable { display_name: "Sequencing status PhyloAlps", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_prgp" => Variable { display_name: "Sequencing status PRGP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_psyche" => Variable { display_name: "Sequencing status Psyche", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_squalomix" => Variable { display_name: "Sequencing status Squalomix", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_tsi" => Variable { display_name: "Sequencing status TSI", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_vgp" => Variable { display_name: "Sequencing status VGP", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_ygg" => Variable { display_name: "Sequencing status YGG", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status_zoonomia" => Variable { display_name: "Sequencing status Zoonomia", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "sequencing_status" => Variable { display_name: "Sequencing status", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "open", "in_assembly", "data_generation", "in_progress", "sample_acquired", "sample_collected", "sample_available"]), functions: Function::None },
        "open" => Variable { display_name: "Open", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "cngb", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "sample_collected" => Variable { display_name: "Sample collected", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "cngb", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "sample_acquired" => Variable { display_name: "Sample acquired", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "cngb", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "in_progress" => Variable { display_name: "In progress", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "cngb", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "long_list" => Variable { display_name: "Long list", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "odb10_lineage" => Variable { display_name: "Busco_odb10 lineage", type_of: TypeOf::Keyword(vec!["aconoidasida_odb10", "actinopterygii_odb10", "agaricales_odb10", "agaricomycetes_odb10", "alveolata_odb10", "apicomplexa_odb10", "arachnida_odb10", "arthropoda_odb10", "ascomycota_odb10", "aves_odb10", "basidiomycota_odb10", "boletales_odb10", "brassicales_odb10", "capnodiales_odb10", "carnivora_odb10", "cetartiodactyla_odb10", "chaetothyriales_odb10", "chlorophyta_odb10", "coccidia_odb10", "cyprinodontiformes_odb10", "diptera_odb10", "dothideomycetes_odb10", "embryophyta_odb10", "endopterygota_odb10", "euarchontoglires_odb10", "eudicots_odb10", "euglenozoa_odb10", "eukaryota_odb10", "eurotiales_odb10", "eurotiomycetes_odb10", "eutheria_odb10", "fabales_odb10", "fungi_odb10", "glires_odb10", "glomerellales_odb10", "helotiales_odb10", "hemiptera_odb10", "hymenoptera_odb10", "hypocreales_odb10", "insecta_odb10", "laurasiatheria_odb10", "leotiomycetes_odb10", "lepidoptera_odb10", "liliopsida_odb10", "mammalia_odb10", "metazoa_odb10", "microsporidia_odb10", "mollusca_odb10", "mucorales_odb10", "mucoromycota_odb10", "nematoda_odb10", "onygenales_odb10", "passeriformes_odb10", "plasmodium_odb10", "pleosporales_odb10", "poales_odb10", "polyporales_odb10", "primates_odb10", "saccharomycetes_odb10", "sauropsida_odb10", "solanales_odb10", "sordariomycetes_odb10", "stramenopiles_odb10", "tetrapoda_odb10", "tremellomycetes_odb10", "vertebrata_odb10", "viridiplantae_odb10"]), functions: Function::None },
        "sample_available" => Variable { display_name: "Sample available", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "cngb", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "resampling_required" => Variable { display_name: "Resampling required", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "cngb", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "insdc_submitted" => Variable { display_name: "In progress", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "cngb", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "insdc_open" => Variable { display_name: "Open on INSDC", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "cngb", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "published" => Variable { display_name: "Published", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "cngb", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "other_priority" => Variable { display_name: "Other priority", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "family_representative" => Variable { display_name: "Family representative", type_of: TypeOf::Keyword(vec!["africabp", "ag100pest", "agi", "arg", "asg", "atlasea", "bat1k", "b10k", "bpa", "canbp", "cbp", "ccgp", "cfgp", "cgp", "dtol", "ebpn", "ein", "endemixit", "erga", "erga-bge", "erga-ch", "erga-pil", "eurofish", "gaga", "gap", "gbr", "giga", "i5k", "ilebp", "1kfg", "lmgp", "loewe-tbg", "metainvert", "og", "ogg", "omg", "other", "pgp", "phyloalps", "prgp", "psyche", "squalomix", "tsi", "vgp", "ygg", "zoonomia"]), functions: Function::None },
        "data_generation" => Variable { display_name: "data_generation", type_of: TypeOf::None, functions: Function::None },
        "in_assembly" => Variable { display_name: "in_assembly", type_of: TypeOf::None, functions: Function::None },
        "sequencing_status_beenome100" => Variable { display_name: "sequencing_status_beenome100", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "sequencing_status_ebp" => Variable { display_name: "sequencing_status_ebp", type_of: TypeOf::None, functions: Function::None },
        "sequencing_status_erga" => Variable { display_name: "sequencing_status_erga", type_of: TypeOf::None, functions: Function::None },
        // automated input end taxon
    );
}

// this is the data for `assembly` index.

lazy_static! {
    /// Automatically generated GoaT variable data from a Bash script in the
    /// `/vars` directory.
    pub static ref GOAT_ASSEMBLY_VARIABLE_DATA: BTreeMap<&'static str, Variable<'static>> = collection!(
        // automated input start assembly
        "ebp_standard_date" => Variable { display_name: "EBP metric date", type_of: TypeOf::Date, functions: Function::None },
        "ebp_standard_criteria" => Variable { display_name: "EBP standard criteria", type_of: TypeOf::Keyword(vec!["6.c", "6.7", "5.c", "5.6"]), functions: Function::None },
        "mitochondrion_assembly_span" => Variable { display_name: "mitochondrion_assembly_span", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "mitochondrion_gc_percent" => Variable { display_name: "mitochondrion_gc_percent", type_of: TypeOf::TwoDP, functions: Function::None },
        "plastid_assembly_span" => Variable { display_name: "plastid_assembly_span", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "plastid_gc_percent" => Variable { display_name: "plastid_gc_percent", type_of: TypeOf::TwoDP, functions: Function::None },
        "qv_score" => Variable { display_name: "Quality Value score", type_of: TypeOf::None, functions: Function::None },
        "source_accession" => Variable { display_name: "source_accession", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "mitochondrion_accession" => Variable { display_name: "mitochondrion_accession", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "mitochondrion_scaffolds" => Variable { display_name: "mitochondrion_scaffolds", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "plastid_accession" => Variable { display_name: "plastid_accession", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "plastid_scaffolds" => Variable { display_name: "plastid_scaffolds", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "assigned_percent" => Variable { display_name: "assigned_percent", type_of: TypeOf::TwoDP, functions: Function::None },
        "assembly_status" => Variable { display_name: "assembly_status", type_of: TypeOf::Keyword(vec!["primary", "alternate"]), functions: Function::None },
        "bioproject" => Variable { display_name: "bioproject", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "biosample" => Variable { display_name: "biosample", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "gene_count" => Variable { display_name: "Gene count", type_of: TypeOf::Integer, functions: Function::None },
        "assembly_level" => Variable { display_name: "Assembly level", type_of: TypeOf::Keyword(vec!["complete genome", "chromosome", "scaffold", "contig"]), functions: Function::None },
        "assembly_span" => Variable { display_name: "Assembly span", type_of: TypeOf::Long, functions: Function::None },
        "contig_n50" => Variable { display_name: "Contig N50", type_of: TypeOf::Long, functions: Function::None },
        "scaffold_n50" => Variable { display_name: "Scaffold N50", type_of: TypeOf::Long, functions: Function::None },
        "last_updated" => Variable { display_name: "Last updated", type_of: TypeOf::Date, functions: Function::None },
        "ebp_metric_date" => Variable { display_name: "EBP metric date", type_of: TypeOf::Date, functions: Function::None },
        "organelle" => Variable { display_name: "organelle", type_of: TypeOf::Keyword(vec!["nucleus", "mitochondrion", "chloroplast", "plastid", "apicoplast"]), functions: Function::None },
        "protein_count" => Variable { display_name: "Protein count", type_of: TypeOf::Integer, functions: Function::None },
        "pseudogene_count" => Variable { display_name: "Pseudogene count", type_of: TypeOf::Integer, functions: Function::None },
        "noncoding_gene_count" => Variable { display_name: "Non-coding gene count", type_of: TypeOf::Integer, functions: Function::None },
        "sample_sex" => Variable { display_name: "Sample sex", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "isolate" => Variable { display_name: "Isolate", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "assembly_type" => Variable { display_name: "Assembly type", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "ungapped_span" => Variable { display_name: "Ungapped span", type_of: TypeOf::Long, functions: Function::None },
        "contig_l50" => Variable { display_name: "Contig L50", type_of: TypeOf::Long, functions: Function::None },
        "scaffold_l50" => Variable { display_name: "Scaffold L50", type_of: TypeOf::Long, functions: Function::None },
        "contig_count" => Variable { display_name: "Contig count", type_of: TypeOf::Long, functions: Function::None },
        "scaffold_count" => Variable { display_name: "Scaffold count", type_of: TypeOf::Long, functions: Function::None },
        "chromosome_count" => Variable { display_name: "Chromosome count", type_of: TypeOf::Long, functions: Function::None },
        "sequence_count" => Variable { display_name: "Sequence count", type_of: TypeOf::Long, functions: Function::None },
        "refseq_category" => Variable { display_name: "RefSeq category", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "submitter" => Variable { display_name: "Submitter", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "gc_percent" => Variable { display_name: "gc_percent", type_of: TypeOf::TwoDP, functions: Function::None },
        "n_percent" => Variable { display_name: "n_percent", type_of: TypeOf::TwoDP, functions: Function::None },
        "busco_completeness" => Variable { display_name: "BUSCO completeness", type_of: TypeOf::OneDP, functions: Function::None },
        "busco_lineage" => Variable { display_name: "BUSCO lineage", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "busco_string" => Variable { display_name: "BUSCO string", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
        "btk_nohit" => Variable { display_name: "BTK no hit", type_of: TypeOf::OneDP, functions: Function::None },
        "btk_target" => Variable { display_name: "BTK_target", type_of: TypeOf::OneDP, functions: Function::None },
        // automated input end assembly
    );
}
