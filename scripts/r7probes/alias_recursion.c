// recursion hidden behind a function alias: real -> alias_fn(==real) -> real ...
void real_fn(void);
void alias_fn(void) __attribute__((alias("real_fn")));
void real_fn(void) { alias_fn(); }
int main(void) { real_fn(); return 0; }
