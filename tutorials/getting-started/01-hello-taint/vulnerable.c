// Command Injection via argv - Hello Taint Tutorial
//
// This simple program demonstrates a taint flow vulnerability:
// data from argv (user-controlled command-line input) flows directly
// to system() (command execution) without any sanitization.
//
// CWE-78: Improper Neutralization of Special Elements used in an OS Command

#include <stdlib.h>

int main(int argc, char **argv) {
    // SOURCE: argv[1] is user-controlled input from the command line
    // An attacker can provide arbitrary input here

    if (argc > 1) {
        // SINK: system() executes argv[1] as a shell command
        // This is a classic command injection vulnerability
        system(argv[1]);
    }

    return 0;
}
