// Tests trace partitioning: constant branch should produce precise intervals.
#include <stdlib.h>

#define BUFFER_SIZE 10

int main() {
    char buf[BUFFER_SIZE];
    int idx;

    // Constant comparison — partitioning should split here
    if (BUFFER_SIZE == 10) {
        idx = 5;  // Safe: 5 < 10
    } else {
        idx = 15; // Overflow: 15 >= 10, but this is dead code
    }

    buf[idx] = 'A';
    return 0;
}
