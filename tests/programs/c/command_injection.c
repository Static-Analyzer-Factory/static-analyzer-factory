// CWE-78: OS Command Injection
// Tainted argv flows directly to system() without sanitization.
//
// Expected finding: argv[1] -> system() (taint flow)
#include <stdlib.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char *user_cmd = argv[1];      // SOURCE: user-controlled input
    return system(user_cmd);        // SINK: command execution
}
