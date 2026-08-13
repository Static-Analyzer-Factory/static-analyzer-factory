// R7 (plan 201): the ONLY loop lives in `dead`, which is never called from
// `main`. The reachability refinement scopes the loop-free check to functions
// reachable from `main`, so the unreachable loop is ignored => `true`.
int dead(void) {
    int s = 0;
    int i = 0;
    while (i < 10) {
        s += i;
        i++;
    }
    return s;
}

int main(void) {
    return 0;
}
