// Virtual Method Dispatch through Base Pointer
// A virtual method is called on a base pointer that actually points to
// a derived class. PTA must resolve the vtable to find the actual callee.
//
// Expected finding: indirect call via base->speak() resolves to Dog::speak()
#include <cstdio>

class Animal {
public:
    virtual void speak() {
        printf("...\n");
    }
    virtual ~Animal() = default;
};

class Dog : public Animal {
public:
    void speak() override {                  // resolved target
        printf("woof\n");
    }
};

int main() {
    Animal *a = new Dog();                   // SOURCE: Dog vtable assigned
    a->speak();                              // SINK: indirect virtual call
    delete a;
    return 0;
}
