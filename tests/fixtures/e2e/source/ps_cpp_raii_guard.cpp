// Path-sensitive test: C++ guarded resource pattern
// A boolean flag controls whether cleanup has occurred.
// Path-insensitive may report double-free; path-sensitive filters.
#include <cstdlib>

extern "C" void use_ptr(int *p);

class Resource {
    int *data;
    bool released;
public:
    Resource() : data((int *)malloc(sizeof(int) * 10)), released(false) {
        if (data) *data = 0;
    }

    void release() {
        if (data && !released) {
            free(data);
            released = true;
        }
    }

    void use_data() {
        if (data && !released) {
            use_ptr(data);
        }
    }

    ~Resource() {
        release();  // Safe: released flag prevents double-free
    }
};

int main() {
    Resource r;
    r.use_data();
    r.release();
    // Destructor also calls release(), but flag prevents double-free
    return 0;
}
