// CWE-78: Interprocedural Command Injection
//
// This program demonstrates taint flow through a helper function:
// user-controlled input from getenv() flows through process_input()
// and reaches system() without sanitization.
//
// The SAF IFDS taint analyzer detects this by:
// 1. Identifying getenv() return as a taint SOURCE
// 2. Identifying system() arg 0 as a taint SINK
// 3. Using the IFDS tabulation algorithm to track taint through
//    the interprocedural call boundary (main → process_input → main)

#include <stdlib.h>

// A helper function that passes input through unchanged.
// IFDS tracks taint from the parameter to the return value
// across this function boundary.
char *process_input(char *input) {
    return input;
}

int main() {
    // SOURCE: getenv() returns attacker-influenced data
    char *data = getenv("USER_CMD");

    // Taint flows into process_input() and back out
    char *processed = process_input(data);

    // SINK: system() executes the string as a shell command
    return system(processed);
}
