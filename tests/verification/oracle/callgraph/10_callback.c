// PURPOSE: Callback stored in struct and invoked
typedef void (*event_cb)(int);
struct EventHandler { event_cb on_event; };

void handle(int x) { (void)x; }

void dispatch(struct EventHandler *h, int val) {
    h->on_event(val);
}

void test() {
    struct EventHandler eh;
    eh.on_event = handle;
    dispatch(&eh, 42);
}
