typedef int (*BinOp)(int, int);

int add(int a, int b) { return a + b; }
int mul(int a, int b) { return a * b; }

int apply(BinOp fn, int x, int y) {
    return fn(x, y);  // indirect call
}

int main() {
    BinOp op = add;
    int r1 = apply(op, 3, 4);

    op = mul;
    int r2 = apply(op, 3, 4);

    return r1 + r2;
}
