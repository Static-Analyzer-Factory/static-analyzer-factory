// PURPOSE: Value flows through global variable
int g;

void writer(void) {
    g = 42;
}

void reader(void) {
    int x = g;
    (void)x;
}

void test() {
    writer();
    reader();
}
