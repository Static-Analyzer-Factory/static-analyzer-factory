// RAII Resource Management
// Constructor acquires a file resource, destructor releases it.
// Analysis must track interprocedural flow through ctor/dtor to
// verify proper resource lifecycle.
//
// Expected finding: resource acquired in ctor, released in dtor
#include <cstdio>

class FileHandle {
    FILE *fp;
public:
    FileHandle(const char *path) {
        fp = fopen(path, "r");               // SOURCE: resource acquisition
    }
    int read_byte() {
        if (fp) return fgetc(fp);
        return -1;
    }
    ~FileHandle() {
        if (fp) fclose(fp);                  // SINK: resource release in dtor
    }
};

int main() {
    FileHandle fh("/etc/hostname");
    int ch = fh.read_byte();
    printf("first byte: %d\n", ch);
    return 0;
    // destructor called here, closing file
}
