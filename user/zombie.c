// Create a zombie process that
// must be reparented at exit.

#include "user.h"

int main(void) {
  if (_fork() > 0) {
    _sleep(5); // Let child exit before parent.
  }
  _exit(0);
}
