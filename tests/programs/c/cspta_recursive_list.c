/**
 * cspta_recursive_list.c — Test CS-PTA with recursive function.
 *
 * traverse() recursively walks a linked list. As a recursive function,
 * it should be collapsed to empty context (SCC) to ensure termination.
 */
#include <stdlib.h>

struct Node {
    int value;
    struct Node *next;
};

struct Node *create_node(int val) {
    struct Node *n = (struct Node *)malloc(sizeof(struct Node));
    if (n) {
        n->value = val;
        n->next = NULL;
    }
    return n;
}

int traverse(struct Node *head) {
    if (!head) return 0;
    return head->value + traverse(head->next);  // recursive call
}

int main() {
    struct Node *a = create_node(1);
    struct Node *b = create_node(2);
    struct Node *c = create_node(3);

    a->next = b;
    b->next = c;

    int sum = traverse(a);

    free(a);
    free(b);
    free(c);
    return sum;
}
