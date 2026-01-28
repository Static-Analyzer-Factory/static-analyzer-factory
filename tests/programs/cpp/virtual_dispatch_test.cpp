// Test program for virtual dispatch resolution
// Verifies that SAF correctly resolves virtual calls to the actual implementation

class Shape {
public:
    virtual int area() { return 0; }
    virtual ~Shape() = default;
};

class Rectangle : public Shape {
    int w, h;
public:
    Rectangle(int w, int h) : w(w), h(h) {}
    int area() override { return w * h; }
};

class Square : public Rectangle {
public:
    Square(int s) : Rectangle(s, s) {}
    // Inherits area() from Rectangle
};

// Test 1: Direct instantiation - should resolve to exact type's method
int test_direct() {
    Rectangle r(3, 4);
    return r.area();  // Should resolve to Rectangle::area only
}

// Test 2: Polymorphic call via base pointer - should resolve based on actual type
int test_polymorphic() {
    Shape* s = new Rectangle(5, 6);
    int result = s->area();  // Rectangle::area
    delete s;
    return result;
}

// Test 3: Call through base reference with derived type
int test_reference(Shape& shape) {
    return shape.area();  // Could be any Shape subclass
}

int main() {
    int a = test_direct();
    int b = test_polymorphic();
    Rectangle r(7, 8);
    int c = test_reference(r);
    return a + b + c;
}
