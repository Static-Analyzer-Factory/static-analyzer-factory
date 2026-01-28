// Two pointers assigned on mutually exclusive branches.
// PTA says may-alias, but Z3 should prove no-alias.
#include <stdlib.h>

int main(int argc, char **argv) {
    int a = 1, b = 2;
    int *p, *q;

    if (argc > 1) {
        p = &a;
        q = &b;
    } else {
        p = &b;
        q = &a;
    }
    // PTA may-alias: {a, b} for both p and q
    // But on each branch, p and q point to different objects
    return *p + *q;
}
