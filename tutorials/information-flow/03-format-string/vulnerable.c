// CWE-134: Uncontrolled Format String
//
// This program reads user input via gets() and passes it directly
// as the format string argument to printf(). An attacker can supply
// format specifiers like %x, %n to read/write memory.
//
// SAF detects this by tracing the return value of gets() (tainted)
// to the first argument of printf() (format string position).

#include <stdio.h>

// Declare gets() explicitly — it was removed from the C11 standard but
// we need its correct signature (returning char *) for SAF to trace the
// return value through the value-flow graph.
extern char *gets(char *);

int main(void) {
    char buf[256];

    // SOURCE: gets() returns a pointer to the tainted buffer.
    // We capture the return value so SAF can trace the SSA data flow.
    char *input = gets(buf);

    // SINK: printf()'s argument 0 is the format string.
    // If user-controlled, this enables format string attacks.
    printf(input);

    return 0;
}
