// PURPOSE: Simple function pointer call
void target(void) {}

void test() {
    void (*fp)(void) = target;
    fp();
}
