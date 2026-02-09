// CWE-78: OS Command Injection
//
// This program demonstrates a classic taint flow vulnerability:
// user-controlled input (argv) flows directly to a dangerous
// function (system()) without any sanitization.
//
// The SAF static analyzer detects this by:
// 1. Identifying argv as a taint SOURCE (user-controlled input)
// 2. Identifying system() arg 0 as a taint SINK (command execution)
// 3. Tracing data flow from source to sink through the value-flow graph

#include <stdlib.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;

    // SOURCE: argv[1] is controlled by the user at runtime
    char *user_cmd = argv[1];

    // SINK: system() executes the string as a shell command.
    // If user_cmd is attacker-controlled, this is command injection.
    return system(user_cmd);
}
