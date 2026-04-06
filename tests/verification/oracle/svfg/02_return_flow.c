// PURPOSE: Return value flows from callee to caller
int produce(void) {
    return 42;
}

void test() {
    int x = produce();
    (void)x;
}
