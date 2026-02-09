int main() {
    int x = 0, y = 100;

    // while-loop with nested if-else chain
    while (x < 10) {
        if (x % 3 == 0) {
            y += x;
        } else if (x % 3 == 1) {
            y -= x;
        } else {
            y *= 2;
        }
        x++;
    }

    // for-loop with early break and continue
    for (int i = 0; i < y; i++) {
        if (i == 42)
            break;
        if (i % 7 == 0)
            continue;
        x += i;
    }

    // switch with fallthrough and default
    switch (x % 5) {
        case 0: y = 1;   break;
        case 1: y = 10;  break;
        case 2: y = 100; break;
        case 3:           // fallthrough
        case 4: y = -1;  break;
        default: y = 0;
    }

    // short-circuit boolean + ternary
    int z = (x > 50 && y > 0) ? x : y;

    return z;
}
