// CWE-78: Cross-module command injection
// Module B: receives data from module A and executes it

#include <stdlib.h>

void module_b_process(const char *data) {
    // SINK: system() executes the string as a shell command.
    // The data comes from getenv() in module_a.c — this is command injection.
    system(data);
}
