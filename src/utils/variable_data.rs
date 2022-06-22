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

lazy_static! {
    /// Automatically generated GoaT variable data from a Bash script in the
    /// `/vars` directory.
    pub static ref GOAT_VARIABLE_DATA: BTreeMap<&'static str, Variable<'static>> = collection!(
        // automated input start
		"assembly_level" => Variable { display_name: "Assembly level", type_of: TypeOf::Keyword(vec!["complete genome", "chromosome", "scaffold", "contig"]), functions: Function::None },
		"assembly_span" => Variable { display_name: "Assembly span", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
		"bioproject" => Variable { display_name: "Bioproject", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"biosample" => Variable { display_name: "Biosample", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"contig_n50" => Variable { display_name: "Contig N50", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
		"assembly_date" => Variable { display_name: "Last updated", type_of: TypeOf::Date, functions: Function::Some(vec!["min", "max"]) },
		"scaffold_n50" => Variable { display_name: "Scaffold N50", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
		"gene_count" => Variable { display_name: "Gene count", type_of: TypeOf::Integer, functions: Function::Some(vec!["min", "max"]) },
		"sample_sex" => Variable { display_name: "Sample sex", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"ebp_metric_date" => Variable { display_name: "EBP metric date", type_of: TypeOf::Date, functions: Function::Some(vec!["min", "max"]) },
		"busco_completeness" => Variable { display_name: "BUSCO completeness", type_of: TypeOf::OneDP, functions: Function::None },
		"busco_lineage" => Variable { display_name: "BUSCO lineage", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"busco_string" => Variable { display_name: "BUSCO string", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"gc_percent" => Variable { display_name: "GC percent", type_of: TypeOf::OneDP, functions: Function::None },
		"n_percent" => Variable { display_name: "N percent", type_of: TypeOf::OneDP, functions: Function::None },
		"nohit" => Variable { display_name: "No hit", type_of: TypeOf::OneDP, functions: Function::None },
		"target" => Variable { display_name: "Target", type_of: TypeOf::OneDP, functions: Function::None },
		"mitochondrion_assembly_span" => Variable { display_name: "mitochondrion span", type_of: TypeOf::Long, functions: Function::None },
		"mitochondrion_gc_percent" => Variable { display_name: "mitochondrion GC%", type_of: TypeOf::TwoDP, functions: Function::None },
		"plastid_assembly_span" => Variable { display_name: "plastid span", type_of: TypeOf::Long, functions: Function::None },
		"plastid_gc_percent" => Variable { display_name: "plastid GC%", type_of: TypeOf::TwoDP, functions: Function::None },
		"odb10_lineage" => Variable { display_name: "Busco_odb10 lineage", type_of: TypeOf::Keyword(vec!["aconoidasida_odb10", "actinopterygii_odb10", "agaricales_odb10", "agaricomycetes_odb10", "alveolata_odb10", "apicomplexa_odb10", "arachnida_odb10", "arthropoda_odb10", "ascomycota_odb10", "aves_odb10", "basidiomycota_odb10", "boletales_odb10", "brassicales_odb10", "capnodiales_odb10", "carnivora_odb10", "cetartiodactyla_odb10", "chaetothyriales_odb10", "chlorophyta_odb10", "coccidia_odb10", "cyprinodontiformes_odb10", "diptera_odb10", "dothideomycetes_odb10", "embryophyta_odb10", "endopterygota_odb10", "euarchontoglires_odb10", "eudicots_odb10", "euglenozoa_odb10", "eukaryota_odb10", "eurotiales_odb10", "eurotiomycetes_odb10", "eutheria_odb10", "fabales_odb10", "fungi_odb10", "glires_odb10", "glomerellales_odb10", "helotiales_odb10", "hemiptera_odb10", "hymenoptera_odb10", "hypocreales_odb10", "insecta_odb10", "laurasiatheria_odb10", "leotiomycetes_odb10", "lepidoptera_odb10", "liliopsida_odb10", "mammalia_odb10", "metazoa_odb10", "microsporidia_odb10", "mollusca_odb10", "mucorales_odb10", "mucoromycota_odb10", "nematoda_odb10", "onygenales_odb10", "passeriformes_odb10", "plasmodium_odb10", "pleosporales_odb10", "poales_odb10", "polyporales_odb10", "primates_odb10", "saccharomycetes_odb10", "sauropsida_odb10", "solanales_odb10", "sordariomycetes_odb10", "stramenopiles_odb10", "tetrapoda_odb10", "tremellomycetes_odb10", "vertebrata_odb10", "viridiplantae_odb10"]), functions: Function::None },
		"chromosome_number" => Variable { display_name: "Chromosome number", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
		"haploid_number" => Variable { display_name: "Haploid number", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
		"sex_determination" => Variable { display_name: "Sex Determination", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"ploidy" => Variable { display_name: "Ploidy", type_of: TypeOf::Short, functions: Function::Some(vec!["min", "max"]) },
		"echabs92" => Variable { display_name: "EC Habitats Directive 1992", type_of: TypeOf::Keyword(vec!["echabs92_annex_iib", "echabs92_annex_ivb", "echabs92_annex_iva"]), functions: Function::None },
		"habreg_2017" => Variable { display_name: "Conservation of Habitats and Species Regulations 2017", type_of: TypeOf::Keyword(vec!["habreg-sch2", "habreg-sch5"]), functions: Function::None },
		"marhabreg-2017" => Variable { display_name: "Conservation of Offshore Marine Habitats and Species Regulations 2017", type_of: TypeOf::Keyword(vec!["marhabreg-sch1"]), functions: Function::None },
		"waca_1981" => Variable { display_name: "Wildlife and Countryside Act 1981", type_of: TypeOf::Keyword(vec!["waca-sch1", "waca-sch5"]), functions: Function::None },
		"isb_wildlife_act_1976" => Variable { display_name: "Irish Statute Book Wildlife Act, 1976", type_of: TypeOf::Keyword(vec!["iwa-nsch3", "iwa-sch5"]), functions: Function::None },
		"protection_of_badgers_act_1992" => Variable { display_name: "Protection of Badgers Act 1992", type_of: TypeOf::Keyword(vec!["badgers92"]), functions: Function::None },
		"country_list" => Variable { display_name: "Country list", type_of: TypeOf::Keyword(vec!["gb", "ie"]), functions: Function::None },
		"c_value" => Variable { display_name: "C value", type_of: TypeOf::HalfFloat, functions: Function::Some(vec!["min", "max"]) },
		"c_value_method" => Variable { display_name: "C value method", type_of: TypeOf::Keyword(vec!["biochemical analysis", "bulk fluorometric assay", "complete genome sequencing", "feulgen densitometry", "feulgen image analysis densitometry", "flow cytometry", "flow karyotyping", "fluorescence fading analysis", "gallocyanin chrom alum densitometry", "methyl green densitometry", "not specified", "static cell fluorometry", "ultraviolet microscopy", "unknown", "biochemical analysis", "feulgen image analysis densitometry", "flow cytometry", "feulgen densitometry", "feulgen densitometry & flow cytometry", "microdensitometry", "pulse field gel electrophoresis", "reassociation kinetics", "whole genome sequencing"]), functions: Function::None },
		"c_value_cell_type" => Variable { display_name: "C value cell type", type_of: TypeOf::Keyword(vec!["antennae", "antennal gland", "blood cells", "brain", "buccal epithelium", "coelomocytes", "corneal epithelium", "digestive gland", "dorsal fin clip", "egg", "embyro", "epidermis", "exopodite", "fibroblasts", "fin clips", "germarium", "gills", "haemocytes", "heart cells", "individual chromosomes", "intestine", "kidney cells", "legs", "leukocytes", "liver", "lung (culture)", "mantle", "midgut", "muscle cells", "ne", "not specified", "oocytes", "ovaries", "pancreas", "pharynx", "polypide cells in suspension", "red blood cells", "retinal cells", "salivary gland", "somatic cells", "sperm", "spleen", "tentacles", "testes", "thymus", "tissue culture", "various", "ventral hypodermal chord", "whole body", "whole body squash"]), functions: Function::None },
		"genome_size" => Variable { display_name: "Genome size", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
		"genome_size_kmer" => Variable { display_name: "Genome size kmer", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
		"genome_size_draft" => Variable { display_name: "Genome size draft", type_of: TypeOf::Long, functions: Function::Some(vec!["min", "max"]) },
		"sample_collected" => Variable { display_name: "sample_collected", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"sample_collected_by" => Variable { display_name: "sample_collected_by", type_of: TypeOf::Keyword(vec!["nhm", "oxf", "sanger"]), functions: Function::None },
		"sample_acquired" => Variable { display_name: "sample_acquired", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"in_progress" => Variable { display_name: "in_progress", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"insdc_submitted" => Variable { display_name: "insdc_submitted", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"insdc_open" => Variable { display_name: "insdc_open", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"published" => Variable { display_name: "published", type_of: TypeOf::Keyword(vec![""]), functions: Function::None },
		"sequencing_status" => Variable { display_name: "sequencing_status", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_asg" => Variable { display_name: "sequencing_status_asg", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_b10k" => Variable { display_name: "sequencing_status_b10k", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_cbp" => Variable { display_name: "sequencing_status_cbp", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_cfgp" => Variable { display_name: "sequencing_status_cfgp", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_dtol" => Variable { display_name: "sequencing_status_dtol", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_ebp" => Variable { display_name: "sequencing_status_ebp", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_ebpn" => Variable { display_name: "sequencing_status_ebpn", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_endemixit" => Variable { display_name: "sequencing_status_endemixit", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_erga" => Variable { display_name: "sequencing_status_erga", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_eurofish" => Variable { display_name: "sequencing_status_eurofish", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_gaga" => Variable { display_name: "sequencing_status_gaga", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_metainvert" => Variable { display_name: "sequencing_status_metainvert", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_squalomix" => Variable { display_name: "sequencing_status_squalomix", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"sequencing_status_vgp" => Variable { display_name: "sequencing_status_vgp", type_of: TypeOf::Keyword(vec!["published", "insdc_open", "insdc_submitted", "in_progress", "sample_acquired", "sample_collected"]), functions: Function::None },
		"long_list" => Variable { display_name: "long_list", type_of: TypeOf::Keyword(vec!["asg", "cbp", "ebpn", "ein", "cfgp", "dtol", "ebpn", "endemixit", "erga", "eurofish", "gaga", "squalomix", "metainvert", "vgp", "agi", "arg", "gap", "gbr", "omg", "tsi", "b10k"]), functions: Function::None },
		"other_priority" => Variable { display_name: "other_priority", type_of: TypeOf::Keyword(vec!["asg", "cbp", "ebpn", "ein", "cfgp", "dtol", "ebpn", "endemixit", "erga", "eurofish", "gaga", "squalomix", "metainvert", "vgp", "agi", "arg", "gap", "gbr", "omg", "tsi", "b10k"]), functions: Function::None },
		"family_representative" => Variable { display_name: "family_representative", type_of: TypeOf::Keyword(vec!["asg", "cbp", "ebpn", "ein", "cfgp", "dtol", "ebpn", "endemixit", "erga", "eurofish", "gaga", "squalomix", "metainvert", "vgp", "agi", "arg", "gap", "gbr", "omg", "tsi", "b10k"]), functions: Function::None },
        // automated input end
    );
}
