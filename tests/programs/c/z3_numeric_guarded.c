// Buffer access guarded by branch but interval analysis widens to [0, TOP].
// Z3 should refute the buffer overflow warning.
#include <stdlib.h>

void fill_array(int *buf, int n) {
    for (int i = 0; i < n; i++) {
        // Interval: i in [0, TOP] after widening
        // But branch guard ensures i < n
        if (i < 10) {
            buf[i] = i * 2;  // Always safe when n <= 10
        }
    }
}

int main() {
    int buf[10];
    fill_array(buf, 10);
    return buf[0];
}
