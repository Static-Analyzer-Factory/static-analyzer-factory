/**
 * cspta_cpp_factory.cpp — Test CS-PTA with C++ factory pattern.
 *
 * create_object() allocates via new through a shared path.
 * Called from different factory sites, CS-PTA separates the allocations.
 */
#include <cstdlib>

struct Widget {
    int type_id;
    int data;
};

Widget *create_object(int type_id) {
    Widget *w = new Widget();
    w->type_id = type_id;
    w->data = 0;
    return w;
}

void process(Widget *w) {
    w->data = w->type_id * 10;
}

int main() {
    Widget *button = create_object(1);    // factory call 1
    Widget *label  = create_object(2);    // factory call 2

    // With CS-PTA: button and label point to distinct objects
    process(button);
    process(label);

    int result = button->data + label->data;

    delete button;
    delete label;
    return result;
}
