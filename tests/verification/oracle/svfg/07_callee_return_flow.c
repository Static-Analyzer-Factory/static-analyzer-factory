// PURPOSE: Value flows caller→callee→return→caller
int transform(int x) {
    return x + 1;
}

void test() {
    int a = 10;
    int b = transform(a);
    (void)b;
}
