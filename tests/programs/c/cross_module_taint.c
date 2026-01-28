// Cross-Module Taint Flow
// Taint originates in one "module" (function acting as module A) and
// reaches a dangerous sink in another "module" (function acting as module B).
// In a real build these would be separate translation units linked together.
//
// Expected finding: module_a_get_input() -> module_b_execute() (cross-module taint)
#include <stdlib.h>
#include <string.h>

// --- Module A: input handling ---
char *module_a_get_input(int argc, char *argv[]) {
    if (argc < 2) return NULL;
    return argv[1];                          // SOURCE: user-controlled input
}

// --- Module B: command execution ---
void module_b_execute(const char *cmd) {
    system(cmd);                             // SINK: command execution
}

int main(int argc, char *argv[]) {
    char *input = module_a_get_input(argc, argv);
    if (input) {
        module_b_execute(input);             // taint crosses module boundary
    }
    return 0;
}
