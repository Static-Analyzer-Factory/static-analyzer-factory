// IDE Typestate verification with C++ patterns
// Tests edge function composition through class methods

extern "C" int open(const char*, int);
extern "C" int close(int);
extern "C" int read(int, void*, int);
extern "C" int write(int, const void*, int);

// RAII file wrapper
class File {
public:
    int fd;

    File(const char* path) {
        fd = open(path, 0);  // opened state
    }

    void read_data(char* buf, int size) {
        read(fd, buf, size);  // valid in opened state
    }

    void close_file() {
        close(fd);  // transition to closed
    }

    ~File() {
        // Destructor - might close if not already
    }
};

// Test basic class typestate
void test_class_typestate() {
    File f("test.txt");
    char buf[64];
    f.read_data(buf, 64);
    f.close_file();
    // f is now closed
}

// Test use after close through method
void test_class_use_after_close() {
    File f("data.txt");
    f.close_file();
    char buf[32];
    f.read_data(buf, 32);  // ERROR: use after close
}

// Test edge composition through method chain
class DataProcessor {
public:
    File* file;

    void process() {
        char buf[128];
        file->read_data(buf, 128);  // Needs file in opened state
    }

    void finish() {
        file->close_file();
    }
};

void test_method_chain() {
    File f("input.txt");
    DataProcessor dp;
    dp.file = &f;
    dp.process();  // Valid: file is opened
    dp.finish();   // Closes file
    // dp.process() here would be error
}

// Test multiple file handles
void test_multiple_handles() {
    File f1("file1.txt");
    File f2("file2.txt");

    char buf[32];
    f1.read_data(buf, 32);  // f1 opened
    f1.close_file();        // f1 closed
    f2.read_data(buf, 32);  // f2 still opened - valid
    f2.close_file();        // f2 closed
}

// Test path-sensitive state
void test_path_sensitive(bool cond) {
    File f("path.txt");

    if (cond) {
        f.close_file();  // closed on true path
    }
    // On false path: still opened

    // At merge: state is unknown (joined)
    // Conservative analysis should flag potential issues
}
