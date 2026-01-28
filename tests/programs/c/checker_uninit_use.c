#include <stdlib.h>

int process() {
    int *p = (int *)malloc(sizeof(int));

    // BUG: reading *p before writing to it
    int val = *p;

    free(p);
    return val;
}

int main() {
    return process();
}
