// DDA Recursion Test
// Tests: Recursive function handling and context collapse.
// Expected: DDA should detect recursion and handle context appropriately.

#include <cstdlib>

struct Node {
    int value;
    Node* next;
};

// Recursive function to traverse a linked list
int sum_list(Node* node) {
    if (node == nullptr) {
        return 0;
    }
    return node->value + sum_list(node->next);
}

int main() {
    // Build a small linked list
    Node* n1 = new Node{1, nullptr};
    Node* n2 = new Node{2, n1};
    Node* n3 = new Node{3, n2};

    // Recursive traversal
    int total = sum_list(n3);

    delete n1;
    delete n2;
    delete n3;

    return total;
}
