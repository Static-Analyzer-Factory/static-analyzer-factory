// Phase 7: MSSA/SVFG Integration Tests (C++)
// Tests: Class fields, constructors, virtual calls, RAII

extern void sink(int);

class Container {
public:
    int value;

    Container(int v) : value(v) {}

    void setValue(int v) {
        value = v;  // Store to this->value
    }

    int getValue() const {
        return value;  // Load from this->value
    }

    void increment() {
        value = value + 1;  // Load then store
    }
};

// Test: field store/load through this pointer
void test_field_access() {
    Container c(10);

    c.setValue(20);
    int r = c.getValue();  // Should see store from setValue

    sink(r);
}

// Test: constructor initializes field
void test_constructor() {
    Container c(42);

    int r = c.getValue();  // Should see store from constructor
    sink(r);
}

// Test: method chains
void test_method_chain() {
    Container c(0);

    c.increment();  // 0 -> 1
    c.increment();  // 1 -> 2
    c.increment();  // 2 -> 3

    int r = c.getValue();  // Should see last increment's store
    sink(r);
}

// Test: multiple objects, no aliasing
void test_multiple_objects() {
    Container c1(100);
    Container c2(200);

    c1.setValue(111);
    c2.setValue(222);

    int r1 = c1.getValue();  // From c1's store
    int r2 = c2.getValue();  // From c2's store

    sink(r1);
    sink(r2);
}

// Test: pointer to object
void test_object_pointer() {
    Container c(0);
    Container* p = &c;

    p->setValue(50);
    int r = p->getValue();  // Through pointer

    sink(r);
}

// Test: conditional field update
void test_conditional_field(int cond) {
    Container c(0);

    if (cond) {
        c.setValue(10);
    } else {
        c.setValue(20);
    }

    // Memory phi for c.value
    int r = c.getValue();
    sink(r);
}

// Entry point
int main() {
    test_field_access();
    test_constructor();
    test_method_chain();
    test_multiple_objects();
    test_object_pointer();
    test_conditional_field(1);
    return 0;
}
