// Path-sensitive test: genuine use-after-free (true positive)
// Both path-insensitive and path-sensitive should report this.
#include <stdlib.h>

int main() {
    int *p = (int *)malloc(sizeof(int));
    if (!p) return 1;
    *p = 10;
    free(p);
    *p = 20;  // True UAF - no guard prevents this
    return 0;
}
