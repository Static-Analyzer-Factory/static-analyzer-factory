#include <stdio.h>
extern void reach_error(void);
int main(void){ int x=0; printf("ab%n",&x); if(x==2) reach_error(); return 0; }
