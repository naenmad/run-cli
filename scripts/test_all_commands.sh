#!/usr/bin/env bash
# ==============================================================================
# run-cli: Comprehensive Automated Command Test Harness
# ==============================================================================
# Self-discovering: Discovers all subcommands dynamically from the compiled
# binary. Adding new commands NEVER requires modifying this script or test code.
# ==============================================================================

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Terminal Colors & Icons
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

TICK="✔"
CROSS="✖"
ARROW="➜"

# Counters
TOTAL_HELP_TESTS=0
PASSED_HELP_TESTS=0
FAILED_HELP_TESTS=0

TOTAL_SMOKE_TESTS=0
PASSED_SMOKE_TESTS=0
FAILED_SMOKE_TESTS=0

START_TIME=$(date +%s)

echo -e "${BOLD}${CYAN}================================================================${RESET}"
echo -e "${BOLD}${CYAN}         RUN-CLI COMPREHENSIVE AUTOMATED TEST SUITE             ${RESET}"
echo -e "${BOLD}${CYAN}================================================================${RESET}"
echo -e "${MAGENTA}Engine:${RESET} Self-discovering dynamic harness (Zero manual updates required)"
echo -e "${MAGENTA}Root:${RESET}   $PROJECT_ROOT\n"

# ------------------------------------------------------------------------------
# STEP 1: Build the Debug Binary
# ------------------------------------------------------------------------------
echo -e "${BOLD}${BLUE}[1/5] Building project binary...${RESET}"
cargo build --quiet
BIN="$PROJECT_ROOT/target/debug/run"

if [ ! -f "$BIN" ]; then
    echo -e "${RED}${CROSS} Error: Binary not found at $BIN${RESET}"
    exit 1
fi
echo -e "  ${GREEN}${TICK}${RESET} Binary compiled successfully: ${BOLD}$BIN${RESET}\n"

# ------------------------------------------------------------------------------
# STEP 2: Dynamically Discover All Commands
# ------------------------------------------------------------------------------
echo -e "${BOLD}${BLUE}[2/5] Auto-discovering all subcommands from binary...${RESET}"

# Dynamically extract subcommand names from `run --help` under the Commands section
DISCOVERED_COMMANDS=($("$BIN" --help | awk '/^Commands:/,/^Options:/' | grep -E '^  [a-z]' | awk '{print $1}'))
CMD_COUNT=${#DISCOVERED_COMMANDS[@]}

if [ "$CMD_COUNT" -eq 0 ]; then
    echo -e "${RED}${CROSS} Error: Failed to discover any subcommands from binary output!${RESET}"
    exit 1
fi

echo -e "  ${GREEN}${TICK}${RESET} Discovered ${BOLD}${GREEN}$CMD_COUNT${RESET} subcommands dynamically:"
echo -e "     ${CYAN}${DISCOVERED_COMMANDS[*]}${RESET}\n"

# ------------------------------------------------------------------------------
# STEP 3: Test `--help` Parsing and Documentation on EVERY Discovered Command
# ------------------------------------------------------------------------------
echo -e "${BOLD}${BLUE}[3/5] Testing '--help' and documentation for all $CMD_COUNT commands...${RESET}"

FAILED_HELP_LIST=()

for cmd in "${DISCOVERED_COMMANDS[@]}"; do
    TOTAL_HELP_TESTS=$((TOTAL_HELP_TESTS + 1))
    CMD_START=$(date +%s%N 2>/dev/null || date +%s)

    # Execute --help
    if HELP_OUTPUT=$("$BIN" "$cmd" --help 2>&1); then
        # Verify output contains usage or description
        if echo "$HELP_OUTPUT" | grep -qi -E "(Usage:|Arguments:|Options:|Commands:|--help)"; then
            PASSED_HELP_TESTS=$((PASSED_HELP_TESTS + 1))
            printf "  ${GREEN}${TICK}${RESET} %-14s ${GREEN}[PASS]${RESET}\n" "$cmd"
        else
            FAILED_HELP_TESTS=$((FAILED_HELP_TESTS + 1))
            FAILED_HELP_LIST+=("$cmd (Empty or invalid help output)")
            printf "  ${RED}${CROSS}${RESET} %-14s ${RED}[FAIL - Invalid Help Text]${RESET}\n" "$cmd"
        fi
    else
        FAILED_HELP_TESTS=$((FAILED_HELP_TESTS + 1))
        FAILED_HELP_LIST+=("$cmd (Exit status $?)")
        printf "  ${RED}${CROSS}${RESET} %-14s ${RED}[FAIL - Non-Zero Exit]${RESET}\n" "$cmd"
    fi
done

echo ""

# ------------------------------------------------------------------------------
# STEP 4: Live Non-Destructive Smoke Tests (Real Execution)
# ------------------------------------------------------------------------------
echo -e "${BOLD}${BLUE}[4/5] Running live non-destructive smoke tests...${RESET}"

run_smoke_test() {
    local desc="$1"
    shift
    local cmd_args=("$@")

    TOTAL_SMOKE_TESTS=$((TOTAL_SMOKE_TESTS + 1))
    printf "  %-40s " "$desc..."

    if OUTPUT=$("$BIN" "${cmd_args[@]}" 2>&1); then
        PASSED_SMOKE_TESTS=$((PASSED_SMOKE_TESTS + 1))
        echo -e "${GREEN}${TICK} [PASS]${RESET}"
    else
        FAILED_SMOKE_TESTS=$((FAILED_SMOKE_TESTS + 1))
        echo -e "${RED}${CROSS} [FAIL]${RESET}"
        echo -e "${RED}    Command: $BIN ${cmd_args[*]}${RESET}"
        echo -e "${RED}    Output: $OUTPUT${RESET}"
    fi
}

run_smoke_test "run time (timestamp format)" time
run_smoke_test "run whoami (system identity)" whoami
run_smoke_test "run uuid (generate UUIDv4)" uuid
run_smoke_test "run pass 16 (16-char password)" pass 16
run_smoke_test "run color #3B82F6 (hex inspector)" color "#3B82F6"
run_smoke_test "run color red (named color converter)" color "red"
run_smoke_test "run mock user 2 --json (JSON dummy data)" mock user 2 --json
run_smoke_test "run mock product 2 --csv (CSV dummy data)" mock product 2 --csv
run_smoke_test "run path (current directory)" path
run_smoke_test "run help (command guide overview)" help
run_smoke_test "run qr --help (QR suite help)" qr --help
run_smoke_test "run img --help (terminal img help)" img --help
run_smoke_test "run ocr --help (Apple Vision OCR help)" ocr --help
run_smoke_test "run encrypt --help (crypto help)" encrypt --help
run_smoke_test "run decrypt --help (crypto help)" decrypt --help
run_smoke_test "run lan --help (LAN scanner help)" lan --help

echo ""

# ------------------------------------------------------------------------------
# STEP 5: Run In-Tree Rust Integration & Symmetry Test Suite
# ------------------------------------------------------------------------------
echo -e "${BOLD}${BLUE}[5/5] Running Rust compiler-level test suite (cargo test)...${RESET}"
if cargo test -- --nocapture; then
    RUST_TEST_STATUS="${GREEN}${TICK} All Rust tests passed${RESET}"
else
    RUST_TEST_STATUS="${RED}${CROSS} Rust tests failed${RESET}"
    FAILED_SMOKE_TESTS=$((FAILED_SMOKE_TESTS + 1))
fi

echo ""

# ------------------------------------------------------------------------------
# SUMMARY REPORT
# ------------------------------------------------------------------------------
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo -e "${BOLD}${CYAN}================================================================${RESET}"
echo -e "${BOLD}${CYAN}                     TEST SUMMARY REPORT                        ${RESET}"
echo -e "${BOLD}${CYAN}================================================================${RESET}"
echo -e "Total Subcommands Discovered: ${BOLD}$CMD_COUNT${RESET}"
echo -e "Help Tests:                   ${GREEN}$PASSED_HELP_TESTS passed${RESET} / ${TOTAL_HELP_TESTS} total"
echo -e "Smoke Tests:                  ${GREEN}$PASSED_SMOKE_TESTS passed${RESET} / ${TOTAL_SMOKE_TESTS} total"
echo -e "Rust In-Tree Tests:           $RUST_TEST_STATUS"
echo -e "Total Execution Time:         ${BOLD}${ELAPSED}s${RESET}"

if [ "$FAILED_HELP_TESTS" -eq 0 ] && [ "$FAILED_SMOKE_TESTS" -eq 0 ]; then
    echo -e "\n${BOLD}${GREEN}✔ ALL TESTS PASSED SUCCESSFULLY! (100% HEALTHY)${RESET}\n"
    exit 0
else
    echo -e "\n${BOLD}${RED}✖ SOME TESTS FAILED!${RESET}"
    if [ ${#FAILED_HELP_LIST[@]} -gt 0 ]; then
        echo -e "${RED}Failed commands:${RESET}"
        for f in "${FAILED_HELP_LIST[@]}"; do
            echo -e "  - $f"
        done
    fi
    echo ""
    exit 1
fi
