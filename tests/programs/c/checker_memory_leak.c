#include <stdlib.h>

int *create_buffer(int size) {
    int *p = (int *)malloc(size * sizeof(int));
    p[0] = 42;
    return p;
}

void process() {
    int *buf = create_buffer(10);
    buf[1] = 100;
    // BUG: buf is never freed — memory leak
}

int main() {
    process();
    return 0;
}
