// valid-memsafety FALSE: unconditional NULL dereference (valid-deref).
// ASan reports 'SEGV on unknown address 0x0' -> false(valid-deref).
int main(void) {
    volatile int *p = 0;
    return *p; // null dereference
}
