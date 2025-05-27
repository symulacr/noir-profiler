use crate::core::{CircuitAnalysis, PROVING_TIME_FACTOR, get_operation_details, update_cost_database, save_cost_database};
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::collections::HashMap;

#[allow(dead_code)]
pub fn analyze_circuit(path: &Path) -> Result<CircuitAnalysis> {
    let json = fs::read_to_string(path)
        .with_context(|| format!("Failed to read circuit file: {}", path.display()))?;
    
    let data: Value = serde_json::from_str(&json)
        .context("Failed to parse JSON")?;
    
    let empty_vec = Vec::new();
    let opcodes = data["opcodes"].as_array().unwrap_or(&empty_vec);
    
    let public_inputs = if let Some(inputs) = data["public_inputs"].as_array() {
        inputs.len()
    } else {
        0
    };
    
    let return_values = if let Some(outputs) = data["return_values"].as_array() {
        outputs.len()
    } else {
        0
    };
    
    let mut _total_witnesses = 0;
    
    if let Some(witnesses) = data["witnesses"].as_object() {
        _total_witnesses = witnesses.len();
    } else {
        let mut witness_set = std::collections::HashSet::new();
        
        for op in opcodes {
            if let Some(op_type) = op["type"].as_str() {
                match op_type {
                    "AssertZero" => {
                        if let Some(terms) = op["expression"]["terms"].as_array() {
                            for term in terms {
                                if let Some(var) = term["variable"].as_str() {
                                    witness_set.insert(var.to_string());
                                }
                            }
                        }
                    },
                    "BlackBoxFunction" => {
                        if let Some(inputs) = op["inputs"].as_array() {
                            for input in inputs {
                                if let Some(var) = input["variable"].as_str() {
                                    witness_set.insert(var.to_string());
                                }
                            }
                        }
                        if let Some(outputs) = op["outputs"].as_array() {
                            for output in outputs {
                                if let Some(var) = output["variable"].as_str() {
                                    witness_set.insert(var.to_string());
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        
        _total_witnesses = witness_set.len();
    }
    
    let private_inputs = if _total_witnesses >= public_inputs {
        _total_witnesses - public_inputs
    } else {
        let max_var_index: usize = 0;
        max_var_index.saturating_sub(public_inputs)
    };
    
    let mut analysis = CircuitAnalysis::default();
    analysis.total_opcodes = opcodes.len();
    analysis.public_inputs = public_inputs;
    analysis.private_inputs = private_inputs;
    analysis.return_values = return_values;
    
    let mut op_counts: HashMap<String, usize> = HashMap::new();
    let mut black_box_usages = Vec::new();
    let mut operation_costs = Vec::new();
    let mut black_box_functions: Vec<(String, usize, usize)> = Vec::new();
    
    let mut operation_types = HashMap::new();
    
    for (idx, op) in opcodes.iter().enumerate() {
        let op_type = op["type"].as_str().unwrap_or("Unknown");
        let op_key = if op_type == "BlackBoxFunction" {
            "External".to_string()
        } else if op_type == "AssertZero" {
            "Constraint".to_string()
        } else {
            op_type.to_string()
        };
        
        *op_counts.entry(op_key.clone()).or_insert(0) += 1;
        
        let (cost, confidence) = match op_type {
            "BlackBoxFunction" => {
                let fn_name = op["function"].as_str().unwrap_or("unknown");
                let (op_cost, conf) = get_operation_details(fn_name);
                
                black_box_usages.push((fn_name, idx));
                operation_costs.push((format!("External::{}", fn_name), op_cost));
                
                operation_types.entry(fn_name.to_string())
                    .or_insert_with(Vec::new)
                    .push(idx);
                
                if let Some(idx) = black_box_functions.iter().position(|(name, _, _)| name == fn_name) {
                    black_box_functions[idx].1 += 1;
                } else {
                    black_box_functions.push((fn_name.to_string(), 1, op_cost));
                }
                
                (op_cost, conf)
            },
            "AssertZero" => {
                let terms = op["expression"]["terms"].as_array().unwrap_or(&empty_vec).len();
                let op_cost = if terms > 0 { (terms + 3) / 4 } else { 1 };
                operation_costs.push(("Constraint".to_string(), op_cost));
                
                operation_types.entry("AssertZero".to_string())
                    .or_insert_with(Vec::new)
                    .push(idx);
                
                (op_cost, 0.98)
            },
            _ => {
                let (op_cost, conf) = (1, 0.9);
                operation_costs.push((op_type.to_string(), op_cost));
                
                operation_types.entry(op_type.to_string())
                    .or_insert_with(Vec::new)
                    .push(idx);
                
                (op_cost, conf)
            }
        };
        
        analysis.constraints += cost;
        
        if cost > 10_000 {
            analysis.bottlenecks.push((op_key, cost));
        }
        
        if analysis.confidence == 0.0 {
            analysis.confidence = confidence;
        } else {
            analysis.confidence = (analysis.confidence + confidence) / 2.0;
        }
    }
    
    analysis.operation_counts = op_counts.into_iter().collect();
    analysis.black_box_functions = black_box_functions;
    analysis.operation_counts.sort_by(|a, b| b.1.cmp(&a.1));
    
    let hardware_factor = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as f64 / 1_000_000_000.0;
        
        0.85 + (seed.sin().abs() * 0.3)
    };
    
    let base_proving_time = (analysis.constraints as f64) * PROVING_TIME_FACTOR / 50.0;
    
    analysis.estimated_proving_time = base_proving_time * hardware_factor;
    
    if analysis.constraints > 0 {
        let parallel_factor = if has_sequential_dependencies(&analysis) {
            1.0 - (0.15 * (analysis.public_inputs as f64).sqrt() / 10.0).min(0.5)
        } else {
            1.0 - (0.3 * (analysis.public_inputs as f64).sqrt() / 10.0).min(0.7)
        };
        
        analysis.estimated_proving_time *= parallel_factor;
    }
    
    update_cost_database_from_circuit(&operation_types, &analysis);
    
    Ok(analysis)
}

fn update_cost_database_from_circuit(
    operation_types: &HashMap<String, Vec<usize>>,
    analysis: &CircuitAnalysis
) {
    for (op_name, instances) in operation_types {
        if instances.len() < 1 {
            continue;
        }
        
        if op_name == "BlackBoxFunction" {
            continue;
        }
        
        if let Some(bb_func) = analysis.black_box_functions.iter()
            .find(|(name, count, _)| name == op_name && *count == 1) {
                
            let (_, _, cost) = bb_func;
            update_cost_database(op_name, *cost);
        }
        
        if op_name == "AssertZero" && instances.len() >= 10 {
            let avg_cost = analysis.constraints / instances.len();
            update_cost_database(op_name, avg_cost);
        }
    }
    
    save_cost_database();
}

fn has_sequential_dependencies(analysis: &CircuitAnalysis) -> bool {
    let has_memory_ops = analysis.operation_counts.iter()
        .any(|(op, _)| op.contains("Memory") || op.contains("Array"));
    
    let has_multiple_hashes = analysis.black_box_functions.iter()
        .filter(|(name, _, _)| name.contains("hash") || name.contains("Hash"))
        .map(|(_, count, _)| count)
        .sum::<usize>() > 1;
    
    has_memory_ops || !has_multiple_hashes
}

#[allow(dead_code)]
pub fn compare_circuits(path1: &Path, path2: &Path) -> Result<(CircuitAnalysis, CircuitAnalysis)> {
    let analysis1 = analyze_circuit(path1)?;
    let analysis2 = analyze_circuit(path2)?;
    
    analyze_diff_from_cost_model(&analysis1, &analysis2);
    
    Ok((analysis1, analysis2))
}

fn analyze_diff_from_cost_model(analysis1: &CircuitAnalysis, analysis2: &CircuitAnalysis) {
    let diff = analysis2.constraints as i64 - analysis1.constraints as i64;
    
    if diff.abs() < 100 {
        return;
    }
    
    let mut op_diffs = Vec::new();
    
    let mut all_ops = std::collections::HashMap::new();
    
    for (op_name, count) in &analysis1.operation_counts {
        all_ops.entry(op_name.clone()).or_insert((0, 0)).0 = *count;
    }
    
    for (op_name, count) in &analysis2.operation_counts {
        all_ops.entry(op_name.clone()).or_insert((0, 0)).1 = *count;
    }
    
    for (op_name, (count1, count2)) in all_ops {
        let op_diff = count2 as i64 - count1 as i64;
        if op_diff != 0 {
            op_diffs.push((op_name, op_diff));
        }
    }
    
    op_diffs.sort_by(|a, b| b.1.abs().cmp(&a.1.abs()));
    
    let mut external_diffs = Vec::new();
    let bb1: std::collections::HashMap<_, _> = analysis1.black_box_functions
        .iter()
        .map(|(name, count, _)| (name.clone(), *count))
        .collect();
        
    let bb2: std::collections::HashMap<_, _> = analysis2.black_box_functions
        .iter()
        .map(|(name, count, _)| (name.clone(), *count))
        .collect();
        
    let mut all_bb = std::collections::HashSet::new();
    for (name, _) in &bb1 {
        all_bb.insert(name.clone());
    }
    for (name, _) in &bb2 {
        all_bb.insert(name.clone());
    }
    
    for name in all_bb {
        let count1 = *bb1.get(&name).unwrap_or(&0);
        let count2 = *bb2.get(&name).unwrap_or(&0);
        let op_diff = count2 as i64 - count1 as i64;
        
        if op_diff != 0 {
            external_diffs.push((name, op_diff));
        }
    }
}

#[allow(dead_code)]
pub fn batch_analyze(dir: &Path) -> Result<Vec<(String, Result<CircuitAnalysis>)>> {
    let mut results = Vec::new();
    
    if !dir.exists() || !dir.is_dir() {
        return Err(anyhow::anyhow!("Directory not found or is not a directory: {}", dir.display()));
    }
    
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "json") && e.path().exists()
        })
    {
        let path = entry.path();
        let file_name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.is_file() && metadata.len() > 0 {
                    results.push((file_name, analyze_circuit(path)));
                }
            },
            Err(_) => continue
        }
    }
    
    Ok(results)
} 