// Abstract interpretation test: integer overflow detection (CWE-190)
// This program has:
// 1. Safe arithmetic that stays within i32 bounds
// 2. Multiplication that may overflow

#include <stdlib.h>

int safe_add(int a, int b) {
    // If a, b are small, this is safe
    return a + b;
}

long compute_area(int width, int height) {
    // Potential i32 overflow: width * height can exceed 2^31-1
    // if both are large (e.g., 50000 * 50000 = 2.5 billion > 2^31)
    int area = width * height;
    return (long)area;
}

int accumulate(int n) {
    int sum = 0;
    for (int i = 0; i < n; i++) {
        sum = sum + i;  // sum grows without bound if n is large
    }
    return sum;
}

int main(void) {
    int a = safe_add(10, 20);
    long area = compute_area(640, 480);
    int sum = accumulate(100);
    return (int)(a + area + sum);
}
