// R7 (plan 201): `read` is an opaque, non-allowlisted external that may block
// indefinitely, so R7 cannot assume it returns => abstain => `unknown`.
extern long read(int fd, void *buf, unsigned long n);

int main(void) {
    char buf[8];
    long r = read(0, buf, sizeof buf);
    return (int) r;
}
