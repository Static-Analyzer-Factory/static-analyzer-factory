int *get_value() {
    int x = 42;
    // BUG: returning address of stack variable
    return &x;
}

int main() {
    int *p = get_value();
    return *p;
}
