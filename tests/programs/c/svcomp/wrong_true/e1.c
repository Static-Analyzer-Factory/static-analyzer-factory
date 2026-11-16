extern void reach_error(void);
void f(int n){ int i=0; while(i<n) i++; }
void g(void){ f(0); }
int main(void){ g(); f(1); reach_error(); return 0; }
