// valid-memsafety FALSE: unconditional double-free (valid-free).
// ASan reports 'attempting double-free' -> false(valid-free).
#include <stdlib.h>

int main(void) {
    int *p = (int *)malloc(sizeof(int));
    if (!p) return 0;
    free(p);
    free(p); // double free
    return 0;
}
