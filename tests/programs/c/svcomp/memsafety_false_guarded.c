// valid-memsafety FALSE, scalar-nondet-GUARDED: the heap OOB fires only when the
// nondet input equals 42. The zeroed-nondet probe (input 0) does NOT reach it, but
// the multi-constant mini-fuzz confirmer (Slice 2) tries 42 and reproduces it ->
// false(valid-deref). Sound: 42 is a valid concrete input.
#include <stdlib.h>
extern int __VERIFIER_nondet_int(void);

int main(void) {
    int *a = (int *)malloc(4 * sizeof(int));
    if (!a) return 0;
    int x = __VERIFIER_nondet_int();
    if (x == 42) {
        a[100] = 1; // out of bounds, only on input 42
    }
    return a[0];
}
