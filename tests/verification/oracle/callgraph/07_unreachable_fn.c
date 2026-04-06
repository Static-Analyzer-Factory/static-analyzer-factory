// PURPOSE: Unreachable function should not appear in call edges
void unreachable(void) {}
void reachable(void) {}

void test() {
    reachable();
}
