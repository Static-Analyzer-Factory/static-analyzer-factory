#include <setjmp.h>
static jmp_buf b;
int main(void) { volatile int x = 0; setjmp(b); x++; longjmp(b, 1); return 0; }
