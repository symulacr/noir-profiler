# noir circuit profiler

![Noir Profiler](https://img.shields.io/badge/NOIR-PROFILER-blue)

circuit analysis tool for noir

```
  ███╗   ██╗ ██████╗ ██╗██████╗     ██████╗ ██████╗  ██████╗ ███████╗██╗██╗     ███████╗██████╗ 
  ████╗  ██║██╔═══██╗██║██╔══██╗    ██╔══██╗██╔══██╗██╔═══██╗██╔════╝██║██║     ██╔════╝██╔══██╗
  ██╔██╗ ██║██║   ██║██║██████╔╝    ██████╔╝██████╔╝██║   ██║█████╗  ██║██║     █████╗  ██████╔╝
  ██║╚██╗██║██║   ██║██║██╔══██╗    ██╔═══╝ ██╔══██╗██║   ██║██╔══╝  ██║██║     ██╔══╝  ██╔══██╗
  ██║ ╚████║╚██████╔╝██║██║  ██║    ██║     ██║  ██║╚██████╔╝██║     ██║███████╗███████╗██║  ██║
  ╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═╝  ╚═╝    ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝
```

circuit analysis tool - experimental demo version

## features

- **constraint analysis**: measure constraints and distribution
- **operation breakdown**: identify frequent operations
- **black box function tracking**: measure cryptographic operations
- **comparison tools**: compare circuit implementations
- **research data collection**: gather statistics

## installation

```bash
# clone repository
git clone git@github.com:symulacr/noir-circuit-profiler.git
cd noir-circuit-profiler

# build tool
cargo build --release
```

## usage

```bash
# analyze circuit
./np.sh analyze path/to/circuit.json

# compare circuits
./np.sh compare circuit1.json circuit2.json

# batch analyze
./np.sh batch directory/with/circuits

# collect statistics
./np.sh stats circuits_dir > stats_output.csv

# calibrate cost model
./np.sh calibrate directory/with/circuits

# show help
./np.sh help
```

## circuit analysis

```bash
./np.sh analyze examples/circuits/circuit2.json
```

shows:
- total constraints and opcodes
- constraint per opcode ratio
- black box function analysis
- operation type distribution
- constraint category breakdown

## circuit comparison

```bash
./np.sh compare circuit1.json circuit2.json
```

highlights:
- constraint counts
- black box function usage
- overall efficiency

## creating test circuits

1. create noir program
2. compile with `nargo compile`
3. analyze resulting acir file

### example noir program

```rust
fn main(x: Field, y: Field, hash_this: pub Field) -> pub Field {
    // arithmetic operations
    let sum = x + y;
    let product = x * y;
    
    // control flow
    let result = if sum > 10 {
        sum * 2
    } else {
        product
    };
    
    // hashing operation
    let hash_result = std::hash::pedersen([hash_this]);
    
    // array operations
    let mut array = [0; 5];
    for i in 0..5 {
        array[i] = result + i as Field;
    }
    
    // output calculation
    let output = array[0] + hash_result[0];
    
    output
}
```

analyze with:
```bash
./np.sh analyze target/main.json
```

## metrics

| metric | description |
|--------|-------------|
| **constraints** | total constraints - directly impacts proving time |
| **opcodes** | operations in acir representation |
| **constraint amplification** | ratio of constraints to opcodes |
| **public inputs** | number of public inputs |
| **black box functions** | cryptographic operations |

## optimization tips

- use constants when possible
- avoid unnecessary hash operations
- reuse hash outputs
- batch similar operations
- consider lookup tables for repetitive operations
- precompute values off-chain when possible

## license

mit license - see license file for details
