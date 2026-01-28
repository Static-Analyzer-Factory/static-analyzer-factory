// Library Wrapper Hiding Dangerous Sink
// A wrapper function calls a dangerous sink internally. The taint flow
// from user input to the sink is obscured by the wrapper layer.
//
// Expected finding: argv -> safe_exec() -> system() (taint through wrapper)
#include <stdlib.h>
#include <string.h>

void safe_exec(const char *cmd) {
    // Intended to add validation, but doesn't actually sanitize
    system(cmd);                             // SINK: hidden inside wrapper
}

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char *user_input = argv[1];              // SOURCE: user-controlled input
    safe_exec(user_input);                   // taint flows through wrapper
    return 0;
}
