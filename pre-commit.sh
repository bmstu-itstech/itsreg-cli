#!/bin/bash

echo "Running pre-commit checks..."

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ERRORS=0

error() {
    echo -e "${RED}ERROR: $1${NC}"
    ERRORS=1
}

warning() {
    echo -e "${YELLOW}WARNING: $1${NC}"
}

info() {
    echo -e "${GREEN}INFO: $1${NC}"
}

info "Checking Python syntax..."
find . -name "*.py" -not -path "./venv/*" -not -path "./.venv/*" -not -path "*/__pycache__/*" | while read -r file; do
    if ! python -m py_compile "$file" >/dev/null 2>&1; then
        error "Syntax error in $file"
        python -m py_compile "$file" 2>&1 | head -10
    fi
done

if command -v flake8 >/dev/null 2>&1; then
    info "Running flake8..."
    if ! flake8 . --exclude=venv,.venv,__pycache__ --max-line-length=120; then
        error "flake8 found issues"
    fi
else
    warning "flake8 not installed, skipping"
fi

if command -v isort >/dev/null 2>&1; then
    info "Checking import sorting with isort..."
    if ! isort . --check-only --diff --skip venv --skip .venv --skip __pycache__; then
        error "Imports need sorting. Run 'isort .' to fix"
    fi
else
    warning "isort not installed, skipping"
fi

if command -v black >/dev/null 2>&1; then
    info "Checking code formatting with black..."
    if ! black . --check --exclude="venv|.venv|__pycache__"; then
        error "Code needs formatting. Run 'black .' to fix"
    fi
else
    warning "black not installed, skipping"
fi

if command -v mypy >/dev/null 2>&1; then
    info "Running type checking with mypy..."
    if ! mypy . --ignore-missing-imports --exclude '(venv|.venv|__pycache__)'; then
        error "Type checking found issues"
    fi
else
    info "mypy not installed, skipping"
fi

if [ $ERRORS -eq 0 ]; then
    info "All pre-commit checks passed!"
    exit 0
else
    error "Pre-commit checks failed. Please fix the issues above."
    exit 1
fi
