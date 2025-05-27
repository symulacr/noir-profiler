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
./np.sh analyze examples/circuits/circuit.json

# compare circuits
./np.sh compare circuit1.json circuit2.json

# batch analyze
./np.sh batch directory/with/circuits

# collect statistics
./np.sh stats circuits_dir > stats_output.csv

# show help
./np.sh help
```

## circuit analysis

```bash
./np.sh analyze examples/circuits/mirror_zero.json
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

## example circuits

The repository includes reference circuits in `examples/circuits/` ready for analysis.

To analyze:
```bash
./np.sh analyze examples/circuits/mirror_zero.json
```

The `examples/sample.nr` file is a reference implementation that demonstrates various optimization approaches.

## optimization techniques

- **field arithmetic optimization**: efficient use of basic operations
- **unconstrained computation**: moving work off-circuit
- **conditional logic minimization**: reducing branching
- **lookup tables**: for expensive calculations
- **cryptographic primitive comparison**: different hash functions
- **static vs. dynamic access**: optimizing array usage
- **witness size reduction**: combining values
- **type conversion handling**: minimizing constraint overhead
- **efficient bit operations**: for logic operations

## metrics

Example analysis output:

```
[METRICS] Circuit Analysis: examples/circuits/mirror_zero.json
╭───────────────────────────────────╮
│ Metric   Value                    │
│ Total Constraints   0             │
│ Total ACIR Opcodes  0             │
│ Public Inputs       0             │
│ Private Inputs      0             │
│ Input/Output Count  0 in / 0 out  │
│ Est. Proving Time   0.00ms        │
╰───────────────────────────────────╯
```

## license

MIT License - See LICENSE file for details
