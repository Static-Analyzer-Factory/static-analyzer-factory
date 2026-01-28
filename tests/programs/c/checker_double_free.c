#include <stdlib.h>

void process() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;

    free(p);

    // BUG: p is freed again
    free(p);
}

int main() {
    process();
    return 0;
}
