// PURPOSE: Function pointer stored in struct field
typedef void (*callback_t)(int);
struct Handler { callback_t cb; };

void on_event(int x) { (void)x; }

void test() {
    struct Handler h;
    h.cb = on_event;
    h.cb(42);
}
