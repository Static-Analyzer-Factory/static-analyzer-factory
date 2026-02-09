// Field-sensitive pointer analysis with structs and linked lists
//
// This program demonstrates how field sensitivity affects PTA precision.
// With field sensitivity, SAF tracks each struct field separately:
//   points_to(pair.first) = {a}
//   points_to(pair.second) = {b}
//
// Without field sensitivity, all fields merge:
//   points_to(pair) = {a, b}  -- overly conservative, spurious aliases
//
// The linked list tests the depth limit of field sensitivity:
// n1->next->next->next traverses three levels deep, which may exceed
// the configured max_depth and cause the analysis to merge nodes.

#include <stdio.h>
#include <stdlib.h>

struct Pair {
    int *first;
    int *second;
};

struct Node {
    int data;
    struct Node *next;
};

void print_pair(const struct Pair *p) {
    printf("pair: first=%d, second=%d\n", *p->first, *p->second);
}

void print_list(const struct Node *head) {
    const struct Node *cur = head;
    while (cur) {
        printf("node: %d\n", cur->data);
        cur = cur->next;
    }
}

int main(void) {
    int a = 10, b = 20, c = 30;

    // --- Field sensitivity with struct Pair ---

    // first and second point to different objects
    struct Pair pair;
    pair.first = &a;
    pair.second = &b;

    // With field sensitivity:
    //   points_to(pair.first) = {a}
    //   points_to(pair.second) = {b}
    //   no_alias(pair.first, pair.second) = true
    // Without field sensitivity:
    //   points_to(pair) = {a, b} -- overly conservative

    print_pair(&pair);

    // --- Second pair to show distinct tracking ---
    struct Pair pair2;
    pair2.first = &c;
    pair2.second = &a;  // shares 'a' with pair.first

    // may_alias(pair.first, pair2.second) = true (both -> a)
    // no_alias(pair.first, pair2.first) = true (a vs c)

    print_pair(&pair2);

    // --- Linked list: tests depth of field sensitivity ---
    struct Node *n1 = (struct Node *)malloc(sizeof(struct Node));
    struct Node *n2 = (struct Node *)malloc(sizeof(struct Node));
    struct Node *n3 = (struct Node *)malloc(sizeof(struct Node));

    n1->data = 1;
    n1->next = n2;
    n2->data = 2;
    n2->next = n3;
    n3->data = 3;
    n3->next = NULL;

    // Traverse: n1 -> n2 -> n3 -> NULL
    // At max_depth=2, the analysis may merge n2 and n3's next fields
    print_list(n1);

    free(n3);
    free(n2);
    free(n1);

    return 0;
}
