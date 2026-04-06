// PURPOSE: Pointer passed as function parameter
void callee(int *p) {
    *p = 42;
}

void test() {
    int x;
    callee(&x);
}
