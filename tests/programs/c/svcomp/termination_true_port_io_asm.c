// Termination TRUE with straight-line inline asm (port I/O, as in Linux
// drivers): `inb`/`outb` neither loop nor jump, so they do not block the proof.
static unsigned char inb(unsigned short port) {
    unsigned char value;
    __asm__ volatile("inb %w1, %b0" : "=a"(value) : "Nd"(port));
    return value;
}
static void outb(unsigned char value, unsigned short port) {
    __asm__ volatile("outb %b0, %w1" : : "a"(value), "Nd"(port));
}
extern int __VERIFIER_nondet_int(void);
int main(void) {
    if (__VERIFIER_nondet_int()) {
        outb(1, 0x61);
        return inb(0x61);
    }
    return 0;
}
