// PURPOSE: Store in callee visible after call returns
void modify(int *p) {
    *p = 99;
}

void test() {
    int x = 0;
    modify(&x);
    int y = x;  // reaching def is inside modify()
    (void)y;
}
