pub mod analyzer;
pub mod core;

pub use core::CircuitAnalysis;
pub use core::{get_operation_details, update_cost_database, save_cost_database, get_cost_database, 
               find_operations_by_cost, apply_real_world_variability, PROVING_TIME_FACTOR};
pub use analyzer::{analyze_circuit, compare_circuits, batch_analyze};

pub fn main() -> anyhow::Result<()> {
    use colored::*;
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "analyze" => {
            if args.len() < 3 {
                println!("{}", "Error: Missing circuit path".red());
                println!("Usage: noir-circuit-profiler analyze <circuit.json>");
                return Ok(());
            }
            let path = &args[2];
            println!("{}", "🔍 Analyzing circuit...".blue().bold());
            let analysis = analyzer::analyze_circuit(std::path::Path::new(path))?;
            print_analysis(&analysis);
        },
        "compare" => {
            if args.len() < 4 {
                println!("{}", "Error: Missing circuit paths".red());
                println!("Usage: noir-circuit-profiler compare <circuit1.json> <circuit2.json>");
                return Ok(());
            }
            let path1 = &args[2];
            let path2 = &args[3];
            println!("{}", "🔄 Comparing circuits...".blue().bold());
            let (analysis1, analysis2) = analyzer::compare_circuits(
                std::path::Path::new(path1), 
                std::path::Path::new(path2)
            )?;
            print_comparison(&analysis1, &analysis2);
        },
        "batch" => {
            if args.len() < 3 {
                println!("{}", "Error: Missing directory path".red());
                println!("Usage: noir-circuit-profiler batch <directory>");
                return Ok(());
            }
            let dir = &args[2];
            println!("{}", "📊 Batch analyzing circuits...".blue().bold());
            let results = analyzer::batch_analyze(std::path::Path::new(dir))?;
            print_batch_results(&results);
        },
        "calibrate" => {
            if args.len() < 3 {
                println!("{}", "Error: Missing directory path".red());
                println!("Usage: noir-circuit-profiler calibrate <directory> [--reset]");
                return Ok(());
            }
            let dir = &args[2];
            let reset = args.len() > 3 && args[3] == "--reset";
            
            if reset {
                println!("{}", "🔄 Resetting cost database...".yellow().bold());
                std::fs::remove_file("circuit_stats/cost_database.json").ok();
                println!("{}", "✓ Database reset to defaults".green());
            }
            
            println!("{}", "📊 Calibrating with example circuits...".blue().bold());
            let results = analyzer::batch_analyze(std::path::Path::new(dir))?;
            
            let successful = results.iter().filter(|(_, r)| r.is_ok()).count();
            println!("\n{} Cost model calibration complete", "✓".green().bold());
            println!("Processed {} circuits ({} successful)", results.len(), successful);
            
            print_cost_database();
        },
        "help" => {
            print_usage();
        },
        unknown => {
            println!("{}: {}", "Unknown command".red().bold(), unknown);
            print_usage();
        }
    }
    
    Ok(())
}

fn print_analysis(analysis: &CircuitAnalysis) {
    use colored::*;
    
    println!("\n{} {}", "Total constraints:".bold(), analysis.constraints);
    
    if !analysis.bottlenecks.is_empty() {
        println!("\n{}", "🚨 Performance bottlenecks:".red().bold());
        for (op_type, cost) in &analysis.bottlenecks {
            println!("  {} - {} constraints", op_type, cost);
        }
    }
    
    println!("\n{} {:.2}ms", "Est. proving time:".bold(), analysis.estimated_proving_time);
}

fn print_comparison(analysis1: &CircuitAnalysis, analysis2: &CircuitAnalysis) {
    use colored::*;
    
    println!("\n{}", "Constraint comparison:".bold());
    println!("  Circuit 1: {} constraints", analysis1.constraints);
    println!("  Circuit 2: {} constraints", analysis2.constraints);
    
    let diff = analysis2.constraints as i64 - analysis1.constraints as i64;
    let percent = if analysis1.constraints > 0 {
        diff as f64 / analysis1.constraints as f64 * 100.0
    } else {
        0.0
    };
    
    println!("  Difference: {:+} constraints ({:+.1}%)", diff, percent);
}

fn print_batch_results(results: &[(String, anyhow::Result<CircuitAnalysis>)]) {
    let mut total_constraints = 0;
    let mut successful = 0;
    
    println!("\n{:<30} | {:<10}", "Circuit", "Constraints");
    println!("{:-<43}", "");
    
    for (name, result) in results {
        match result {
            Ok(analysis) => {
                println!("{:<30} | {:<10}", name, analysis.constraints);
                total_constraints += analysis.constraints;
                successful += 1;
            },
            Err(_) => {
                println!("{:<30} | Error", name);
            }
        }
    }
    
    let avg = if successful > 0 {
        total_constraints / successful
    } else {
        0
    };
    
    println!("\nTotal: {} circuits, {} constraints (avg: {})", 
        results.len(), total_constraints, avg);
}

fn print_cost_database() {
    use colored::*;
    
    let db = core::get_cost_database();
    
    println!("\n{}", "📈 Current Cost Model:".blue().bold());
    
    println!("{:<30} | {:<10} | {:<10} | {:<8}", 
        "Operation".bold(), 
        "Cost".bold(),
        "Confidence".bold(),
        "Samples".bold());
    println!("{:-<64}", "");
    
    for (op_name, (cost, confidence, samples)) in db.iter() {
        let confidence_str = format!("{:.1}%", confidence * 100.0);
        let confidence_display = if *confidence > 0.7 {
            confidence_str.green()
        } else if *confidence > 0.4 {
            confidence_str.yellow()
        } else {
            confidence_str.red()
        };
        
        println!("{:<30} | {:<10} | {:<10} | {:<8}", 
            op_name.cyan(),
            cost.to_string().yellow(),
            confidence_display,
            samples);
    }
    
    if let Some(last_updated) = db.last_updated() {
        println!("\nLast updated: {}", last_updated);
    }
}

fn print_usage() {
    use colored::*;
    
    println!("{}", "\nNoir Circuit Profiler 🔍".bold().blue());
    println!("Zero-complexity ACIR analysis tool for Noir circuit optimization\n");
    
    println!("{}", "COMMANDS:".bold());
    println!("  {} {:<15} {}", "analyze".green().bold(), "<circuit.json>", "Analyze a circuit file");
    println!("  {} {:<15} {}", "compare".green().bold(), "<file1> <file2>", "Compare two circuits");
    println!("  {} {:<15} {}", "batch".green().bold(), "<directory>", "Analyze all circuits in a directory");
    println!("  {} {:<15} {}", "calibrate".green().bold(), "<directory> [--reset]", "Calibrate the cost model");
    println!("  {} {:<15} {}", "help".green().bold(), "", "Show this help message");
    
    println!("\n{}", "EXAMPLES:".bold());
    println!("  noir-circuit-profiler analyze examples/circuits/circuit2.json");
    println!("  noir-circuit-profiler compare circuit1.json circuit2.json");
    println!("  noir-circuit-profiler batch examples/circuits");
} 