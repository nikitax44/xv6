#include "kernel/types.h"
#include <string.h>
#define stderr 2

struct stat;

// system calls
int   _fork(void);
void  _exit(int) __attribute__((noreturn));
int   _wait(int*);
int   _pipe(int*);
int   _write(int, const void*, int);
int   _read(int, void*, int);
isize _seek(int, isize, int);
int   _close(int);
int   _kill(int);
int   _exec(str, char**);
int   _execve(str, char**, char**);
int   _open(str, int);
int   _mknod(str, short, short);
int   _unlink(str);
int   _fstat(int fd, struct stat*);
int   _link(str, str);
int   _mkdir(str);
int   _chdir(str);
int   _dup(int);
int   _getpid(void);
char* _sbrk(int);
int   _sleep(int);
int   _uptime(void);

// ulib.c
int   stat(str, struct stat*);
void  fdprintf(int, str, ...) __attribute__((format(printf, 2, 3)));
void  printf(str, ...) __attribute__((format(printf, 1, 2)));
char* gets(char*, int max);
int   atoi(str);
