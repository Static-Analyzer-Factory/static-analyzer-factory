// CWE-416: Dangling pointer (C++)
#include <cstdio>
#include <cstdlib>

int *create_value(int x) {
    int *p = (int *)malloc(sizeof(int));
    *p = x;
    return p;
}

void use_after_free(int *p) {
    free(p);
    printf("value = %d\n", *p);  // dangling
}

int main(void) {
    int *val = create_value(42);
    use_after_free(val);
    return 0;
}
