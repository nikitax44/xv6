#include "user.h"

//
// wrapper so that it's OK if main() does not call exit().
//
void _start(void) {
  extern int main(void);
  main();
  exit(0);
}
