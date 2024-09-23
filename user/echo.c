#include "kernel/types.h"
#include "kernel/stat.h"
#include "user/user.h"

int
main(int argc, char *argv[])
{
  int i;

  for(i = 1; i < argc; i++){
    _write(1, argv[i], strlen(argv[i]));
    if(i + 1 < argc){
      _write(1, " ", 1);
    } else {
      _write(1, "\n", 1);
    }
  }
  _exit(0);
}
