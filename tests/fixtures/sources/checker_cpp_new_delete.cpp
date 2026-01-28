// C++ new without delete: memory leak via C++ operators.
// Expected: memory-leak checker should find the leak.

struct Data {
    int value;
    Data(int v) : value(v) {}
};

int main() {
    Data *d = new Data(42);
    int v = d->value;
    // LEAK: 'd' is never deleted
    return v;
}
