#!/usr/bin/env bash
# Professional Test Runner Script with Comprehensive Reporting
# Tracks individual test cases and generates detailed reports

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m' # No Color

# Test tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0
START_TIME=$(date +%s)

# Arrays to track test details
declare -a PASSED_TEST_NAMES=()
declare -a FAILED_TEST_NAMES=()
declare -a SKIPPED_TEST_NAMES=()

# Suite tracking
declare -a SUITE_NAMES=()
declare -a SUITE_RESULTS=()
declare -a SUITE_PASSED=()
declare -a SUITE_FAILED=()
declare -a SUITE_TOTAL=()

# Report file
REPORT_DIR="tmp/test-reports"
REPORT_FILE="${REPORT_DIR}/test-report-$(date +%Y%m%d-%H%M%S).txt"

# Create report directory
mkdir -p "${REPORT_DIR}"

# Initialize report
init_report() {
    {
        echo "╔════════════════════════════════════════════════════════════════╗"
        echo "║           Axum Backend - Comprehensive Test Report            ║"
        echo "╚════════════════════════════════════════════════════════════════╝"
        echo ""
        echo "📅 Date:       $(date '+%Y-%m-%d %H:%M:%S')"
        echo "🏷️  Test Mode:  $1"
        echo "📂 Project:    Axum Backend"
        echo ""
        echo "════════════════════════════════════════════════════════════════"
        echo ""
    } > "${REPORT_FILE}"
}

# Print header
print_header() {
    echo ""
    echo -e "${BLUE}${BOLD}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}${BOLD}║           Axum Backend - Professional Test Runner             ║${NC}"
    echo -e "${BLUE}${BOLD}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# Print test case result
print_test_result() {
    local test_name=$1
    local status=$2
    local suite_name=$3
    
    if [ "$status" = "PASS" ]; then
        echo -e "  ${GREEN}✓${NC} ${test_name}"
        echo "  ✓ ${test_name}" >> "${REPORT_FILE}"
        PASSED_TEST_NAMES+=("${suite_name}::${test_name}")
        ((PASSED_TESTS++))
    elif [ "$status" = "FAIL" ]; then
        echo -e "  ${RED}✗${NC} ${test_name}"
        echo "  ✗ ${test_name}" >> "${REPORT_FILE}"
        FAILED_TEST_NAMES+=("${suite_name}::${test_name}")
        ((FAILED_TESTS++))
    elif [ "$status" = "SKIP" ]; then
        echo -e "  ${YELLOW}○${NC} ${test_name} ${DIM}(skipped)${NC}"
        echo "  ○ ${test_name} (skipped)" >> "${REPORT_FILE}"
        SKIPPED_TEST_NAMES+=("${suite_name}::${test_name}")
        ((SKIPPED_TESTS++))
    fi
    ((TOTAL_TESTS++))
}

# Parse cargo test output and track individual tests
parse_test_output() {
    local output=$1
    local test_suite_name=$2
    
    local suite_passed=0
    local suite_failed=0
    local suite_total=0
    
    echo -e "\n${CYAN}${BOLD}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${CYAN}${BOLD}│ Running: ${test_suite_name}${NC}"
    echo -e "${CYAN}${BOLD}└─────────────────────────────────────────────────────────────┘${NC}\n"
    
    {
        echo ""
        echo "┌─────────────────────────────────────────────────────────────┐"
        echo "│ Test Suite: ${test_suite_name}"
        echo "└─────────────────────────────────────────────────────────────┘"
        echo ""
    } >> "${REPORT_FILE}"
    
    # Extract test results from cargo output
    while IFS= read -r line; do
        # Match lines like "test test_name ... ok" or "test test_name ... FAILED"
        if [[ $line =~ ^test[[:space:]]+([^[:space:]]+)[[:space:]]+\.\.\.[[:space:]]+(ok|FAILED) ]]; then
            test_name="${BASH_REMATCH[1]}"
            result="${BASH_REMATCH[2]}"
            
            if [ "$result" = "ok" ]; then
                print_test_result "$test_name" "PASS" "$test_suite_name"
                ((suite_passed++))
            else
                print_test_result "$test_name" "FAIL" "$test_suite_name"
                ((suite_failed++))
            fi
            ((suite_total++))
        fi
    done <<< "$output"
    
    # Store suite statistics
    SUITE_NAMES+=("$test_suite_name")
    SUITE_PASSED+=("$suite_passed")
    SUITE_FAILED+=("$suite_failed")
    SUITE_TOTAL+=("$suite_total")
    
    # If no individual tests were parsed, check overall result
    if [ $suite_total -eq 0 ]; then
        if [[ $output =~ test[[:space:]]result:[[:space:]]ok ]]; then
            print_test_result "${test_suite_name}" "PASS" "$test_suite_name"
            suite_passed=1
            suite_total=1
        elif [[ $output =~ test[[:space:]]result:[[:space:]]FAILED ]]; then
            print_test_result "${test_suite_name}" "FAIL" "$test_suite_name"
            suite_failed=1
            suite_total=1
        fi
    fi
    
    # Print suite summary
    echo ""
    if [ $suite_failed -eq 0 ] && [ $suite_total -gt 0 ]; then
        echo -e "${GREEN}${BOLD}  ✓ Suite Result: ${suite_passed}/${suite_total} tests passed${NC}"
        echo "  ✓ Suite Result: ${suite_passed}/${suite_total} tests passed" >> "${REPORT_FILE}"
        SUITE_RESULTS+=("PASS")
    else
        echo -e "${RED}${BOLD}  ✗ Suite Result: ${suite_passed}/${suite_total} tests passed, ${suite_failed} failed${NC}"
        echo "  ✗ Suite Result: ${suite_passed}/${suite_total} tests passed, ${suite_failed} failed" >> "${REPORT_FILE}"
        SUITE_RESULTS+=("FAIL")
    fi
    echo "" >> "${REPORT_FILE}"
}

# Run test suite with detailed tracking
run_test_suite() {
    local test_name=$1
    local test_cmd=$2
    local capture_output=${3:-true}
    
    echo -e "${YELLOW}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}📋 Test Suite: ${test_name}${NC}"
    echo -e "${YELLOW}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Run the test and capture output
    local exit_code=0
    local output=""
    
    if [ "$capture_output" = true ]; then
        output=$(eval "${test_cmd}" 2>&1) || exit_code=$?
    else
        eval "${test_cmd}" || exit_code=$?
    fi
    
    if [ "$capture_output" = true ]; then
        parse_test_output "$output" "$test_name"
    fi
    
    # Check overall result
    if [ $exit_code -eq 0 ]; then
        echo -e "\n${GREEN}${BOLD}✓ ${test_name} completed successfully${NC}\n"
        echo "Result: ✓ SUCCESS" >> "${REPORT_FILE}"
    else
        echo -e "\n${RED}${BOLD}✗ ${test_name} failed${NC}\n"
        echo "Result: ✗ FAILED" >> "${REPORT_FILE}"
        
        # Show error output
        if [ -n "$output" ]; then
            echo -e "${RED}${BOLD}Error Details:${NC}"
            echo "$output" | tail -30
            {
                echo ""
                echo "Error Details:"
                echo "$output" | tail -30
            } >> "${REPORT_FILE}"
        fi
    fi
    
    echo "" >> "${REPORT_FILE}"
    echo "────────────────────────────────────────────────────────────────" >> "${REPORT_FILE}"
    echo "" >> "${REPORT_FILE}"
    
    return $exit_code
}

# Generate detailed final report
generate_final_report() {
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))
    
    # Print to console
    echo ""
    echo -e "${BLUE}${BOLD}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}${BOLD}║                    COMPREHENSIVE TEST SUMMARY                  ║${NC}"
    echo -e "${BLUE}${BOLD}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Write to report
    {
        echo ""
        echo "╔════════════════════════════════════════════════════════════════╗"
        echo "║                    COMPREHENSIVE TEST SUMMARY                  ║"
        echo "╚════════════════════════════════════════════════════════════════╝"
        echo ""
    } >> "${REPORT_FILE}"
    
    # Suite-by-suite breakdown
    if [ ${#SUITE_NAMES[@]} -gt 0 ]; then
        echo -e "${CYAN}${BOLD}📊 Test Suite Breakdown:${NC}"
        echo "📊 Test Suite Breakdown:" >> "${REPORT_FILE}"
        echo "" >> "${REPORT_FILE}"
        
        for i in "${!SUITE_NAMES[@]}"; do
            local suite="${SUITE_NAMES[$i]}"
            local result="${SUITE_RESULTS[$i]}"
            local passed="${SUITE_PASSED[$i]}"
            local failed="${SUITE_FAILED[$i]}"
            local total="${SUITE_TOTAL[$i]}"
            
            if [ "$result" = "PASS" ]; then
                echo -e "  ${GREEN}✓${NC} ${suite}: ${passed}/${total} passed"
                echo "  ✓ ${suite}: ${passed}/${total} passed" >> "${REPORT_FILE}"
            else
                echo -e "  ${RED}✗${NC} ${suite}: ${passed}/${total} passed, ${failed} failed"
                echo "  ✗ ${suite}: ${passed}/${total} passed, ${failed} failed" >> "${REPORT_FILE}"
            fi
        done
        echo ""
        echo "" >> "${REPORT_FILE}"
    fi
    
    # Detailed test results
    if [ ${#PASSED_TEST_NAMES[@]} -gt 0 ]; then
        echo -e "${GREEN}${BOLD}✓ Passed Tests (${#PASSED_TEST_NAMES[@]}):${NC}"
        echo "✓ Passed Tests (${#PASSED_TEST_NAMES[@]}):" >> "${REPORT_FILE}"
        for test in "${PASSED_TEST_NAMES[@]}"; do
            echo -e "  ${GREEN}•${NC} ${test}"
            echo "  • ${test}" >> "${REPORT_FILE}"
        done
        echo ""
        echo "" >> "${REPORT_FILE}"
    fi
    
    if [ ${#FAILED_TEST_NAMES[@]} -gt 0 ]; then
        echo -e "${RED}${BOLD}✗ Failed Tests (${#FAILED_TEST_NAMES[@]}):${NC}"
        echo "✗ Failed Tests (${#FAILED_TEST_NAMES[@]}):" >> "${REPORT_FILE}"
        for test in "${FAILED_TEST_NAMES[@]}"; do
            echo -e "  ${RED}•${NC} ${test}"
            echo "  • ${test}" >> "${REPORT_FILE}"
        done
        echo ""
        echo "" >> "${REPORT_FILE}"
    fi
    
    if [ ${#SKIPPED_TEST_NAMES[@]} -gt 0 ]; then
        echo -e "${YELLOW}${BOLD}○ Skipped Tests (${#SKIPPED_TEST_NAMES[@]}):${NC}"
        echo "○ Skipped Tests (${#SKIPPED_TEST_NAMES[@]}):" >> "${REPORT_FILE}"
        for test in "${SKIPPED_TEST_NAMES[@]}"; do
            echo -e "  ${YELLOW}•${NC} ${test}"
            echo "  • ${test}" >> "${REPORT_FILE}"
        done
        echo ""
        echo "" >> "${REPORT_FILE}"
    fi
    
    # Overall statistics
    echo -e "${BOLD}═══════════════════════════════════════════════════════════════${NC}"
    echo "═══════════════════════════════════════════════════════════════" >> "${REPORT_FILE}"
    echo ""
    echo -e "${BOLD}📈 Overall Statistics:${NC}"
    echo "📈 Overall Statistics:" >> "${REPORT_FILE}"
    echo ""
    
    local stats=(
        "Total Tests:    ${TOTAL_TESTS}"
        "Passed:         ${PASSED_TESTS}"
        "Failed:         ${FAILED_TESTS}"
        "Skipped:        ${SKIPPED_TESTS}"
        "Duration:       ${duration}s"
    )
    
    for stat in "${stats[@]}"; do
        if [[ $stat == "Passed:"* ]]; then
            echo -e "  ${GREEN}${BOLD}${stat}${NC}"
        elif [[ $stat == "Failed:"* ]]; then
            echo -e "  ${RED}${BOLD}${stat}${NC}"
        elif [[ $stat == "Skipped:"* ]]; then
            echo -e "  ${YELLOW}${BOLD}${stat}${NC}"
        else
            echo -e "  ${BOLD}${stat}${NC}"
        fi
        echo "  ${stat}" >> "${REPORT_FILE}"
    done
    
    # Calculate success rate
    if [ $TOTAL_TESTS -gt 0 ]; then
        success_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        echo -e "  ${BOLD}Success Rate:   ${success_rate}%${NC}"
        echo "  Success Rate:   ${success_rate}%" >> "${REPORT_FILE}"
    fi
    
    echo ""
    echo "" >> "${REPORT_FILE}"
    
    # Final result
    echo -e "${BOLD}═══════════════════════════════════════════════════════════════${NC}"
    echo "═══════════════════════════════════════════════════════════════" >> "${REPORT_FILE}"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ] && [ $TOTAL_TESTS -gt 0 ]; then
        echo -e "${GREEN}${BOLD}╔═══════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}${BOLD}║                    ✓ ALL TESTS PASSED!                        ║${NC}"
        echo -e "${GREEN}${BOLD}╚═══════════════════════════════════════════════════════════════╝${NC}"
        {
            echo "╔═══════════════════════════════════════════════════════════════╗"
            echo "║                    ✓ ALL TESTS PASSED!                        ║"
            echo "╚═══════════════════════════════════════════════════════════════╝"
        } >> "${REPORT_FILE}"
        final_status="PASSED"
    else
        echo -e "${RED}${BOLD}╔═══════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${RED}${BOLD}║                    ✗ TESTS FAILED                             ║${NC}"
        echo -e "${RED}${BOLD}╚═══════════════════════════════════════════════════════════════╝${NC}"
        {
            echo "╔═══════════════════════════════════════════════════════════════╗"
            echo "║                    ✗ TESTS FAILED                             ║"
            echo "╚═══════════════════════════════════════════════════════════════╝"
        } >> "${REPORT_FILE}"
        final_status="FAILED"
    fi
    
    echo ""
    {
        echo ""
        echo "Final Result: ${final_status}"
        echo ""
        echo "════════════════════════════════════════════════════════════════"
        echo "Report generated at: $(date '+%Y-%m-%d %H:%M:%S')"
        echo "════════════════════════════════════════════════════════════════"
    } >> "${REPORT_FILE}"
    
    # Show report location
    echo -e "${CYAN}${BOLD}📄 Report saved to: ${REPORT_FILE}${NC}"
    echo ""
    
    # Return appropriate exit code
    if [ "$final_status" = "PASSED" ]; then
        return 0
    else
        return 1
    fi
}

# Show help
show_help() {
    echo "Usage: ./tests/run_tests.sh [option]"
    echo ""
    echo "Options:"
    echo "  quick       - Run quick integration tests (health check + registration)"
    echo "  preflight   - Run pre-flight system checks"
    echo "  unit        - Run unit tests"
    echo "  api         - Run all API tests"
    echo "  integration - Run all integration tests"
    echo "  mail        - Run email integration tests (requires real credentials)"
    echo "  authentication - Run authentication tests (Flows, Cookies, Users)"
    echo "  health      - Run health check"
    echo "  stress      - Run stress/performance tests"
    echo "  bench       - Run benchmarks"
    echo "  all         - Run all tests (default if no argument)"
    echo "  help        - Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./tests/run_tests.sh              # Runs all tests (default)"
    echo "  ./tests/run_tests.sh quick"
    echo "  ./tests/run_tests.sh preflight"
    echo "  ./tests/run_tests.sh all"
    echo "  ./tests/run_tests.sh mail"
    echo ""
    echo "Reports are saved to: ${REPORT_DIR}/"
}

# Main execution
main() {
    local test_option="${1:-all}"  # Default to 'all' if no argument
    
    # Show help
    if [ "$test_option" = "help" ] || [ "$test_option" = "-h" ] || [ "$test_option" = "--help" ]; then
        show_help
        exit 0
    fi
    
    print_header
    init_report "$test_option"
    
    # 1. Check formatting
    echo -e "${CYAN}${BOLD}🎨 Checking code formatting...${NC}\n"
    
    # Try to verify formatting first
    if ! cargo fmt --all -- --check > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Formatting issues detected. Attempting to fix automatically...${NC}"
        
        # Run formatter to fix issues
        if cargo fmt --all; then
            echo -e "${GREEN}✅ Formatting fixed.${NC}"
        else
            echo -e "${RED}${BOLD}🛑 Code formatting correction failed. Aborting tests.${NC}"
            generate_final_report
            exit 1
        fi
    else
        echo -e "${GREEN}✅ Code formatting is correct.${NC}"
    fi
    
    # Track if any suite fails
    all_passed=true
    
    case "$test_option" in
        "quick")
            echo -e "${CYAN}${BOLD}🚀 Running quick API tests...${NC}\n"
            run_test_suite \
                "Quick API Tests" \
                "cargo test --test api_tests -- --nocapture test_health_check test_register_success" \
                true || all_passed=false
            ;;
        
        "mail")
            echo -e "${CYAN}${BOLD}📧 Running email integration tests...${NC}\n"
            run_test_suite \
                "Email Integration Tests" \
                "cargo test --test integration_tests -- --ignored --nocapture" \
                true || all_passed=false
            ;;
        
        "stress")
            echo -e "${CYAN}${BOLD}💪 Running stress tests...${NC}\n"
            run_test_suite \
                "Stress Tests" \
                "cargo test --test load_tests -- --nocapture" \
                true || all_passed=false
            ;;
        
        "preflight")
            echo -e "${CYAN}${BOLD}🔍 Running pre-flight checks...${NC}\n"
            run_test_suite \
                "Pre-Flight System Checks" \
                "cargo test --test api_tests -- --nocapture preflight" \
                true || all_passed=false
            ;;
        
        "integration")
            echo -e "${CYAN}${BOLD}🧪 Running all integration tests (Email/Service)...${NC}\n"
            run_test_suite \
                "Integration Tests" \
                "cargo test --test integration_tests -- --nocapture --include-ignored" \
                true || all_passed=false
            ;;
        
        "api")
            echo -e "${CYAN}${BOLD}🌐 Running API tests...${NC}\n"
            run_test_suite \
                "API Tests" \
                "cargo test --test api_tests -- --nocapture" \
                true || all_passed=false
            ;;

        "authentication")
            echo -e "${CYAN}${BOLD}🔐 Running authentication tests (Flows, Cookies, Users)...${NC}\n"
            run_test_suite \
                "Authentication Tests" \
                "cargo test --test api_tests -- --nocapture auth" \
                true || all_passed=false
            ;;
        
        "health")
            echo -e "${CYAN}${BOLD}❤️ Running health check...${NC}\n"
            run_test_suite \
                "Health Check" \
                "cargo test --test api_tests -- --nocapture health" \
                true || all_passed=false
            ;;
        
        "bench")
            echo -e "${CYAN}${BOLD}⚡ Running benchmarks...${NC}\n"
            run_test_suite \
                "Benchmarks" \
                "cargo bench --bench backend_benchmarks -- --test" \
                false || all_passed=false
            ;;
        
        "all")
            echo -e "${CYAN}${BOLD}🎯 Running complete test suite...${NC}\n"
            
            run_test_suite \
                "API Tests" \
                "cargo test --test api_tests -- --nocapture" \
                true || all_passed=false
            
            run_test_suite \
                "Integration Tests" \
                "cargo test --test integration_tests -- --nocapture" \
                true || all_passed=false
            
            run_test_suite \
                "Load/Stress Tests" \
                "cargo test --test load_tests -- --nocapture" \
                true || all_passed=false
            ;;
        
        *)
            echo -e "${RED}Unknown option: $test_option${NC}"
            echo "Run './tests/run_tests.sh help' for usage information"
            exit 1
            ;;
    esac
    
    # Generate final comprehensive report
    generate_final_report
}

# Run main function
main "$@"
