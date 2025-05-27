#!/bin/bash
# noir profiler - circuit analysis tool

set -e

# colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
BOLD='\033[1m'
UNDERLINE='\033[4m'
NC='\033[0m'

MODE=$1
TARGET=$2

print_header() {
  local title=$1
  echo -e "\n${BLUE}${BOLD}╭────────────────────────────────────────────────╮${NC}"
  echo -e "${BLUE}${BOLD}│${NC} ${WHITE}${title}${NC} ${BLUE}${BOLD}│${NC}"
  echo -e "${BLUE}${BOLD}╰────────────────────────────────────────────────╯${NC}\n"
}

show_help() {
  echo -e "${CYAN}${BOLD}
  ███╗   ██╗ ██████╗ ██╗██████╗     ██████╗ ██████╗  ██████╗ ███████╗██╗██╗     ███████╗██████╗ 
  ████╗  ██║██╔═══██╗██║██╔══██╗    ██╔══██╗██╔══██╗██╔═══██╗██╔════╝██║██║     ██╔════╝██╔══██╗
  ██╔██╗ ██║██║   ██║██║██████╔╝    ██████╔╝██████╔╝██║   ██║█████╗  ██║██║     █████╗  ██████╔╝
  ██║╚██╗██║██║   ██║██║██╔══██╗    ██╔═══╝ ██╔══██╗██║   ██║██╔══╝  ██║██║     ██╔══╝  ██╔══██╗
  ██║ ╚████║╚██████╔╝██║██║  ██║    ██║     ██║  ██║╚██████╔╝██║     ██║███████╗███████╗██║  ██║
  ╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═╝  ╚═╝    ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝${NC}\n
"
  echo -e "${CYAN}${BOLD}  circuit analysis tool - experimental demo version${NC}\n"
  
  echo -e "${UNDERLINE}${WHITE}usage:${NC}"
  echo -e "  ${0} ${BLUE}[command]${NC} ${YELLOW}[arguments]${NC}\n"
  
  echo -e "${UNDERLINE}${WHITE}commands:${NC}"
  echo -e "  ${GREEN}${BOLD}analyze${NC} ${YELLOW}<circuit.json>${NC}      analyze circuit file"
  echo -e "  ${GREEN}${BOLD}compare${NC} ${YELLOW}<file1> <file2>${NC}     compare two circuits"
  echo -e "  ${GREEN}${BOLD}batch${NC} ${YELLOW}<directory>${NC}           analyze all circuits in directory"
  echo -e "  ${GREEN}${BOLD}stats${NC} ${YELLOW}<directory>${NC}           collect research statistics"
  echo -e "  ${GREEN}${BOLD}calibrate${NC} ${YELLOW}<directory>${NC}       calibrate cost model with example circuits"
  echo -e "  ${GREEN}${BOLD}demo${NC}                        run demonstration"
  echo -e "  ${GREEN}${BOLD}help${NC}                        show help message\n"
  
  echo -e "${UNDERLINE}${WHITE}examples:${NC}"
  echo -e "  ${0} analyze examples/circuits/simple_hash.json"
  echo -e "  ${0} compare examples/circuits/circuit1.json examples/circuits/circuit2.json"
  echo -e "  ${0} batch examples/circuits"
  echo -e "  ${0} stats examples/circuits > stats_output.csv"
}

fatal_error() {
  echo -e "${RED}${BOLD}error: ${1}${NC}" >&2
  exit 1
}

# ensure build is complete
ensure_build() {
  if [ ! -f "target/release/noir-circuit-profiler" ]; then
    echo -e "\n${YELLOW}${BOLD}building noir-circuit-profiler...${NC}"
    cargo build --release > /dev/null 2>&1
    echo -e "${GREEN}${BOLD}build complete${NC}"
  fi
}

# execute main binary
run_profiler() {
  TARGET_FILE=$(realpath "${1}")
  ensure_build
  target/release/noir-circuit-profiler "$@"
}

# analyze a circuit file
analyze_circuit() {
  if [ -z "$TARGET" ]; then
    echo -e "${RED}${BOLD}error: missing circuit file${NC}"
    echo -e "usage: $0 analyze <circuit.json>"
    exit 1
  fi
  
  if [ ! -f "$TARGET" ]; then
    fatal_error "file not found: $TARGET"
  fi
  
  print_header "analyzing circuit: $TARGET"
  run_profiler analyze "$TARGET"
}

# compare two circuits
compare_circuits() {
  CIRCUIT1="$TARGET"
  CIRCUIT2="$2"
  
  if [ -z "$CIRCUIT1" ] || [ -z "$CIRCUIT2" ]; then
    echo -e "${RED}${BOLD}error: missing circuit files${NC}"
    echo -e "usage: $0 compare <circuit1.json> <circuit2.json>"
    exit 1
  fi
  
  if [ ! -f "$CIRCUIT1" ]; then
    fatal_error "file not found: $CIRCUIT1"
  fi
  
  if [ ! -f "$CIRCUIT2" ]; then
    fatal_error "file not found: $CIRCUIT2"
  fi
  
  print_header "comparing circuits"
  run_profiler compare "$CIRCUIT1" "$CIRCUIT2"
}

run_demo() {
  print_header "noir profiler demonstration"
  
  # use simple example circuits
  EXAMPLE_CIRCUITS="examples/circuits"
  SIMPLE="$EXAMPLE_CIRCUITS/simple_arithmetic.json"
  HASH="$EXAMPLE_CIRCUITS/simple_hash.json"
  COMPLEX="$EXAMPLE_CIRCUITS/complex_crypto.json"
  
  if [ ! -d "$EXAMPLE_CIRCUITS" ]; then
    echo -e "${RED}${BOLD}error: example circuits not found${NC}"
    echo "expected to find examples in $EXAMPLE_CIRCUITS"
    exit 1
  fi
  
  # ensure fresh build
  echo -e "${YELLOW}${BOLD}building latest version...${NC}"
  cargo build --release > /dev/null
  echo -e "${GREEN}${BOLD}build complete${NC}"

  # analyze simple circuit
  echo -e "\n${CYAN}${BOLD}analyzing simple circuit...${NC}"
  sleep 1
  run_profiler analyze "$SIMPLE" | tail -n +3
  
  # analyze hash circuit
  echo -e "\n${CYAN}${BOLD}analyzing circuit with hash operation...${NC}"
  sleep 1
  run_profiler analyze "$HASH" | tail -n +3
  
  # compare circuits
  echo -e "\n${CYAN}${BOLD}comparing circuits...${NC}"
  sleep 1
  run_profiler compare "$SIMPLE" "$HASH" | tail -n +3
  
  echo -e "\n${GREEN}${BOLD}demonstration complete${NC}"
  echo "use ./np.sh help for more options"
}

batch_analyze() {
  if [ -z "$TARGET" ]; then
    echo -e "${RED}${BOLD}error: missing directory${NC}"
    echo -e "usage: $0 batch <directory>"
    exit 1
  fi
  
  if [ ! -d "$TARGET" ]; then
    fatal_error "directory not found: $TARGET"
  fi
  
  print_header "batch analyzing circuits in $TARGET"
  run_profiler batch "$TARGET"
}

collect_stats() {
  if [ -z "$TARGET" ]; then
    echo -e "${RED}${BOLD}error: missing directory${NC}"
    echo -e "usage: $0 stats <directory>"
    exit 1
  fi
  
  if [ ! -d "$TARGET" ]; then
    fatal_error "directory not found: $TARGET"
  fi
  
  print_header "collecting research statistics"
  run_profiler stats "$TARGET"
}

calibrate_model() {
  if [ -z "$TARGET" ]; then
    echo -e "${RED}${BOLD}error: missing directory${NC}"
    echo -e "usage: $0 calibrate <directory> [--reset]"
    exit 1
  fi
  
  if [ ! -d "$TARGET" ]; then
    fatal_error "directory not found: $TARGET"
  fi
  
  RESET=""
  if [ "$2" = "--reset" ]; then
    RESET="--reset"
  fi
  
  print_header "calibrating cost model"
  run_profiler calibrate "$TARGET" "$RESET"
}

# main script logic
case $MODE in
  "analyze")
    analyze_circuit
    ;;
  "compare")
    compare_circuits "$2" "$3"
    ;;
  "batch")
    batch_analyze
    ;;
  "stats")
    collect_stats
    ;;
  "calibrate")
    calibrate_model "$TARGET" "$3"
    ;;
  "demo")
    run_demo
    ;;
  "help"|*)
    show_help
    ;;
esac 