// DDA Context-Sensitive Test
// Tests: Wrapper function disambiguation with CFL context matching.
// Expected: DDA should distinguish p1 and p2 through get_ptr calls.

#include <stdlib.h>

int *get_ptr(int *arr) {
    return arr;
}

int main() {
    int arr1[10];
    int arr2[10];

    int *p1 = get_ptr(arr1);  // Should point to arr1
    int *p2 = get_ptr(arr2);  // Should point to arr2

    *p1 = 1;
    *p2 = 2;

    return *p1 + *p2;
}
