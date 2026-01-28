// CWE-78: Command Injection — Sanitized (Negative Test)
// User input flows through sanitize_input() before reaching system().
// The sanitizer should block the taint flow.
//
// Expected finding: NONE (sanitizer blocks taint)
#include <stdlib.h>
#include <string.h>

// Sanitization function — strips dangerous characters
char *sanitize_input(const char *input) {
    static char safe[256];
    size_t j = 0;
    for (size_t i = 0; input[i] && j < sizeof(safe) - 1; i++) {
        // Allow only alphanumeric characters
        if ((input[i] >= 'a' && input[i] <= 'z') ||
            (input[i] >= 'A' && input[i] <= 'Z') ||
            (input[i] >= '0' && input[i] <= '9')) {
            safe[j++] = input[i];
        }
    }
    safe[j] = '\0';
    return safe;
}

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char *user_input = argv[1];                  // SOURCE
    char *safe_cmd = sanitize_input(user_input);  // SANITIZER
    return system(safe_cmd);                      // SINK (safe: input was sanitized)
}
