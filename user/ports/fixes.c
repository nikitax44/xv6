#include "user/user.h"

long _lseek(int __fildes, long __offset, int __whence) {
  return _seek(__fildes, __offset, __whence);
}

int _isatty() { return 1; }

int main(int argc, char** argv);

void _start(int argc, char** argv) { _exit(main(argc, argv)); }
