#!/bin/bash

# ICN CLI Test Script
# Description: Tests the ICN CLI functionality against the running system

# Ensure we're in the right directory
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

# Log file setup
LOG_DATE=$(date +"%Y%m%d-%H%M%S")
CLI_TEST_LOG="${PROJECT_ROOT}/logs/icn-cli-test-${LOG_DATE}.log"
mkdir -p "${PROJECT_ROOT}/logs"
touch "$CLI_TEST_LOG"
echo "--- ICN CLI Test Log - $(date) ---" >> "$CLI_TEST_LOG"

# Set default values if not in environment
ICN_SERVER_PORT=${ICN_SERVER_PORT:-8081}
ICN_API_URL=${ICN_API_URL:-"http://localhost:${ICN_SERVER_PORT}/api/v1"}

# Function to log messages
log_message() {
    local MESSAGE=$1
    local LEVEL=${2:-INFO}
    local TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
    
    case "$LEVEL" in
        "ERROR")
            echo -e "${RED}[ERROR] $MESSAGE${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}[WARNING] $MESSAGE${NC}"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[SUCCESS] $MESSAGE${NC}"
            ;;
        *)
            echo -e "[INFO] $MESSAGE"
            ;;
    esac
    
    echo "[$TIMESTAMP][$LEVEL] $MESSAGE" >> "$CLI_TEST_LOG"
}

# Function to find the ICN CLI executable
find_cli() {
    local CLI_PATH=""
    
    # Check in service registry first
    if [ -f "${PROJECT_ROOT}/.icn_services" ]; then
        CLI_PATH=$(grep "icn-cli" "${PROJECT_ROOT}/.icn_services" | cut -d':' -f3)
        if [ -n "$CLI_PATH" ] && [ -f "$CLI_PATH" ]; then
            echo "$CLI_PATH"
            return 0
        fi
    fi
    
    # Try common locations
    if [ -f "${PROJECT_ROOT}/target/debug/icn-cli" ]; then
        echo "${PROJECT_ROOT}/target/debug/icn-cli"
        return 0
    elif [ -f "${PROJECT_ROOT}/target/release/icn-cli" ]; then
        echo "${PROJECT_ROOT}/target/release/icn-cli"
        return 0
    elif [ -f "${PROJECT_ROOT}/crates/icn-cli/target/debug/icn-cli" ]; then
        echo "${PROJECT_ROOT}/crates/icn-cli/target/debug/icn-cli"
        return 0
    fi
    
    # Try to build it if not found
    log_message "ICN CLI not found. Attempting to build it..." "WARNING"
    (cd "${PROJECT_ROOT}" && cargo build --bin icn-cli) > /dev/null 2>&1
    
    if [ -f "${PROJECT_ROOT}/target/debug/icn-cli" ]; then
        echo "${PROJECT_ROOT}/target/debug/icn-cli"
        return 0
    fi
    
    # Not found
    return 1
}

# Function to run a CLI command and check for errors
run_cli_command() {
    local CLI_PATH=$1
    local COMMAND=$2
    local EXPECTED_CODE=${3:-0}
    local EXPECTED_OUTPUT=$4
    
    local TEMP_OUTPUT="/tmp/icn-cli-output-$$"
    
    echo -e "\n${YELLOW}Testing command:${NC} $COMMAND"
    echo "$ $CLI_PATH $COMMAND" >> "$CLI_TEST_LOG"
    
    # Execute command and capture output and exit code
    $CLI_PATH $COMMAND > "$TEMP_OUTPUT" 2>&1
    local EXIT_CODE=$?
    
    # Log the output
    cat "$TEMP_OUTPUT" >> "$CLI_TEST_LOG"
    
    # Check if exit code matches expected
    if [ $EXIT_CODE -eq $EXPECTED_CODE ]; then
        echo -e "${GREEN}✓ Command completed with expected exit code ${EXPECTED_CODE}${NC}"
        
        # Check for expected output if specified
        if [ -n "$EXPECTED_OUTPUT" ]; then
            if grep -q "$EXPECTED_OUTPUT" "$TEMP_OUTPUT"; then
                echo -e "${GREEN}✓ Expected output found${NC}"
            else
                echo -e "${RED}✗ Expected output not found${NC}"
                echo -e "${GRAY}Expected to contain: ${EXPECTED_OUTPUT}${NC}"
                echo -e "${GRAY}Actual output:${NC}"
                cat "$TEMP_OUTPUT" | sed 's/^/  /'
                rm "$TEMP_OUTPUT"
                return 1
            fi
        fi
        
        # Display the output for informational commands
        if [ $EXPECTED_CODE -eq 0 ]; then
            echo -e "${GRAY}Command output:${NC}"
            cat "$TEMP_OUTPUT" | sed 's/^/  /'
        fi
        
        rm "$TEMP_OUTPUT"
        return 0
    else
        echo -e "${RED}✗ Command failed with exit code ${EXIT_CODE} (expected ${EXPECTED_CODE})${NC}"
        echo -e "${GRAY}Command output:${NC}"
        cat "$TEMP_OUTPUT" | sed 's/^/  /'
        rm "$TEMP_OUTPUT"
        return 1
    fi
}

# Run a suite of tests
run_test_suite() {
    local CLI_PATH=$1
    local TESTS_PASSED=0
    local TESTS_FAILED=0
    
    echo -e "\n${BLUE}========== Running ICN CLI Test Suite ==========${NC}"
    
    # Test 1: Basic help command
    echo -e "\n${BLUE}Test 1: Basic Help Command${NC}"
    if run_cli_command "$CLI_PATH" "--help" 0 "USAGE:"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    
    # Test 2: Version information
    echo -e "\n${BLUE}Test 2: Version Information${NC}"
    if run_cli_command "$CLI_PATH" "--version" 0 "icn-cli"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    
    # Test 3: Status command (connect to backend)
    echo -e "\n${BLUE}Test 3: Status Command${NC}"
    if run_cli_command "$CLI_PATH" "status --url $ICN_API_URL" 0; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    
    # Test 4: Invalid command
    echo -e "\n${BLUE}Test 4: Invalid Command Handling${NC}"
    if run_cli_command "$CLI_PATH" "nonexistent-command" 1 "error"; then
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    
    # More tests can be added here
    
    # Report results
    echo -e "\n${BLUE}Test Results:${NC}"
    echo -e "${GREEN}✓ Tests passed: ${TESTS_PASSED}${NC}"
    echo -e "${RED}✗ Tests failed: ${TESTS_FAILED}${NC}"
    
    # Log results
    echo "Tests passed: ${TESTS_PASSED}" >> "$CLI_TEST_LOG"
    echo "Tests failed: ${TESTS_FAILED}" >> "$CLI_TEST_LOG"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        return 0
    else
        return 1
    fi
}

# Main execution
main() {
    echo -e "${BLUE}========== ICN CLI Test ==========${NC}"
    echo -e "Running tests at $(date)\n"
    
    # Check if the ICN server is running (backend API)
    if ! curl -s --max-time 3 "$ICN_API_URL/health" > /dev/null; then
        log_message "ICN server is not running at $ICN_API_URL" "ERROR"
        echo -e "${RED}ICN server is not running or not accessible at $ICN_API_URL${NC}"
        echo -e "${YELLOW}Please make sure the ICN system is started using ./start_icn.sh${NC}"
        exit 1
    fi
    
    log_message "ICN server is running at $ICN_API_URL" "SUCCESS"
    
    # Find the CLI executable
    CLI_PATH=$(find_cli)
    
    if [ -z "$CLI_PATH" ]; then
        log_message "Could not find or build the ICN CLI" "ERROR"
        echo -e "${RED}Could not find or build the ICN CLI.${NC}"
        echo -e "${YELLOW}Make sure Rust is installed and the project can be built.${NC}"
        exit 1
    fi
    
    log_message "Found ICN CLI at $CLI_PATH" "SUCCESS"
    echo -e "${GREEN}Found ICN CLI at:${NC} $CLI_PATH"
    
    # Run the test suite
    run_test_suite "$CLI_PATH"
    TEST_RESULT=$?
    
    # Final message
    if [ $TEST_RESULT -eq 0 ]; then
        echo -e "\n${GREEN}All CLI tests passed!${NC}"
        log_message "All CLI tests completed successfully" "SUCCESS"
    else
        echo -e "\n${RED}Some CLI tests failed.${NC}"
        log_message "Some CLI tests failed" "ERROR"
    fi
    
    echo -e "Detailed test log: ${CLI_TEST_LOG}"
}

# Execute main function
main