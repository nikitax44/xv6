#include "user/user.h"
int _isatty() { return 1; }

int main(int argc, char** argv, char** envp);

extern char** environ;

void _start(int argc, char** argv, char** envp) {
  environ = envp;
  _exit(main(argc, argv, envp));
}
