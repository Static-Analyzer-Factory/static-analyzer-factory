// Genuine taint flow that Z3 confirms as feasible.
// getenv → system with no sanitization.
#include <stdlib.h>

extern char *getenv(const char *);
extern int system(const char *);

void run_command() {
    char *cmd = getenv("CMD");
    // Direct taint: getenv → system, no branch guards
    system(cmd);
}

int main() {
    run_command();
    return 0;
}
