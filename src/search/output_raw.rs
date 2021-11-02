use crate::search::combine_output::CombinedValues;
use crate::search::raw_values::GoaTValue;
use crate::utils::ranks::RankHeaders;
use crate::utils::url::FieldBuilder;

use anyhow::Result;

pub fn print_raw_output(
    awaited_fetches: Vec<Result<CombinedValues, anyhow::Error>>,
    fields: FieldBuilder,
    ranks_vec: RankHeaders,
) -> Result<()> {
    // still have to collect into Vec's and iterate later.
    // some/most of these will remain empty depending on the query.
    // That's okay. Making a Vec is super cheap.
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
    let mut raw_plastid_assembly_span_vec = Vec::new();
    let mut raw_plastid_gc_percent_vec = Vec::new();

    // decompose FieldBuilder
    let all = fields.all;
    let assembly = fields.assembly;
    let busco = fields.busco;
    let cvalues = fields.cvalues;
    let gs = fields.gs;
    let karyotype = fields.karyotype;
    let mitochondrion = fields.mitochondrion;
    let plastid = fields.plastid;

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
                            GoaTValue::RawPlastidAssemblySpan(res) => {
                                if all || plastid {
                                    raw_plastid_assembly_span_vec.push((
                                        r.ranks.clone(),
                                        r.taxon_name.clone(),
                                        r.taxon_ncbi.clone(),
                                        r.source_id.clone(),
                                        r.source.clone(),
                                        res,
                                    ));
                                }
                            }
                            GoaTValue::RawPlastidGCPercent(res) => {
                                if all || plastid {
                                    raw_plastid_gc_percent_vec.push((
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
            Err(e) => eprintln!("{}", e),
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
    if all || plastid {
        println!("--- Plastid Assembly Span ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "source_id", "plastid_assembly_span"
        );
        for el in raw_plastid_assembly_span_vec {
            println!("{}{}\t{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4, el.5)
        }
    }
    if all || plastid {
        println!("--- Plastid GC Percent ---");
        println!(
            "{}{}\t{}\t{}\t{}\t{}",
            ranks_vec, "taxon_name", "ncbi_id", "source", "source_id", "plastid_gc_percent"
        );
        for el in raw_plastid_gc_percent_vec {
            println!("{}{}\t{}\t{}\t{}\t{}", el.0, el.1, el.2, el.3, el.4, el.5)
        }
    }

    Ok(())
}
