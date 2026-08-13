// destructor loops forever AFTER main returns -> program never terminates
__attribute__((destructor)) void dtor(void) { while (1) { } }
int main(void) { return 0; }
