// PURPOSE: Simple function pointer call — PTA resolves indirect target
void handler(int x) { (void)x; }

void test() {
    void (*fp)(int) = handler;
    fp(42);
}
