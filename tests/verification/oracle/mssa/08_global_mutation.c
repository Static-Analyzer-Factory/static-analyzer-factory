// PURPOSE: Global variable modified by function call
int g = 0;

void modify_global(void) {
    g = 42;
}

void test() {
    modify_global();
    int y = g;  // reaching def is inside modify_global
    (void)y;
}
