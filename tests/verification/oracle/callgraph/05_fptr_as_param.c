// PURPOSE: Function pointer passed as callback parameter
void callback(int x) { (void)x; }

void invoke(void (*cb)(int)) {
    cb(42);
}

void test() {
    invoke(callback);
}
