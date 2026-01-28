// Genuine file leak — file is opened but not closed on any path.
// Z3 should confirm the leak is feasible.
#include <stdio.h>
#include <stdlib.h>

void process(const char *filename) {
    FILE *f = fopen(filename, "r");
    if (f == NULL) return;
    char buf[256];
    fgets(buf, sizeof(buf), f);
    // LEAK: f is never closed
}

int main() {
    process("data.txt");
    return 0;
}
