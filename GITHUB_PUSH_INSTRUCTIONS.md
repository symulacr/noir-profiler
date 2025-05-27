# GitHub Push Instructions

This repository has been sanitized and prepared for GitHub. Follow these steps to push:

## Setup

1. Configure your Git email if not already set:
   ```
   git config user.email "your@email.com"
   ```

2. Add the GitHub repository as the origin remote:
   ```
   git remote add origin git@github.com:symulacr/noir-circuit-profiler.git
   ```

3. Push to GitHub:
   ```
   git push -u origin main
   ```

## Repository Structure

This clean repository includes:

- Core Noir Profiler source code in `src/`
- Example circuits in `examples/`
- Documentation in `docs/`
- Main executable script `np.sh` 
- Empty `circuit_stats/` directory for storing analysis output

## Features

- Circuit constraint analysis
- Operation breakdown
- Black box function tracking
- Comparison tools
- Research data collection

Refer to the README.md for complete usage instructions. 