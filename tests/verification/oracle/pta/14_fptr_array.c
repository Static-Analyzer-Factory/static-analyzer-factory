// PURPOSE: Function pointer array — PTA resolves all possible targets
void a(void) {}
void b(void) {}
void c(void) {}

int rand_index(void);

void test() {
    void (*handlers[3])(void) = {a, b, c};
    handlers[rand_index()]();
}
