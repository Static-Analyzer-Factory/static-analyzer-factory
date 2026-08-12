// valid-memsafety FALSE: unconditional heap-buffer-overflow (valid-deref).
// ASan traps at the out-of-bounds store on any input, so the R5 unsteered
// concrete-replay confirmer reproduces it -> false(valid-deref).
#include <stdlib.h>

int main(void) {
    int *a = (int *)malloc(4 * sizeof(int));
    if (!a) return 0;
    a[100] = 1; // 400 bytes into a 16-byte allocation (well past ASan's redzone)
    return a[0];
}
