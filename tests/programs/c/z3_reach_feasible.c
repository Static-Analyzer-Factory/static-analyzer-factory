// Two program points connected by a feasible CFG path.
// Z3 should confirm reachability with a witness path.
#include <stdio.h>

void process(int x) {
    if (x > 0) {
        // Block A: reached when x > 0
        printf("positive: %d\n", x);
    }
    if (x < 100) {
        // Block B: reached when x < 100
        // Feasible path from A to B when 0 < x < 100
        printf("bounded: %d\n", x);
    }
}

int main() {
    process(50);
    return 0;
}
