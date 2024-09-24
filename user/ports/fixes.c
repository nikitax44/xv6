#include "user/user.h"

long _lseek(int __fildes, long __offset, int __whence) {
  return _seek(__fildes, __offset, __whence);
}

int _isatty() { return 1; }

int main(int argc, char** argv, char** envp);

static char** environ;

void _start(int argc, char** argv, char** envp) {
  environ = envp;
  _exit(main(argc, argv, envp));
}
