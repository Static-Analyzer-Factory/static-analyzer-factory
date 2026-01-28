// indirect_call_named_ssa.c
// Regression test: indirect call through a named SSA value (%call)
// must be classified as CallIndirect, not CallDirect.

typedef int (*callback_t)(int);

int double_it(int x) { return x * 2; }
int triple_it(int x) { return x * 3; }

// Returns a function pointer — the call result becomes a named SSA value
callback_t get_callback(int choice) {
    if (choice)
        return double_it;
    return triple_it;
}

int use_callback(int choice, int value) {
    callback_t cb = get_callback(choice);  // %call or %cb — named SSA
    return cb(value);                       // indirect call through named SSA value
}

int main(void) {
    return use_callback(1, 42);
}
