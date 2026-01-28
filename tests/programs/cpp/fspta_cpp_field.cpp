/* fspta_cpp_field.cpp — C++ class field pointer reassignment
 *
 * A class has a pointer field that gets reassigned across methods.
 * Flow-sensitive PTA should track that after set_b(), the field
 * points to b_val, not a_val.
 */

int a_val, b_val;

class Container {
public:
    int *data;

    void set_a() {
        data = &a_val;
    }

    void set_b() {
        data = &b_val;
    }

    int read() {
        return *data;
    }
};

int main() {
    Container c;
    c.set_a();    /* c.data -> {a_val} */
    c.set_b();    /* c.data -> {b_val} (strong update kills a_val) */
    return c.read();
}
