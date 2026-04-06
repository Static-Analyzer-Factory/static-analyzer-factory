// PURPOSE: Array of function pointers dispatched by index
void handler_a(void) {}
void handler_b(void) {}
int rand_int(void);

void test() {
    void (*handlers[2])(void) = {handler_a, handler_b};
    handlers[rand_int()]();
}
