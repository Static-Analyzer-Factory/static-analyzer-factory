#include <stdio.h>

void process(const char *path) {
    FILE *f = fopen(path, "r");
    if (!f) return;

    char buf[256];
    fgets(buf, sizeof(buf), f);

    // BUG: f is never closed
}

int main() {
    process("data.txt");
    return 0;
}
