// PURPOSE: Same function called with different pointer args
// Context-insensitive analysis merges; context-sensitive distinguishes
int *identity(int *p) { return p; }

void test() {
    int x, y;
    int *a = identity(&x);
    int *b = identity(&y);
}
