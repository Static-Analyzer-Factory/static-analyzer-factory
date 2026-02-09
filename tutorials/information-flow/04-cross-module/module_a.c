// CWE-78: Cross-module command injection
// Module A: reads user input and passes to module B

#include <stdlib.h>
#include <stdio.h>

// Declared in module_b.c
extern void module_b_process(const char *data);

int main(void) {
    // SOURCE: getenv() reads an environment variable controlled by the user
    const char *user_input = getenv("USER_CMD");
    if (!user_input) {
        printf("Set USER_CMD environment variable\n");
        return 1;
    }

    // Pass tainted data to module B
    module_b_process(user_input);
    return 0;
}
