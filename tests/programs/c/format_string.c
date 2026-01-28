// CWE-134: Format String Vulnerability
// User input from gets() is passed directly as printf format argument.
//
// Expected finding: gets(buf) -> printf(buf) format arg (taint flow)
#include <stdio.h>

// gets() was removed in C11; provide declaration for analysis purposes.
extern char *gets(char *s);

int main(void) {
    char buf[256];
    gets(buf);          // SOURCE: user-controlled input
    printf(buf);        // SINK: format string (arg index 0)
    return 0;
}
