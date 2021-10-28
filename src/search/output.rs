use crate::search::agg_values::Records;
use crate::search::raw_values::RawRecords;
use crate::utils::ranks::RankHeaders;

use anyhow::{bail, Result};

#[derive(Clone)]
pub struct CombinedValues {
    pub raw: Option<RawRecords>,
    pub agg: Option<Records>,
}

// TODO: add headers for ranks.

pub fn print_raw_output(
    awaited_fetches: Vec<Result<CombinedValues, anyhow::Error>>,
    all: bool,
    assembly: bool,
    gs: bool,
    cvalues: bool,
    karyotype: bool,
    busco: bool,
    ranks_vec: RankHeaders,
) -> Result<()> {
    // this may be the jankiest work-around to date...
    // I have to allocate each field of AggFetches
    // into a separate vector.
    // I need to figure out a more elegant way to do this.

    let mut assembly_span_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                assembly_span_vec.push(e.raw.clone().unwrap().span.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut assembly_level_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                assembly_level_vec.push(e.raw.clone().unwrap().level.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut c_value_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                c_value_vec.push(e.raw.clone().unwrap().c_value.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut chromosome_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                chromosome_vec.push(e.raw.clone().unwrap().chromosome_number.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut genome_sizes_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                genome_sizes_vec.push(e.raw.clone().unwrap().genome_size.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut haploid_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                haploid_vec.push(e.raw.clone().unwrap().haploid.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut busco_completeness_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                busco_completeness_vec.push(e.raw.clone().unwrap().busco_completeness.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    if all || assembly {
        println!("--- Assembly Level ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_taxid", "source_id", "source", "assembly_type"
        );

        for el in assembly_level_vec {
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}\t{}",
                    el2.ranks, el2.taxon_name, el2.taxon_ncbi, el2.source_id, el2.source, el2.value,
                );
            }
        }
    }

    if all || assembly {
        println!("--- Assembly Span ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_taxid", "source_id", "source", "span"
        );

        for el in assembly_span_vec {
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}\t{}",
                    el2.ranks, el2.taxon_name, el2.taxon_ncbi, el2.source_id, el2.source, el2.value,
                );
            }
        }
    }

    if all || busco {
        println!("--- BUSCO Completeness ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_taxid", "source", "span"
        );

        for el in busco_completeness_vec {
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}",
                    el2.ranks, el2.taxon_name, el2.taxon_ncbi, el2.source, el2.value,
                );
            }
        }
    }

    if all || cvalues {
        println!("--- C-Values ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "c_value"
        );

        for el in c_value_vec {
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}",
                    el2.ranks, el2.taxon_name, el2.taxon_ncbi, el2.source, el2.value
                )
            }
        }
    }

    if all || karyotype {
        println!("--- Chromosome Numbers ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "chromosome_number"
        );
        for el in chromosome_vec {
            for el2 in el {
                let mut comma_sep_chroms = String::new();
                let mut peek_chrom_vec = el2.value.iter().peekable();

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

                println!(
                    "{}{}\t{}\t{}\t{}",
                    el2.ranks, el2.taxon_name, el2.taxon_ncbi, el2.source, comma_sep_chroms
                )
            }
        }
    }

    if all || gs {
        println!("--- Genome Sizes ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "genome_size"
        );
        for el in genome_sizes_vec {
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}",
                    el2.ranks, el2.taxon_name, el2.taxon_ncbi, el2.source, el2.value
                )
            }
        }
    }

    if all || karyotype {
        println!("--- Haploid Numbers ---");
        println!(
            "{}{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "haploid_chromosome_number"
        );
        for el in haploid_vec {
            for el2 in el {
                let mut comma_sep_haps = String::new();
                let mut peek_chrom_vec = el2.value.iter().peekable();

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
                println!(
                    "{}{}\t{}\t{}\t{}",
                    el2.ranks, el2.taxon_name, el2.taxon_ncbi, el2.source, comma_sep_haps
                )
            }
        }
    }
    Ok(())
}

pub fn print_agg_output(
    awaited_fetches: Vec<Result<CombinedValues, anyhow::Error>>,
    all: bool,
    assembly: bool,
    gs: bool,
    cvalues: bool,
    karyotype: bool,
    busco: bool,
    ranks_vec: RankHeaders,
) -> Result<()> {
    let mut assembly_span_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                assembly_span_vec.push(e.agg.clone().unwrap().span.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut assembly_level_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                assembly_level_vec.push(e.agg.clone().unwrap().level.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut busco_completeness_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                busco_completeness_vec.push(e.agg.clone().unwrap().busco_completeness.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut cvalue_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                cvalue_vec.push(e.agg.clone().unwrap().c_value.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut chromosome_number_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                chromosome_number_vec.push(e.agg.clone().unwrap().chromosome_number.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut genome_size_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                genome_size_vec.push(e.agg.clone().unwrap().genome_size.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
    }

    let mut haploid_vec = Vec::new();
    for el in &awaited_fetches {
        let _ = match el {
            Ok(e) => {
                // argh more cloning...
                haploid_vec.push(e.agg.clone().unwrap().haploid.clone());
            }
            Err(e) => bail!("[-]\tSomething went wrong? {}", e),
        };
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
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    el2.ranks,
                    el2.taxon_name,
                    el2.taxon_id,
                    el2.aggregation_source,
                    el2.min.unwrap_or(0),
                    el2.max.unwrap_or(0),
                    el2.count,
                    el2.aggregation_taxon_id.unwrap_or("".to_string()),
                    el2.value,
                    el2.aggregation_method,
                    el2.aggregation_rank.unwrap_or("".to_string()),
                );
            }
        }
    }

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
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    el2.ranks,
                    el2.taxon_name,
                    el2.taxon_id,
                    el2.aggregation_source,
                    el2.min.unwrap_or(0),
                    el2.max.unwrap_or(0),
                    el2.count,
                    el2.aggregation_taxon_id.unwrap_or("".to_string()),
                    el2.value,
                    el2.aggregation_method,
                    el2.aggregation_rank.unwrap_or("".to_string()),
                );
            }
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

        for el in busco_completeness_vec {
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    el2.ranks,
                    el2.taxon_name,
                    el2.taxon_id,
                    el2.aggregation_source,
                    el2.min.unwrap_or(0.0),
                    el2.max.unwrap_or(0.0),
                    el2.count,
                    el2.aggregation_taxon_id.unwrap_or("".to_string()),
                    el2.value,
                    el2.aggregation_method,
                    el2.aggregation_rank.unwrap_or("".to_string()),
                );
            }
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

        for el in cvalue_vec {
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    el2.ranks,
                    el2.taxon_name,
                    el2.taxon_id,
                    el2.aggregation_source,
                    el2.min.unwrap_or(0.0),
                    el2.max.unwrap_or(0.0),
                    el2.count,
                    el2.aggregation_taxon_id.unwrap_or("".to_string()),
                    el2.value,
                    el2.aggregation_method,
                    el2.aggregation_rank.unwrap_or("".to_string()),
                );
            }
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

        for el in chromosome_number_vec {
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    el2.ranks,
                    el2.taxon_name,
                    el2.taxon_id,
                    el2.aggregation_source,
                    el2.min.unwrap_or(0),
                    el2.max.unwrap_or(0),
                    el2.count,
                    el2.aggregation_taxon_id.unwrap_or("".to_string()),
                    el2.value,
                    el2.aggregation_method,
                    el2.aggregation_rank.unwrap_or("".to_string()),
                );
            }
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
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    el2.ranks,
                    el2.taxon_name,
                    el2.taxon_id,
                    el2.aggregation_source,
                    el2.min.unwrap_or(0),
                    el2.max.unwrap_or(0),
                    el2.count,
                    el2.aggregation_taxon_id.unwrap_or("".to_string()),
                    el2.value,
                    el2.aggregation_method,
                    el2.aggregation_rank.unwrap_or("".to_string()),
                );
            }
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
            for el2 in el {
                println!(
                    "{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    el2.ranks,
                    el2.taxon_name,
                    el2.taxon_id,
                    el2.aggregation_source,
                    el2.min.unwrap_or(0),
                    el2.max.unwrap_or(0),
                    el2.count,
                    el2.aggregation_taxon_id.unwrap_or("".to_string()),
                    el2.value,
                    el2.aggregation_method,
                    el2.aggregation_rank.unwrap_or("".to_string()),
                );
            }
        }
    }

    Ok(())
}
