extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);
int f(int d,int x){ if(d==0){int r; if(__VERIFIER_nondet_int()) r=1; else r=0; return r;}
  int y=f(d-1,0); if(x==7 && y) reach_error(); return 0; }
int main(void){ int x=__VERIFIER_nondet_int(); if(x!=5 && x!=7) return 0; f(1,x); return 0; }
