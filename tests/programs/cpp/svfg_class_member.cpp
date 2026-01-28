// SVFG E2E test: C++ class with constructor storing to member field.
// Getter loads from member field.
// Tests struct field flow through indirect edges.

extern "C" int source();
extern "C" void sink(int);

class Container {
    int value;
public:
    Container(int v) : value(v) {}
    int get() const { return value; }
};

void test() {
    int tainted = source();
    Container c(tainted);
    int val = c.get();
    sink(val);
}
