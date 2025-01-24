#include "kernel/fcntl.h"
#include "kernel/file/stat.h"
#include "kernel/types.h"
#include "user.h"

char* gets(char* buf, int max) {
  int  i, cc;
  char c;

  for (i = 0; i + 1 < max;) {
    cc = _read(0, &c, 1);
    if (cc < 1) {
      break;
    }
    buf[i++] = c;
    if (c == '\n' || c == '\r') {
      break;
    }
  }
  buf[i] = '\0';
  return buf;
}

int stat(str n, struct stat* st) {
  int fd;
  int r;

  fd = _open(n, O_RDONLY);
  if (fd < 0) {
    return -1;
  }
  r = _fstat(fd, st);
  _close(fd);
  return r;
}

int atoi(str s) {
  int n;

  n = 0;
  while ('0' <= *s && *s <= '9') {
    n = n * 10 + *s++ - '0';
  }
  return n;
}

int _exec(str path, char** argv) { return _execve(path, argv, (char**)0); }
