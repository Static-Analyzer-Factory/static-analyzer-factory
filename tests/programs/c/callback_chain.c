// Callback Chain: A -> B -> C with Tainted Data
// Function A reads user input and passes it to B via function pointer.
// B forwards the data to C, which executes it. The call chain
// A -> B -> C must be fully resolved to detect the taint flow.
//
// Expected finding: argv -> step_a() -> step_b() -> step_c()/system() (taint chain)
#include <stdlib.h>

typedef void (*handler_t)(const char *);

void step_c(const char *data) {
    system(data);                            // SINK: command execution
}

void step_b(const char *data) {
    handler_t next = step_c;
    next(data);                              // indirect call to step_c
}

void step_a(const char *input) {
    handler_t handler = step_b;
    handler(input);                          // indirect call to step_b
}

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char *user_data = argv[1];               // SOURCE: user-controlled input
    step_a(user_data);                       // taint enters the call chain
    return 0;
}
