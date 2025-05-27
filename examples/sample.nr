// comprehensive noir profiler demo
// demonstrates circuit optimization techniques and features

// unconstrained function - used for preprocessing
unconstrained fn precompute(value: Field) -> Field {
    value * value * value // cubic calculation done off-circuit
}

// constant values - zero constraint cost
const LOOKUP_TABLE: [Field; 5] = [0, 1, 4, 9, 16]; // precomputed squares

fn main(
    x: Field, 
    y: Field, 
    selector: Field, 
    hash_input: pub Field,
    array_len: Field
) -> pub [Field; 3] {
    // field arithmetic - efficient base operations
    let sum = x + y;       // cheap operation
    let product = x * y;   // slightly more expensive
    
    // conditional logic - try to minimize
    let result = if selector == 1 {
        sum * 2
    } else if selector == 2 {
        product
    } else {
        // prefer field operations over branching when possible
        sum * selector + product * (1 - selector)
    };
    
    // multiple black box functions - different constraint costs
    let hash_pedersen = std::hash::pedersen([hash_input]);     // medium cost
    let hash_sha256 = std::hash::sha256([hash_input]);         // high cost
    let hash_mimc = std::hash::mimc_bn254([hash_input, result]); // different cost profile
    
    // array access optimizations
    
    // static array - lower cost, fixed size
    let mut static_array = [0; 5];
    for i in 0..5 {
        // minimize operations inside loops
        static_array[i] = i as Field;
    }
    
    // use lookup tables for expensive calculations when possible
    let lookup_index = (x % 5) as u32;
    let square = LOOKUP_TABLE[lookup_index]; // constant-time access
    
    // leverage unconstrained computation when possible
    let precalculated = precompute(y);
    
    // minimize witness size (smaller witness = faster proving)
    let combined_hash = hash_pedersen[0] + hash_sha256[0];
    
    // strategic use of mut/immutable data
    let mut running_total = 0;
    for i in 0..5 {
        // minimize array accesses inside loops
        let current_value = static_array[i];
        running_total += current_value;
    }
    
    // example of type conversions - can add constraints
    let bool_flag = x > y;
    let field_from_bool = bool_flag as Field;
    
    // use bit operations efficiently
    let bit_result = (x & 1) | (y & 2);
    
    // optimize output array for multiple return values
    [
        result + precalculated,           // computation result
        combined_hash + running_total,    // hash result
        square + field_from_bool + bit_result // bit operations
    ]
} 