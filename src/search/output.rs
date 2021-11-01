use crate::search::agg_values::{GoaTValueAgg, Records};
use crate::search::raw_values::{GoaTValue, RawRecords};
use crate::utils::ranks::RankHeaders;
use crate::{search::agg_values::MinMax, utils::url::FieldBuilder};

use anyhow::Result;

#[derive(Clone)]
pub struct CombinedValues {
    pub raw: Option<RawRecords>,
    pub agg: Option<Records>,
}

pub fn print_raw_output(
    awaited_fetches: Vec<Result<CombinedValues, anyhow::Error>>,
    fields: FieldBuilder,
    ranks_vec: RankHeaders,
) -> Result<()> {
    // still have to collect into Vec's and iterate later.
    // some/most of these will remain empty depending on the query.
    let mut raw_assembly_level_vec = Vec::new();
    let mut raw_assembly_span_vec = Vec::new();
    let mut raw_busco_vec = Vec::new();
    let mut raw_chromsome_vec = Vec::new();
    let mut raw_c_value_vec = Vec::new();
    let mut raw_genome_size_vec = Vec::new();
    let mut raw_haploid_vec = Vec::new();
    // display level 2
    let mut raw_mitochondrion_assembly_span_vec = Vec::new();
    let mut raw_mitochondrion_gc_percent_vec = Vec::new();

    // decompose FieldBuilder
    let all = fields.all;
    let assembly = fields.assembly;
    let busco = fields.busco;
    let cvalues = fields.cvalues;
    let gs = fields.gs;
    let karyotype = fields.karyotype;
    let mitochondrion = fields.mitochondrion;

    // iterate over all awaited fetches
    // match on raw records
    // iterate over records
    // match on goatvalue
    for el in &awaited_fetches {
        match el {
            Ok(e) => {
                for record in &e.raw {
                    for r in &record.0 {
                        match &r.value {
                            GoaTValue::RawAssemblyLevel(res) => {
                                if all || assembly {
                                    raw_assembly_level_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source_id.clone(),
                                        r.source.clone(),
                                        res,
                                    ));
                                }
                            }
                            GoaTValue::RawAssemblySpan(res) => {
                                if all || assembly {
                                    raw_assembly_span_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source_id.clone(),
                                        r.source.clone(),
                                        res,
                                    ));
                                }
                            }
                            GoaTValue::RawBuscoCompleteness(res) => {
                                if all || busco {
                                    raw_busco_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source.clone(),
                                        res,
                                    ));
                                }
                            }
                            GoaTValue::RawChromosomeNumber(res) => {
                                if all || karyotype {
                                    let mut comma_sep_chroms = String::new();
                                    let mut peek_chrom_vec = res.iter().peekable();

                                    while let Some(e) = peek_chrom_vec.next() {
                                        match e {
                                            Some(e) => {
                                                if peek_chrom_vec.peek().is_some() {
                                                    let to_add = format!("{},", e);
                                                    comma_sep_chroms += &to_add;
                                                } else {
                                                    comma_sep_chroms += &e.to_string();
                                                }
                                            }
                                            None => comma_sep_chroms += "",
                                        };
                                    }

                                    raw_chromsome_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source.clone(),
                                        comma_sep_chroms,
                                    ))
                                }
                            }
                            GoaTValue::RawCValue(res) => {
                                if all || cvalues {
                                    raw_c_value_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source.clone(),
                                        res,
                                    ))
                                }
                            }
                            GoaTValue::RawGenomeSize(res) => {
                                if all || gs {
                                    raw_genome_size_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source.clone(),
                                        res,
                                    ))
                                }
                            }
                            GoaTValue::RawHaploid(res) => {
                                if all || karyotype {
                                    let mut comma_sep_haps = String::new();
                                    let mut peek_chrom_vec = res.iter().peekable();

                                    while let Some(e) = peek_chrom_vec.next() {
                                        match e {
                                            Some(e) => {
                                                if peek_chrom_vec.peek().is_some() {
                                                    let to_add = format!("{},", e);
                                                    comma_sep_haps += &to_add;
                                                } else {
                                                    comma_sep_haps += &e.to_string();
                                                }
                                            }
                                            None => comma_sep_haps += "",
                                        };
                                    }
                                    raw_haploid_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source.clone(),
                                        comma_sep_haps,
                                    ))
                                }
                            }
                            GoaTValue::RawMitochondrionAssemblySpan(res) => {
                                if all || mitochondrion {
                                    raw_mitochondrion_assembly_span_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source_id.clone(),
                                        r.source.clone(),
                                        res,
                                    ));
                                }
                            }
                            GoaTValue::RawMitochondrionGCPercent(res) => {
                                if all || mitochondrion {
                                    raw_mitochondrion_gc_percent_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source_id.clone(),
                                        r.source.clone(),
                                        res,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Could not create output - {}", e),
        };
    }
    // print here
    // TODO: these prints could be shortened to a function call.
    if all || assembly {
        println!("--- Assembly Level ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_taxid", "source_id", "source", "assembly_type"
        );
        for el in raw_assembly_level_vec {
            println!("{}{}\t{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4, el.5)
        }
    }
    if all || assembly {
        println!("--- Assembly Span ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_taxid", "source_id", "source", "span"
        );
        for el in raw_assembly_span_vec {
            println!("{}{}\t{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4, el.5)
        }
    }
    if all || busco {
        println!("--- BUSCO Completeness ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_taxid", "source", "busco_completeness"
        );
        for el in raw_busco_vec {
            println!("{}{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4)
        }
    }
    if all || karyotype {
        println!("--- Chromosome Numbers ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "chromosome_number"
        );
        for el in raw_chromsome_vec {
            println!("{}{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4)
        }
    }
    if all || cvalues {
        println!("--- C-Values ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "c_value"
        );
        for el in raw_c_value_vec {
            println!("{}{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4)
        }
    }
    if all || gs {
        println!("--- Genome Sizes ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "genome_size"
        );
        for el in raw_genome_size_vec {
            println!("{}{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4)
        }
    }
    if all || karyotype {
        println!("--- Haploid Numbers ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "haploid_chromosome_number"
        );
        for el in raw_haploid_vec {
            println!("{}{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4)
        }
    }
    if all || mitochondrion {
        println!("--- Mitochondrial Assembly Span ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_id",
            "source",
            "source_id",
            "mitochondrial_assembly_span"
        );
        for el in raw_mitochondrion_assembly_span_vec {
            println!("{}{}\t{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4, el.5)
        }
    }
    if all || mitochondrion {
        println!("--- Mitochondrial GC Percent ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "source_id", "mitochondrial_gc_percent"
        );
        for el in raw_mitochondrion_gc_percent_vec {
            println!("{}{}\t{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4, el.5)
        }
    }

    Ok(())
}

pub fn print_agg_output(
    awaited_fetches: Vec<Result<CombinedValues, anyhow::Error>>,
    fields: FieldBuilder,
    ranks_vec: RankHeaders,
) -> Result<()> {
    // still have to collect into Vec's and iterate later.
    // some/most of these will remain empty depending on the query.
    let mut assembly_level_vec = Vec::new();
    let mut assembly_span_vec = Vec::new();
    let mut busco_vec = Vec::new();
    let mut chromsome_vec = Vec::new();
    let mut c_value_vec = Vec::new();
    let mut genome_size_vec = Vec::new();
    let mut haploid_vec = Vec::new();
    // display level 2
    let mut mitochondrion_assembly_span_vec = Vec::new();
    let mut mitochondrion_gc_percent_vec = Vec::new();

    // decompose FieldBuilder
    let all = fields.all;
    let assembly = fields.assembly;
    let busco = fields.busco;
    let cvalues = fields.cvalues;
    let gs = fields.gs;
    let karyotype = fields.karyotype;
    let mitochondrion = fields.mitochondrion;

    // iterate over the fetches and collect into separate vecs.
    for el in &awaited_fetches {
        match el {
            Ok(e) => {
                for record in &e.agg {
                    for r in &record.0 {
                        match &r.value {
                            GoaTValueAgg::AssemblyLevel(res) => {
                                if all || assembly {
                                    let mut min: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.min {
                                        min = e.unwrap_or(0)
                                    };
                                    let mut max: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.max {
                                        max = e.unwrap_or(0)
                                    };

                                    assembly_level_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_id.clone(),
                                        r.aggregation_source.clone(),
                                        min,
                                        max,
                                        r.count,
                                        r.aggregation_taxon_id.clone().unwrap_or("".to_string()),
                                        res,
                                        r.aggregation_method.clone(),
                                        r.aggregation_rank.clone().unwrap_or("".to_string()),
                                    ));
                                }
                            }
                            GoaTValueAgg::AssemblySpan(res) => {
                                if all || assembly {
                                    let mut min: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.min {
                                        min = e.unwrap_or(0)
                                    };
                                    let mut max: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.max {
                                        max = e.unwrap_or(0)
                                    };

                                    assembly_span_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_id.clone(),
                                        r.aggregation_source.clone(),
                                        min,
                                        max,
                                        r.count,
                                        r.aggregation_taxon_id.clone().unwrap_or("".to_string()),
                                        res,
                                        r.aggregation_method.clone(),
                                        r.aggregation_rank.clone().unwrap_or("".to_string()),
                                    ));
                                }
                            }

                            GoaTValueAgg::BuscoCompleteness(res) => {
                                if all || busco {
                                    let mut min: f64 = 0.0;
                                    if let MinMax::Minmaxf64(e) = r.min {
                                        min = e.unwrap_or(0.0)
                                    };
                                    let mut max: f64 = 0.0;
                                    if let MinMax::Minmaxf64(e) = r.max {
                                        max = e.unwrap_or(0.0)
                                    };

                                    busco_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_id.clone(),
                                        r.aggregation_source.clone(),
                                        min,
                                        max,
                                        r.count,
                                        r.aggregation_taxon_id.clone().unwrap_or("".to_string()),
                                        res,
                                        r.aggregation_method.clone(),
                                        r.aggregation_rank.clone().unwrap_or("".to_string()),
                                    ));
                                }
                            }
                            GoaTValueAgg::ChromosomeNumber(res) => {
                                if all || karyotype {
                                    let mut min: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.min {
                                        min = e.unwrap_or(0)
                                    };
                                    let mut max: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.max {
                                        max = e.unwrap_or(0)
                                    };

                                    chromsome_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_id.clone(),
                                        r.aggregation_source.clone(),
                                        min,
                                        max,
                                        r.count,
                                        r.aggregation_taxon_id.clone().unwrap_or("".to_string()),
                                        res,
                                        r.aggregation_method.clone(),
                                        r.aggregation_rank.clone().unwrap_or("".to_string()),
                                    ));
                                }
                            }
                            GoaTValueAgg::CValue(res) => {
                                if all || cvalues {
                                    let mut min: f64 = 0.0;
                                    if let MinMax::Minmaxf64(e) = r.min {
                                        min = e.unwrap_or(0.0)
                                    };
                                    let mut max: f64 = 0.0;
                                    if let MinMax::Minmaxf64(e) = r.max {
                                        max = e.unwrap_or(0.0)
                                    };

                                    c_value_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_id.clone(),
                                        r.aggregation_source.clone(),
                                        min,
                                        max,
                                        r.count,
                                        r.aggregation_taxon_id.clone().unwrap_or("".to_string()),
                                        res,
                                        r.aggregation_method.clone(),
                                        r.aggregation_rank.clone().unwrap_or("".to_string()),
                                    ));
                                }
                            }
                            GoaTValueAgg::GenomeSize(res) => {
                                if all || gs {
                                    let mut min: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.min {
                                        min = e.unwrap_or(0)
                                    };
                                    let mut max: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.max {
                                        max = e.unwrap_or(0)
                                    };

                                    genome_size_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_id.clone(),
                                        r.aggregation_source.clone(),
                                        min,
                                        max,
                                        r.count,
                                        r.aggregation_taxon_id.clone().unwrap_or("".to_string()),
                                        res,
                                        r.aggregation_method.clone(),
                                        r.aggregation_rank.clone().unwrap_or("".to_string()),
                                    ));
                                }
                            }
                            GoaTValueAgg::Haploid(res) => {
                                if all || karyotype {
                                    let mut min: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.min {
                                        min = e.unwrap_or(0)
                                    };
                                    let mut max: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.max {
                                        max = e.unwrap_or(0)
                                    };

                                    haploid_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_id.clone(),
                                        r.aggregation_source.clone(),
                                        min,
                                        max,
                                        r.count,
                                        r.aggregation_taxon_id.clone().unwrap_or("".to_string()),
                                        res,
                                        r.aggregation_method.clone(),
                                        r.aggregation_rank.clone().unwrap_or("".to_string()),
                                    ));
                                }
                            }
                            GoaTValueAgg::MitochondrionAssemblySpan(res) => {
                                if all || mitochondrion {
                                    let mut min: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.min {
                                        min = e.unwrap_or(0)
                                    };
                                    let mut max: u64 = 0;
                                    if let MinMax::Minmaxu64(e) = r.max {
                                        max = e.unwrap_or(0)
                                    };

                                    mitochondrion_assembly_span_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_id.clone(),
                                        r.aggregation_source.clone(),
                                        min,
                                        max,
                                        r.count,
                                        r.aggregation_taxon_id.clone().unwrap_or("".to_string()),
                                        res,
                                        r.aggregation_method.clone(),
                                        r.aggregation_rank.clone().unwrap_or("".to_string()),
                                    ));
                                }
                            }
                            GoaTValueAgg::MitochondrionGCPercent(res) => {
                                if all || mitochondrion {
                                    let mut min: f64 = 0.0;
                                    if let MinMax::Minmaxf64(e) = r.min {
                                        min = e.unwrap_or(0.0)
                                    };
                                    let mut max: f64 = 0.0;
                                    if let MinMax::Minmaxf64(e) = r.max {
                                        max = e.unwrap_or(0.0)
                                    };

                                    mitochondrion_gc_percent_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_id.clone(),
                                        r.aggregation_source.clone(),
                                        min,
                                        max,
                                        r.count,
                                        r.aggregation_taxon_id.clone().unwrap_or("".to_string()),
                                        res,
                                        r.aggregation_method.clone(),
                                        r.aggregation_rank.clone().unwrap_or("".to_string()),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Could not create output - {}", e),
        };
    }
    // these prints could be shortened into a function call.
    if all || assembly {
        println!("--- Assembly Levels ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_taxid",
            "aggregation_source",
            "min",
            "max",
            "count",
            "aggregation_taxon_id",
            "assembly_level",
            "aggregation_method",
            "aggregation_rank"
        );
        for el in assembly_level_vec {
            println!(
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                el.0, el.1, el.2, el.3, el.4, el.5, el.6, el.7, el.8, el.9, el.10
            )
        }
    }
    if all || assembly {
        println!("--- Assembly Spans ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_taxid",
            "aggregation_source",
            "min",
            "max",
            "count",
            "aggregation_taxon_id",
            "assembly_span",
            "aggregation_method",
            "aggregation_rank"
        );
        for el in assembly_span_vec {
            println!(
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                el.0, el.1, el.2, el.3, el.4, el.5, el.6, el.7, el.8, el.9, el.10
            )
        }
    }
    if all || busco {
        println!("--- BUSCO Completeness ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_taxid",
            "aggregation_source",
            "min",
            "max",
            "count",
            "aggregation_taxon_id",
            "busco_completeness",
            "aggregation_method",
            "aggregation_rank"
        );

        for el in busco_vec {
            println!(
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                el.0, el.1, el.2, el.3, el.4, el.5, el.6, el.7, el.8, el.9, el.10
            )
        }
    }
    if all || gs {
        println!("--- Genome Size ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_taxid",
            "aggregation_source",
            "min",
            "max",
            "count",
            "aggregation_taxon_id",
            "genome_size",
            "aggregation_method",
            "aggregation_rank"
        );
        for el in genome_size_vec {
            println!(
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                el.0, el.1, el.2, el.3, el.4, el.5, el.6, el.7, el.8, el.9, el.10
            )
        }
    }
    if all || karyotype {
        println!("--- Chromosome Numbers ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_taxid",
            "aggregation_source",
            "min",
            "max",
            "count",
            "aggregation_taxon_id",
            "chromosome_number",
            "aggregation_method",
            "aggregation_rank"
        );
        for el in chromsome_vec {
            println!(
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                el.0, el.1, el.2, el.3, el.4, el.5, el.6, el.7, el.8, el.9, el.10
            )
        }
    }
    if all || cvalues {
        println!("--- C-Values ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_taxid",
            "aggregation_source",
            "min",
            "max",
            "count",
            "aggregation_taxon_id",
            "c_value",
            "aggregation_method",
            "aggregation_rank"
        );
        for el in c_value_vec {
            println!(
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                el.0, el.1, el.2, el.3, el.4, el.5, el.6, el.7, el.8, el.9, el.10
            )
        }
    }
    if all || karyotype {
        println!("--- Haploid Chromosome Number ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_taxid",
            "aggregation_source",
            "min",
            "max",
            "count",
            "aggregation_taxon_id",
            "haploid_chromosome_number",
            "aggregation_method",
            "aggregation_rank"
        );
        for el in haploid_vec {
            println!(
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                el.0, el.1, el.2, el.3, el.4, el.5, el.6, el.7, el.8, el.9, el.10
            )
        }
    }

    if all || mitochondrion {
        println!("--- Mitochondrial Assembly Span ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_taxid",
            "aggregation_source",
            "min",
            "max",
            "count",
            "aggregation_taxon_id",
            "mitochondrial_assembly_span",
            "aggregation_method",
            "aggregation_rank"
        );
        for el in mitochondrion_assembly_span_vec {
            println!(
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                el.0, el.1, el.2, el.3, el.4, el.5, el.6, el.7, el.8, el.9, el.10
            )
        }
    }

    if all || mitochondrion {
        println!("--- Mitochondrial GC Percent ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            ranks_vec,
            "taxon_name",
            "ncbi_taxid",
            "aggregation_source",
            "min",
            "max",
            "count",
            "aggregation_taxon_id",
            "mitochondrial_gc_percent",
            "aggregation_method",
            "aggregation_rank"
        );
        for el in mitochondrion_gc_percent_vec {
            println!(
                "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                el.0, el.1, el.2, el.3, el.4, el.5, el.6, el.7, el.8, el.9, el.10
            )
        }
    }

    Ok(())
}
