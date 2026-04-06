// PURPOSE: Value flows from argument to parameter
void use_value(int x) {
    int y = x;
    (void)y;
}

void test() {
    use_value(42);
}
