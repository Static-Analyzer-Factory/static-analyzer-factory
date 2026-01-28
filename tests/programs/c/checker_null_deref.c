#include <stdlib.h>

int process() {
    int *p = (int *)malloc(sizeof(int));
    // BUG: malloc can return NULL, but p is used without check
    *p = 42;
    int val = *p;
    free(p);
    return val;
}

int main() {
    return process();
}
