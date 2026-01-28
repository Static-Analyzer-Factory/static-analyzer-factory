// Test: Class Hierarchy Analysis (CHA) for virtual dispatch
// Tests vtable parsing, transitive subclass relationships, and virtual call resolution

#include <cstdlib>

// =============================================================================
// Basic Class Hierarchy
// =============================================================================

// Test 1: Simple single inheritance
class Base {
public:
    virtual ~Base() = default;
    virtual int foo() { return 1; }
    virtual int bar() { return 10; }
};

class Derived : public Base {
public:
    int foo() override { return 2; }  // Overrides Base::foo
    // bar() inherited from Base
};

void test_single_inheritance() {
    Base* b = new Derived();
    int r = b->foo();  // Virtual call - should resolve to Derived::foo
    (void)r;
    delete b;
}

// Test 2: Two-level inheritance (A -> B -> C)
class Level1 {
public:
    virtual ~Level1() = default;
    virtual int method() { return 100; }
};

class Level2 : public Level1 {
public:
    int method() override { return 200; }
};

class Level3 : public Level2 {
public:
    int method() override { return 300; }
};

void test_deep_inheritance() {
    Level1* p = new Level3();
    int r = p->method();  // Should resolve to Level3::method
    (void)r;
    delete p;
}

// Test 3: Multiple derived classes (diamond-free)
class Animal {
public:
    virtual ~Animal() = default;
    virtual void speak() {}
};

class Dog : public Animal {
public:
    void speak() override {}
};

class Cat : public Animal {
public:
    void speak() override {}
};

class Bird : public Animal {
public:
    void speak() override {}
};

void test_multiple_derived(int choice) {
    Animal* a;
    switch (choice) {
        case 0: a = new Dog(); break;
        case 1: a = new Cat(); break;
        default: a = new Bird(); break;
    }
    a->speak();  // CHA: may call Dog::speak, Cat::speak, or Bird::speak
    delete a;
}

// =============================================================================
// Multiple Inheritance
// =============================================================================

// Test 4: Simple multiple inheritance (non-diamond)
class Printable {
public:
    virtual ~Printable() = default;
    virtual void print() {}
};

class Serializable {
public:
    virtual ~Serializable() = default;
    virtual void serialize() {}
};

class Document : public Printable, public Serializable {
public:
    void print() override {}
    void serialize() override {}
};

void test_multiple_inheritance() {
    Printable* p = new Document();
    p->print();
    delete p;

    Serializable* s = new Document();
    s->serialize();
    delete s;
}

// Test 5: Diamond inheritance (virtual base)
class DiamondBase {
public:
    virtual ~DiamondBase() = default;
    virtual int value() { return 0; }
};

class DiamondLeft : virtual public DiamondBase {
public:
    int value() override { return 1; }
};

class DiamondRight : virtual public DiamondBase {
public:
    int value() override { return 2; }
};

class DiamondBottom : public DiamondLeft, public DiamondRight {
public:
    // Must override to resolve ambiguity
    int value() override { return 3; }
};

void test_diamond_inheritance() {
    DiamondBase* b = new DiamondBottom();
    int r = b->value();  // Calls DiamondBottom::value
    (void)r;
    delete b;
}

// =============================================================================
// Pure Virtual (Abstract Classes)
// =============================================================================

// Test 6: Abstract base class
class AbstractShape {
public:
    virtual ~AbstractShape() = default;
    virtual double area() = 0;  // Pure virtual
    virtual double perimeter() = 0;  // Pure virtual
};

class Circle : public AbstractShape {
    double radius;
public:
    explicit Circle(double r) : radius(r) {}
    double area() override { return 3.14159 * radius * radius; }
    double perimeter() override { return 2 * 3.14159 * radius; }
};

class Rectangle : public AbstractShape {
    double width, height;
public:
    Rectangle(double w, double h) : width(w), height(h) {}
    double area() override { return width * height; }
    double perimeter() override { return 2 * (width + height); }
};

void test_abstract_class() {
    AbstractShape* shapes[2];
    shapes[0] = new Circle(5.0);
    shapes[1] = new Rectangle(3.0, 4.0);

    for (int i = 0; i < 2; i++) {
        double a = shapes[i]->area();  // Virtual call
        (void)a;
    }

    delete shapes[0];
    delete shapes[1];
}

// =============================================================================
// Interface-like Patterns
// =============================================================================

// Test 7: Interface pattern (all pure virtual)
class IObserver {
public:
    virtual ~IObserver() = default;
    virtual void update(int value) = 0;
};

class ConcreteObserverA : public IObserver {
public:
    void update(int value) override { (void)value; }
};

class ConcreteObserverB : public IObserver {
public:
    void update(int value) override { (void)value; }
};

void test_interface_pattern() {
    IObserver* observers[2];
    observers[0] = new ConcreteObserverA();
    observers[1] = new ConcreteObserverB();

    for (int i = 0; i < 2; i++) {
        observers[i]->update(42);  // Virtual call through interface
    }

    delete observers[0];
    delete observers[1];
}

// =============================================================================
// CRTP (Compile-Time Polymorphism)
// =============================================================================

// Test 8: CRTP pattern (no vtable, but type hierarchy exists)
template<typename Derived>
class CRTPBase {
public:
    void interface() {
        static_cast<Derived*>(this)->implementation();
    }
    // Default implementation
    void implementation() {}
};

class CRTPDerived : public CRTPBase<CRTPDerived> {
public:
    void implementation() {}  // Override via CRTP
};

void test_crtp() {
    CRTPDerived d;
    d.interface();  // Not virtual, resolved at compile time
}

// =============================================================================
// Factory with Virtual Clone
// =============================================================================

// Test 9: Prototype pattern
class Prototype {
public:
    virtual ~Prototype() = default;
    virtual Prototype* clone() = 0;
};

class ConcretePrototype1 : public Prototype {
public:
    Prototype* clone() override { return new ConcretePrototype1(*this); }
};

class ConcretePrototype2 : public Prototype {
public:
    Prototype* clone() override { return new ConcretePrototype2(*this); }
};

void test_prototype_pattern() {
    Prototype* p = new ConcretePrototype1();
    Prototype* copy = p->clone();  // Virtual call - returns ConcretePrototype1*
    delete p;
    delete copy;
}

// =============================================================================
// Covariant Return Types
// =============================================================================

// Test 10: Covariant return type
class CloneableBase {
public:
    virtual ~CloneableBase() = default;
    virtual CloneableBase* clone_self() { return new CloneableBase(*this); }
};

class CloneableDerived : public CloneableBase {
public:
    // Covariant return type: returns CloneableDerived* instead of CloneableBase*
    CloneableDerived* clone_self() override { return new CloneableDerived(*this); }
};

void test_covariant_return() {
    CloneableBase* b = new CloneableDerived();
    CloneableBase* copy = b->clone_self();  // Virtual call with covariant return
    delete b;
    delete copy;
}

int main() {
    test_single_inheritance();
    test_deep_inheritance();
    test_multiple_derived(0);
    test_multiple_inheritance();
    test_diamond_inheritance();
    test_abstract_class();
    test_interface_pattern();
    test_crtp();
    test_prototype_pattern();
    test_covariant_return();
    return 0;
}
