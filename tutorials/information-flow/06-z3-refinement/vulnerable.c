/**
 * Tutorial 09: Z3 Taint Refinement
 *
 * HTTP request handler with method-dependent sanitization.
 * GET requests sanitize user input via url_decode() before use.
 * POST requests pass raw body directly to process_data().
 *
 * Path-insensitive taint analysis reports a false positive on the
 * GET path because it merges sanitized and unsanitized flows.
 * Z3 refinement filters the infeasible cross-branch flow.
 *
 * Expected findings:
 *   - 1 genuine taint flow: POST path -> system() (no sanitization)
 *   - 1 false positive: GET sanitized path -> system() (Z3 filters this)
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Simulated external functions */
extern char *getenv(const char *name);
extern int system(const char *command);

/* Sanitization function: removes shell metacharacters */
char *sanitize_input(const char *input, char *buf, int len) {
    int j = 0;
    for (int i = 0; input[i] && j < len - 1; i++) {
        char c = input[i];
        if (c != ';' && c != '|' && c != '&' && c != '`') {
            buf[j++] = c;
        }
    }
    buf[j] = '\0';
    return buf;
}

void handle_request(int method) {
    char *user_input = getenv("QUERY_STRING"); /* taint source */
    char cmd[256];
    char safe_buf[256];

    if (method == 0) {
        /* GET: sanitize before use */
        char *safe = sanitize_input(user_input, safe_buf, sizeof(safe_buf));
        snprintf(cmd, sizeof(cmd), "echo %s", safe);
        system(cmd); /* Path-insensitive: reports taint here (FP) */
    } else {
        /* POST: raw input used directly */
        snprintf(cmd, sizeof(cmd), "process %s", user_input);
        system(cmd); /* Genuine taint flow: user_input -> system() */
    }
}

int main(void) {
    handle_request(0); /* GET */
    handle_request(1); /* POST */
    return 0;
}
