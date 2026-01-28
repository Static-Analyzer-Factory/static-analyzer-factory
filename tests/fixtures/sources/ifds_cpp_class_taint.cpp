// IFDS E2E: C++ class method taint propagation
//
// Taint flows through a C++ class:
//   getenv() → Wrapper constructor → get() method → system()
//
// Tests that IFDS tracks taint through C++ object construction
// and method calls (which compile to non-virtual function calls
// with implicit `this` pointer).

#include <cstdlib>

class CommandWrapper {
    char *cmd;
public:
    CommandWrapper(char *input) : cmd(input) {}
    char *get() { return cmd; }
};

int main() {
    char *env = getenv("CMD");
    CommandWrapper wrapper(env);
    char *result = wrapper.get();
    system(result);
    return 0;
}
