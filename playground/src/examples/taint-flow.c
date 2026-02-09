#include <stdio.h>

int source() {
    int x;
    scanf("%d", &x);  // taint source
    return x;
}

void sink(int val) {
    printf("result: %d\n", val);  // taint sink
}

int transform(int input) {
    return input * 2 + 1;
}

int main() {
    int tainted = source();
    int derived = transform(tainted);
    sink(derived);  // tainted value flows to sink
    return 0;
}
