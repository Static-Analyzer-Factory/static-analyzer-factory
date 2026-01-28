// Two program points connected by CFG path but guarded by contradictory
// conditions — Z3 should prove unreachable.
#include <stdio.h>

void process(int x) {
    if (x > 10) {
        // Block A: only reached when x > 10
        printf("x is large: %d\n", x);
    }
    if (x < 5) {
        // Block B: only reached when x < 5
        // No feasible path from A to B since x > 10 && x < 5 is UNSAT
        printf("x is small: %d\n", x);
    }
}

int main() {
    process(7);
    return 0;
}
