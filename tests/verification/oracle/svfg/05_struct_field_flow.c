// PURPOSE: Value flows through struct field store/load
struct Data { int value; };

void test() {
    struct Data d;
    d.value = 42;
    int x = d.value;
    (void)x;
}
