// unreach-call, expected verdict: FALSE. The reach_error() call lives in a
// callee (bug) that is reachable from main, exercising the interprocedural
// error-summary path (aggressive/conservative=false is required to report it).
extern void reach_error(void);

void bug(void) {
    reach_error();
}

int main(void) {
    bug();
    return 0;
}
