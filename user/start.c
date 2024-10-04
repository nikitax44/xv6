#include "user.h"

//
// wrapper so that it's OK if main() does not call _exit().
//
void _start(void) {
  extern int main(void);
  main();
  _exit(0);
}
