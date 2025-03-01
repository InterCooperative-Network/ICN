#!/bin/bash

# Set default parameters
BASE_URL=${BASE_URL:-"http://localhost:8081"}
LOAD_LEVEL=${LOAD_LEVEL:-"low"}
OUTPUT_DIR="./load_test_results"

# Create output directory if it doesn't exist
mkdir -p $OUTPUT_DIR

# Function to display help
show_help() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --url URL       Base URL for the ICN API (default: http://localhost:8081)"
    echo "  --level LEVEL   Load test level: low, medium, high, stress (default: low)"
    echo "  --test TEST     Specific test to run: governance, resource, all (default: all)"
    echo "  --help          Display this help message"
    echo ""
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --url)
            BASE_URL="$2"
            shift
            shift
            ;;
        --level)
            LOAD_LEVEL="$2"
            shift
            shift
            ;;
        --test)
            TEST_TYPE="$2"
            shift
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate load level
if [[ ! "$LOAD_LEVEL" =~ ^(low|medium|high|stress)$ ]]; then
    echo "Invalid load level: $LOAD_LEVEL"
    echo "Valid options are: low, medium, high, stress"
    exit 1
fi

# Timestamp for results
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

echo "=== Running ICN Load Tests ==="
echo "Base URL: $BASE_URL"
echo "Load Level: $LOAD_LEVEL"
echo "Timestamp: $TIMESTAMP"
echo "============================"

# Function to run a specific test
run_test() {
    test_script="$1"
    test_name=$(basename "$test_script" .js)
    
    echo ""
    echo "Running $test_name test..."
    
    # Run the test with k6
    BASE_URL=$BASE_URL LOAD_LEVEL=$LOAD_LEVEL k6 run \
        --out json="$OUTPUT_DIR/${test_name}_${LOAD_LEVEL}_${TIMESTAMP}.json" \
        --out summary="$OUTPUT_DIR/${test_name}_${LOAD_LEVEL}_${TIMESTAMP}_summary.txt" \
        "$test_script"
        
    echo "$test_name test completed."
}

# Determine which tests to run
if [[ -z "$TEST_TYPE" || "$TEST_TYPE" == "all" ]]; then
    # Run all tests
    for test_file in ./tests/load_tests/*_test.js; do
        run_test "$test_file"
    done
else
    # Run specific test
    test_file="./tests/load_tests/${TEST_TYPE}_test.js"
    if [[ -f "$test_file" ]]; then
        run_test "$test_file"
    else
        echo "Test file not found: $test_file"
        exit 1
    fi
fi

echo ""
echo "All tests completed. Results saved to $OUTPUT_DIR"
