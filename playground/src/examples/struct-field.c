#include <stdlib.h>

struct Node {
    int value;
    struct Node *next;
};

struct Node *create(int val) {
    struct Node *n = malloc(sizeof(struct Node));
    n->value = val;
    n->next = NULL;
    return n;
}

int main() {
    struct Node *a = create(1);
    struct Node *b = create(2);

    a->next = b;          // a.next -> b
    b->next = create(3);  // b.next -> new node

    int sum = a->value + a->next->value + b->next->value;
    return sum;
}
