#include "user.h"

//
// wrapper so that it's OK if main() does not call _exit().
//
void _start() {
  extern int main();
  main();
  _exit(0);
}
