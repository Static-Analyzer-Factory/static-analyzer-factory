// PURPOSE: Function returned as pointer then called
void actual_handler(void) {}

void (*get_handler(void))(void) {
    return actual_handler;
}

void test() {
    void (*fp)(void) = get_handler();
    fp();
}
