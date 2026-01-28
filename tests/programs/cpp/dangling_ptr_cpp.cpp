// CWE-416: Dangling Pointer (C++)
// Function returns a reference to a local variable. The reference
// becomes dangling as soon as the function returns.
//
// Expected finding: returned reference points to expired stack frame
#include <cstdio>

int &get_value() {
    int local = 42;                  // SOURCE: stack-local variable
    return local;                    // SINK: returns dangling reference
}

int main() {
    int &ref = get_value();
    printf("%d\n", ref);             // use of dangling reference
    return 0;
}
