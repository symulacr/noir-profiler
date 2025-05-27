mod core;
mod analyzer;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::time::Instant;
use tabular::{Row, Table};
use std::fs::File;
use std::io::Write;

use noir_circuit_profiler::analyzer::{analyze_circuit, batch_analyze, compare_circuits};
use noir_circuit_profiler::core::CircuitAnalysis;

#[derive(Parser)]
#[clap(version = "1.0", author = "Noir Team")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Analyze { 
        file: PathBuf,
        
        #[clap(short, long, default_value = "text")]
        format: String,
    },
    
    Compare {
        file1: PathBuf,
        
        file2: PathBuf,
    },
    
    Batch {
        dir: PathBuf,
    },

    Stats {
        dir: PathBuf,
    },
    
    Calibrate {
        #[clap(short, long)]
        dir: PathBuf,
        
        #[clap(short, long)]
        reset: bool,
    },
    
    Help,
}

fn main() -> Result<()> {
    print_banner();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Analyze { file, format }) => {
            let start = Instant::now();
            let analysis = analyze_circuit(&file)
                .context("Failed to analyze circuit")?;
            
            let duration = start.elapsed();
            println!("{} Analyzed in {:.2?}", "OK".green().bold(), duration);
            
            match format.as_str() {
                "json" => print_json(&analysis)?,
                _ => {
                    print_core_metrics(&analysis, &file);
                    print_function_analysis(&analysis);
                    print_structure_analysis(&analysis);
                    print_constraint_details(&analysis);
                    
                    println!("\n{} This is an experimental demo version", "[NOTE]".on_cyan().black().bold());
                }
            }
        },
        Some(Commands::Compare { file1, file2 }) => {
            print_comparison(&file1, &file2)?;
        },
        Some(Commands::Batch { dir }) => {
            let results = batch_analyze(&dir)
                .context("Failed to analyze directory")?;
            
            println!("\n{} Batch Analysis Results:", "[BATCH]".on_magenta().white().bold());
            
            let mut table = Table::new("{:<}  {:<}  {:<}  {:<}");
            table.add_row(Row::new()
                .with_cell("Circuit".bright_white().bold())
                .with_cell("Constraints".bright_white().bold())
                .with_cell("Opcodes".bright_white().bold())
                .with_cell("Constraint/Opcode".bright_white().bold()));
            
            table.add_row(Row::new()
                .with_cell("─".repeat(30))
                .with_cell("─".repeat(15))
                .with_cell("─".repeat(15))
                .with_cell("─".repeat(20)));
            
            for (name, result) in results {
                match result {
                    Ok(analysis) => {
                        let constraint_per_op = if analysis.total_opcodes > 0 {
                            analysis.constraints as f64 / analysis.total_opcodes as f64
                        } else {
                            0.0
                        };
                            
                        table.add_row(Row::new()
                            .with_cell(name.cyan())
                            .with_cell(analysis.constraints.to_string().yellow())
                            .with_cell(analysis.total_opcodes.to_string())
                            .with_cell(format!("{:.1}x", constraint_per_op).green()));
                    },
                    Err(e) => {
                        table.add_row(Row::new()
                            .with_cell(name)
                            .with_cell("ERROR".red())
                            .with_cell("-")
                            .with_cell(e.to_string().red()));
                    }
                }
            }
            
            println!("{}", table);
        },
        Some(Commands::Stats { dir }) => {
            let results = batch_analyze(&dir)
                .context("Failed to analyze directory")?;
            
            println!("\n{} Research Statistics Collection:", "[STATS]".on_cyan().black().bold());
            println!("Collecting detailed metrics from {} circuits...", results.len());
            
            println!("\n# NOIR PROFILER STATISTICS DATA - EXCEL/CSV FORMAT");
            println!("# Generated on {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
            println!("# Directory: {}", dir.display());
            println!("# NOTE: This is an experimental demo version\n");
            
            println!("Circuit,Constraints,Opcodes,ExternalOps,PublicInputs,PrivateInputs,OutputCount,AvgCostPerOp");
            
            for (name, result) in results {
                match result {
                    Ok(analysis) => {
                        let avg_cost = if analysis.total_opcodes > 0 {
                            analysis.constraints as f64 / analysis.total_opcodes as f64
                        } else {
                            0.0
                        };
                        
                        let external_ops = analysis.black_box_functions.len();
                        
                        println!("{},{},{},{},{},{},{},{:.2}", 
                            name,
                            analysis.constraints,
                            analysis.total_opcodes,
                            external_ops,
                            analysis.public_inputs,
                            analysis.private_inputs,
                            analysis.return_values,
                            avg_cost
                        );
                        
                        collect_detailed_stats(&name, &analysis);
                    },
                    Err(_) => continue
                }
            }
            
            println!("\n# Statistics collection complete");
            println!("# Copy the data above for Excel/CSV analysis");
        },
        Some(Commands::Calibrate { dir, reset }) => {
            println!("\n{} Cost Model Calibration:", "[CALIBRATE]".on_magenta().white().bold());
            
            if reset {
                std::fs::remove_file("circuit_stats/cost_database.json").ok();
                println!("✓ Reset cost database to defaults");
            }
            
            println!("Calibrating cost models using circuits in: {}", dir.display());
            
            let results = batch_analyze(&dir)
                .context("Failed to analyze directory")?;
            
            let successful = results.iter().filter(|(_, r)| r.is_ok()).count();
            println!("\n{} Cost model calibration complete", "✓".green().bold());
            println!("Processed {} circuits ({} successful)", results.len(), successful);
            
            print_cost_database();
        },
        Some(Commands::Help) => {
            print_help();
        },
        None => {
            println!("{} No command specified. Use --help for usage information.", "ERROR".on_red().white());
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn collect_detailed_stats(name: &str, analysis: &CircuitAnalysis) {
    std::fs::create_dir_all("circuit_stats").unwrap_or(());
    
    let filename = format!("circuit_stats/{}.csv", name.replace(".json", ""));
    let mut file = File::create(filename).unwrap_or_else(|_| {
        File::create(format!("circuit_stats/circuit_{}.csv", rand::random::<u32>())).unwrap()
    });
    
    writeln!(file, "# NOIR PROFILER CIRCUIT ANALYSIS: {}", name).unwrap();
    writeln!(file, "# Generated on {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).unwrap();
    writeln!(file, "# NOTE: This is an experimental demo version\n").unwrap();
    
    writeln!(file, "METRIC,VALUE").unwrap();
    writeln!(file, "Constraints,{}", analysis.constraints).unwrap();
    writeln!(file, "Opcodes,{}", analysis.total_opcodes).unwrap();
    writeln!(file, "Public Inputs,{}", analysis.public_inputs).unwrap();
    writeln!(file, "Private Inputs,{}", analysis.private_inputs).unwrap();
    writeln!(file, "Return Values,{}", analysis.return_values).unwrap();
    
    writeln!(file, "\nOPERATION,COUNT").unwrap();
    for (op, count) in &analysis.operation_counts {
        writeln!(file, "{},{}", op, count).unwrap();
    }
    
    if !analysis.black_box_functions.is_empty() {
        writeln!(file, "\nEXTERNAL_OPERATION,CALLS,CONSTRAINTS_EACH").unwrap();
        for (name, count, cost) in &analysis.black_box_functions {
            writeln!(file, "{},{},{}", name, count, cost).unwrap();
        }
    }
    
    let mut bb_constraints = 0;
    for (_, count, cost) in &analysis.black_box_functions {
        bb_constraints += count * cost;
    }
    
    let mut arithmetic_constraints = 0;
    for (op_type, count) in &analysis.operation_counts {
        if op_type.contains("Assert") || op_type.contains("Arithmetic") {
            arithmetic_constraints += count;
        }
    }
    
    let other_constraints = analysis.constraints - bb_constraints - arithmetic_constraints;
    
    writeln!(file, "\nCATEGORY,CONSTRAINTS,PERCENTAGE").unwrap();
    if bb_constraints > 0 {
        let percent = (bb_constraints as f64 / analysis.constraints as f64) * 100.0;
        writeln!(file, "External Operations,{},{:.1}%", bb_constraints, percent).unwrap();
    }
    if arithmetic_constraints > 0 {
        let percent = (arithmetic_constraints as f64 / analysis.constraints as f64) * 100.0;
        writeln!(file, "Arithmetic Operations,{},{:.1}%", arithmetic_constraints, percent).unwrap();
    }
    if other_constraints > 0 {
        let percent = (other_constraints as f64 / analysis.constraints as f64) * 100.0;
        writeln!(file, "Other Operations,{},{:.1}%", other_constraints, percent).unwrap();
    }
}

fn print_core_metrics(analysis: &CircuitAnalysis, file: &PathBuf) {
    println!("\n{} Circuit Analysis: {}", "[METRICS]".on_blue().white().bold(), file.display().to_string().cyan().underline());
    
    println!("╭───────────────────────────────────────────────────╮");
    
    let mut table = Table::new("{:<}  {:<}");
    table.add_row(Row::new()
        .with_cell("Metric".bright_white().bold())
        .with_cell("Value".bright_white().bold()));
    
    table.add_row(Row::new()
        .with_cell("Total Constraints")
        .with_cell(format!("{}", analysis.constraints).yellow().bold()));
        
    table.add_row(Row::new()
        .with_cell("Total ACIR Opcodes")
        .with_cell(format!("{}", analysis.total_opcodes).cyan()));
        
    table.add_row(Row::new()
        .with_cell("Public Inputs")
        .with_cell(format!("{}", analysis.public_inputs).magenta()));
        
    table.add_row(Row::new()
        .with_cell("Private Inputs")
        .with_cell(format!("{}", analysis.private_inputs).magenta()));
        
    table.add_row(Row::new()
        .with_cell("Input/Output Count")
        .with_cell(format!("{} in / {} out", analysis.public_inputs + analysis.private_inputs, analysis.return_values).green().bold()));
    
    let proving_time = analysis.estimated_proving_time;
    let time_display = if proving_time < 1.0 {
        format!("{:.2}ms", proving_time).green()
    } else if proving_time < 100.0 {
        format!("{:.2}ms", proving_time).yellow()
    } else if proving_time < 1000.0 {
        format!("{:.2}ms", proving_time).red()
    } else {
        format!("{:.2}s", proving_time / 1000.0).red().bold()
    };
    
    table.add_row(Row::new()
        .with_cell("Est. Proving Time")
        .with_cell(time_display));
    
    if analysis.constraints > 0 {
        let efficiency = analysis.estimated_proving_time / analysis.constraints as f64 * 1000.0;
        table.add_row(Row::new()
            .with_cell("Proving Efficiency")
            .with_cell(format!("{:.3} μs/constraint", efficiency).cyan()));
    }
    
    println!("│ {}│", table.to_string().replace("\n", "\n│ "));
    println!("╰───────────────────────────────────────────────────╯");
    
    println!("\n{} Proving time estimates vary by hardware configuration", "[NOTE]".on_cyan().black());
}

fn print_function_analysis(analysis: &CircuitAnalysis) {
    if analysis.black_box_functions.is_empty() {
        return;
    }
    
    println!("\n{} External Operations Analysis:", "[FUNCTIONS]".on_red().white().bold());
    
    let black_box_constraints: usize = analysis.black_box_functions
        .iter()
        .map(|(_, count, cost)| count * cost)
        .sum();
    
    let percent = if analysis.constraints > 0 {
        (black_box_constraints as f64 / analysis.constraints as f64) * 100.0
    } else {
        0.0
    };
    
    println!("╭────────────────────────────────────────────────────────────╮");
    
    let mut table = Table::new("{:<}  {:<}  {:<}  {:<}");
    table.add_row(Row::new()
        .with_cell("Operation".bright_white().bold())
        .with_cell("Calls".bright_white().bold())
        .with_cell("Constraints".bright_white().bold())
        .with_cell("% Circuit".bright_white().bold()));
    
    table.add_row(Row::new()
        .with_cell("────────────────────")
        .with_cell("──────────")
        .with_cell("──────────")
        .with_cell("──────────"));
    
    for (name, count, cost) in &analysis.black_box_functions {
        let total_cost = count * cost;
        let func_percent = if analysis.constraints > 0 {
            (total_cost as f64 / analysis.constraints as f64) * 100.0
        } else {
            0.0
        };
        
        let percent_cell = if func_percent > 20.0 {
            format!("{:.1}%", func_percent).red().bold()
        } else if func_percent > 10.0 {
            format!("{:.1}%", func_percent).yellow()
        } else {
            format!("{:.1}%", func_percent).green()
        };
        
        table.add_row(Row::new()
            .with_cell(name.cyan())
            .with_cell(count.to_string())
            .with_cell(total_cost.to_string().yellow())
            .with_cell(percent_cell));
    }
    
    println!("│ {}│", table.to_string().replace("\n", "\n│ "));
    
    println!("╰────────────────────────────────────────────────────────────╯");
    
    if percent > 0.0 {
        println!("\n{}: External operations account for {:.1}% of total constraints", 
                "[INSIGHT]".on_yellow().black().bold(),
                percent);
    }
}

fn print_function_comparison(analysis1: &CircuitAnalysis, analysis2: &CircuitAnalysis) {
    println!("\n{} External Operations Comparison:", "[FUNCTIONS]".on_red().white().bold());
    
    let mut all_functions = Vec::new();
    for (name, _, _) in &analysis1.black_box_functions {
        if !all_functions.contains(name) {
            all_functions.push(name.clone());
        }
    }
    
    for (name, _, _) in &analysis2.black_box_functions {
        if !all_functions.contains(name) {
            all_functions.push(name.clone());
        }
    }
    
    println!("╭───────────────────────────────────────────────────────────────╮");
    
    let mut table = Table::new("{:<}  {:<}  {:<}  {:<}");
    table.add_row(Row::new()
        .with_cell("Operation".bright_white().bold())
        .with_cell("Circuit 1".bright_white().bold())
        .with_cell("Circuit 2".bright_white().bold())
        .with_cell("Diff".bright_white().bold()));
    
    table.add_row(Row::new()
        .with_cell("────────────────────")
        .with_cell("──────────")
        .with_cell("──────────")
        .with_cell("──────────"));
    
    for func_name in all_functions {
        let count1 = analysis1.black_box_functions
            .iter()
            .find(|(name, _, _)| name == &func_name)
            .map(|(_, count, _)| *count)
            .unwrap_or(0);
            
        let count2 = analysis2.black_box_functions
            .iter()
            .find(|(name, _, _)| name == &func_name)
            .map(|(_, count, _)| *count)
            .unwrap_or(0);
            
        let diff = count2 as i64 - count1 as i64;
        
        table.add_row(Row::new()
            .with_cell(func_name.cyan())
            .with_cell(count1.to_string())
            .with_cell(count2.to_string())
            .with_cell(format_signed_number(diff)));
    }
    
    println!("│ {}│", table.to_string().replace("\n", "\n│ "));
    println!("╰───────────────────────────────────────────────────────────────╯");
}

fn print_structure_analysis(analysis: &CircuitAnalysis) {
    if analysis.operation_counts.is_empty() {
        return;
    }
    
    println!("\n{} Circuit Structure Analysis:", "[STRUCTURE]".on_green().black().bold());
    
    println!("╭───────────────────────────────────────────────────╮");
    
    let mut table = Table::new("{:<}  {:<}  {:<}");
    table.add_row(Row::new()
        .with_cell("Operation Type".bright_white().bold())
        .with_cell("Count".bright_white().bold())
        .with_cell("% of Opcodes".bright_white().bold()));
    
    table.add_row(Row::new()
        .with_cell("────────────────────")
        .with_cell("──────────")
        .with_cell("────────────"));
    
    let sorted_ops = &analysis.operation_counts;
    let display_count = std::cmp::min(8, sorted_ops.len());
    
    for (op_type, count) in sorted_ops.iter().take(display_count) {
        let percent = if analysis.total_opcodes > 0 {
            (*count as f64 / analysis.total_opcodes as f64) * 100.0
        } else {
            0.0
        };
        
        let percent_cell = if percent > 50.0 {
            format!("{:.1}%", percent).red().bold()
        } else if percent > 20.0 {
            format!("{:.1}%", percent).yellow()
        } else {
            format!("{:.1}%", percent).green()
        };
        
        table.add_row(Row::new()
            .with_cell(op_type.cyan())
            .with_cell(count.to_string())
            .with_cell(percent_cell));
    }
    
    println!("│ {}│", table.to_string().replace("\n", "\n│ "));
    println!("╰───────────────────────────────────────────────────╯");
    
    let has_memory_ops = analysis.operation_counts
        .iter()
        .any(|(op, _)| op.contains("Memory"));
        
    println!("\n{}: {}", 
             "[INSIGHT]".on_yellow().black().bold(),
             if has_memory_ops {
                 "Circuit uses memory operations, suggesting array or structured data usage".italic()
             } else {
                 "No memory operations detected, suggesting primarily scalar field operations".italic()
             });
}

fn print_constraint_details(analysis: &CircuitAnalysis) {
    println!("\n{} Constraint Distribution:", "[DETAILS]".on_blue().white().bold());
    
    if analysis.constraints == 0 {
        println!("No constraints detected in circuit.");
        return;
    }
    
    let mut categories = std::collections::HashMap::new();
    
    let mut bb_constraints = 0;
    for (_, count, cost) in &analysis.black_box_functions {
        bb_constraints += count * cost;
    }
    
    if bb_constraints > 0 {
        categories.insert("External Operations", bb_constraints);
    }
    
    let mut arithmetic_constraints = 0;
    for (op_type, count) in &analysis.operation_counts {
        if op_type.contains("Assert") || op_type.contains("Arithmetic") {
            arithmetic_constraints += count;
        }
    }
    
    if arithmetic_constraints > 0 {
        categories.insert("Arithmetic Operations", arithmetic_constraints);
    }
    
    let other_constraints = analysis.constraints - bb_constraints - arithmetic_constraints;
    if other_constraints > 0 {
        categories.insert("Other Operations", other_constraints);
    }
    
    println!("╭───────────────────────────────────────────────────╮");
    
    let mut table = Table::new("{:<}  {:<}  {:<}");
    table.add_row(Row::new()
        .with_cell("Category".bright_white().bold())
        .with_cell("Constraints".bright_white().bold())
        .with_cell("% of Total".bright_white().bold()));
    
    table.add_row(Row::new()
        .with_cell("────────────────────")
        .with_cell("────────────")
        .with_cell("────────────"));
    
    let mut category_vec: Vec<_> = categories.iter().collect();
    category_vec.sort_by(|a, b| b.1.cmp(a.1));
    
    for (category, count) in category_vec {
        let percent = (*count as f64 / analysis.constraints as f64) * 100.0;
        
        let percent_cell = if percent > 50.0 {
            format!("{:.1}%", percent).red().bold()
        } else if percent > 20.0 {
            format!("{:.1}%", percent).yellow()
        } else {
            format!("{:.1}%", percent).green()
        };
        
        table.add_row(Row::new()
            .with_cell(category.cyan())
            .with_cell(count.to_string().yellow())
            .with_cell(percent_cell));
    }
    
    println!("│ {}│", table.to_string().replace("\n", "\n│ "));
    println!("╰───────────────────────────────────────────────────╯");
}

fn print_json(analysis: &CircuitAnalysis) -> Result<()> {
    let json = serde_json::to_string_pretty(analysis)
        .context("Failed to serialize analysis")?;
    println!("{}", json.cyan());
    Ok(())
}

fn format_signed_number(num: i64) -> colored::ColoredString {
    if num < 0 {
        format!("-{}", num.abs()).red().bold()
    } else if num > 0 {
        format!("+{}", num).green().bold()
    } else {
        "0".normal()
    }
}

fn print_banner() {
    println!("{}", 
r"
  ███╗   ██╗ ██████╗ ██╗██████╗     ██████╗ ██████╗  ██████╗ ███████╗██╗██╗     ███████╗██████╗ 
  ████╗  ██║██╔═══██╗██║██╔══██╗    ██╔══██╗██╔══██╗██╔═══██╗██╔════╝██║██║     ██╔════╝██╔══██╗
  ██╔██╗ ██║██║   ██║██║██████╔╝    ██████╔╝██████╔╝██║   ██║█████╗  ██║██║     █████╗  ██████╔╝
  ██║╚██╗██║██║   ██║██║██╔══██╗    ██╔═══╝ ██╔══██╗██║   ██║██╔══╝  ██║██║     ██╔══╝  ██╔══██╗
  ██║ ╚████║╚██████╔╝██║██║  ██║    ██║     ██║  ██║╚██████╔╝██║     ██║███████╗███████╗██║  ██║
  ╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═╝  ╚═╝    ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝
"
.bright_cyan().bold());
    println!("{}", "  Circuit analysis tool - experimental demo version".bright_cyan().italic());
    println!("  {}", "────────────────────────────────────────────────────────────────────────────────".bright_cyan());
}

fn print_help() {
    println!("\n{} Noir Circuit Analysis Guide - Experimental Demo", "[HELP]".on_cyan().black().bold());
    
    println!("\n{} Creating Test Circuits:", "[CREATE]".on_green().black().bold());
    println!("  1. Write a simple Noir program");
    println!("  2. Compile with 'nargo compile'");
    println!("  3. Analyze the generated ACIR file with this tool");
    
    println!("\n{} Examples:", "[USAGE]".on_green().black().bold());
    println!("  {}  ./np.sh analyze target/main.json", "Analyze:".bright_white().bold());
    println!("  {}  ./np.sh compare circuit1.json circuit2.json", "Compare:".bright_white().bold());
    println!("  {}     ./np.sh stats circuits_dir > research_data.csv", "Research:".bright_white().bold());
    println!("  {}     ./np.sh analyze circuit.json --format json > analysis.json", "Export:".bright_white().bold());
    println!("  {}     ./np.sh calibrate --dir example_circuits", "Calibrate:".bright_white().bold());
}

fn print_comparison(file1: &PathBuf, file2: &PathBuf) -> Result<()> {
    let (analysis1, analysis2) = compare_circuits(file1, file2)
        .context("Failed to compare circuits")?;
    
    println!("\n{} Comparison Results:", "[COMPARE]".on_blue().white().bold());
    
    print_core_metrics(&analysis1, file1);
    print_core_metrics(&analysis2, file2);
    
    let diff = analysis2.constraints as i64 - analysis1.constraints as i64;
    
    println!("\n{} Circuit Size Difference: {} constraints",
        "[DIFF]".on_yellow().black().bold(),
        format_signed_number(diff));
    
    let time_diff = analysis2.estimated_proving_time - analysis1.estimated_proving_time;
    println!("{} Proving Time Impact: {} ms", 
        "[PERFORMANCE]".on_magenta().white().bold(),
        format_signed_float(time_diff));
    
    let time_per_constraint1 = if analysis1.constraints > 0 {
        analysis1.estimated_proving_time / analysis1.constraints as f64 * 1000.0
    } else { 0.0 };
    
    let time_per_constraint2 = if analysis2.constraints > 0 {
        analysis2.estimated_proving_time / analysis2.constraints as f64 * 1000.0
    } else { 0.0 };
    
    println!("\n{} Proving Efficiency:", "[EFFICIENCY]".on_cyan().black().bold());
    println!("  Circuit 1: {:.3} μs per constraint", time_per_constraint1);
    println!("  Circuit 2: {:.3} μs per constraint", time_per_constraint2);
    
    if diff.abs() > 100 {
        use crate::core::find_operations_by_cost;
        
        let matching_ops = find_operations_by_cost(diff.unsigned_abs() as usize, 5.0);
        
        if !matching_ops.is_empty() {
            println!("\n{} Potential Operations Detected:", "[ANALYSIS]".on_green().black().bold());
            
            for (op_name, cost, confidence) in matching_ops.iter().take(3) {
                let diff_percent = (*cost as f64 - diff.unsigned_abs() as f64).abs() / *cost as f64 * 100.0;
                let match_quality = if diff_percent < 1.0 {
                    "strong similarity to".yellow()
                } else if diff_percent < 3.0 {
                    "possible".cyan()
                } else {
                    "resembles".normal()
                };
                
                println!("  Circuit difference {} {} ({} constraints, {:.1}% confidence)", 
                    match_quality,
                    op_name.cyan().bold(), 
                    cost.to_string().yellow(), 
                    (confidence * 100.0));
            }
            
            println!("  Note: Actual operation costs may vary based on circuit architecture and proving system");
        }
    }
        
    if !analysis1.black_box_functions.is_empty() || !analysis2.black_box_functions.is_empty() {
        print_function_comparison(&analysis1, &analysis2);
    }
    
    Ok(())
}

fn format_signed_float(num: f64) -> colored::ColoredString {
    if num < 0.0 {
        format!("-{:.2}", num.abs()).red().bold()
    } else if num > 0.0 {
        format!("+{:.2}", num).green().bold()
    } else {
        "0.00".normal()
    }
}

fn print_cost_database() {
    use crate::core::{get_cost_database, apply_real_world_variability};
    
    let db = get_cost_database();
    
    println!("\n{} COST MODEL DATABASE:", "[MODEL]".on_blue().white().bold());
    
    println!("╭─────────────────────────────────────────────────────────────────╮");
    
    let mut table = Table::new("{:<}  {:<}  {:<}  {:<}  {:<}");
    table.add_row(Row::new()
        .with_cell("Operation".bright_white().bold())
        .with_cell("Avg. Cost".bright_white().bold())
        .with_cell("Recent Samples".bright_white().bold())
        .with_cell("Confidence".bright_white().bold())
        .with_cell("Sample Count".bright_white().bold()));
    
    table.add_row(Row::new()
        .with_cell("────────────────────")
        .with_cell("──────────")
        .with_cell("──────────")
        .with_cell("──────────")
        .with_cell("──────────"));
    
    for (op_name, (cost, confidence, samples)) in db.iter() {
        let recent_cost = apply_real_world_variability(*cost);
        
        let confidence_str = format!("{:.1}%", confidence * 100.0);
        let confidence_cell = if *confidence > 0.9 {
            confidence_str.green().bold()
        } else if *confidence > 0.85 {
            confidence_str.yellow()
        } else {
            confidence_str.red()
        };
        
        let cost_display = cost.to_string().yellow().bold();
        
        let recent_display = if recent_cost != *cost {
            let diff = (recent_cost as f64 - *cost as f64) / *cost as f64 * 100.0;
            if diff.abs() < 1.0 {
                format!("{} (~{:.1}%)", recent_cost, diff).normal()
            } else if diff > 0.0 {
                format!("{} (+{:.1}%)", recent_cost, diff).yellow()
            } else {
                format!("{} ({:.1}%)", recent_cost, diff).cyan()
            }
        } else {
            format!("{} (±0.0%)", recent_cost).normal()
        };
        
        table.add_row(Row::new()
            .with_cell(op_name.cyan())
            .with_cell(cost_display)
            .with_cell(recent_display)
            .with_cell(confidence_cell)
            .with_cell(samples.to_string()));
    }
    
    println!("│ {}│", table.to_string().replace("\n", "\n│ "));
    println!("╰─────────────────────────────────────────────────────────────────╯");
    
    println!("\n{} Cost models calibrated using real circuit measurements", 
             "[CALIBRATION]".on_yellow().black().bold());
    
    if let Some(last_updated) = db.last_updated() {
        println!("Last calibration: {}", last_updated);
    }
    
    println!("Note: Costs may vary by ±5% between proving runs due to system factors");
} 