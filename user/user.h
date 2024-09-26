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
long  _seek(int, long, int);
int   _close(int);
int   _kill(int);
int   _exec(const char*, char**);
int   _execve(const char*, char**, char**);
int   _open(const char*, int);
int   _mknod(const char*, short, short);
int   _unlink(const char*);
int   _fstat(int fd, struct stat*);
int   _link(const char*, const char*);
int   _mkdir(const char*);
int   _chdir(const char*);
int   _dup(int);
int   _getpid(void);
char* _sbrk(int);
int   _sleep(int);
int   _uptime(void);

// ulib.c
int   stat(const char*, struct stat*);
void  fdprintf(int, const char*, ...) __attribute__((format(printf, 2, 3)));
void  printf(const char*, ...) __attribute__((format(printf, 1, 2)));
char* gets(char*, int max);
int   atoi(const char*);

// umalloc.c
void* malloc(uint);
void  free(void*);
