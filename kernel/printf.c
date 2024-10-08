//
// formatted console output -- printf, panic.
//

#include <stdarg.h>

#include "defs.h"
#include "file.h"
#include "fs.h"
#include "memlayout.h"
#include "param.h"
#include "proc.h"
#include "riscv.h"
#include "sleeplock.h"
#include "spinlock.h"
#include "types.h"

volatile int panicked = 0;

// lock to avoid interleaving concurrent printf's.
static struct {
  struct spinlock lock;
  bool            locking;
  bool            initialized;
} pr;

static char digits[] = "0123456789abcdef";

static void printint(i64 xx, int base, int sign) {
  char buf[16];
  int  i;
  u64  x;

  if (sign && (sign = (xx < 0))) {
    x = -xx;
  } else {
    x = xx;
  }

  i = 0;
  do {
    buf[i++] = digits[x % base];
  } while ((x /= base) != 0);

  if (sign) {
    buf[i++] = '-';
  }

  while (--i >= 0) {
    consputc(buf[i]);
  }
}

static void printptr(u64 x) {
  u32 i;
  consputc('0');
  consputc('x');
  for (i = 0; i < (sizeof(u64) * 2); i++, x <<= 4) {
    consputc(digits[x >> (sizeof(u64) * 8 - 4)]);
  }
}

// Print to the console.
int printf(str fmt, ...) {
  va_list ap;
  int     i, cx, c0, c1, c2, locking;
  char*   s;

  if (!pr.initialized) {
    return -1;
  }

  locking = pr.locking;
  if (locking) {
    acquire(&pr.lock);
  }

  va_start(ap, fmt);
  for (i = 0; (cx = fmt[i] & 0xff) != 0; i++) {
    if (cx != '%') {
      consputc(cx);
      continue;
    }
    i++;
    c0 = fmt[i + 0] & 0xff;
    c1 = c2 = 0;
    if (c0) {
      c1 = fmt[i + 1] & 0xff;
    }
    if (c1) {
      c2 = fmt[i + 2] & 0xff;
    }
    if (c0 == 'c') {
      consputc((char)va_arg(ap, int));
    } else if (c0 == 'd') {
      printint(va_arg(ap, int), 10, 1);
    } else if (c0 == 'l' && c1 == 'd') {
      printint(va_arg(ap, u64), 10, 1);
      i += 1;
    } else if (c0 == 'l' && c1 == 'l' && c2 == 'd') {
      printint(va_arg(ap, u64), 10, 1);
      i += 2;
    } else if (c0 == 'u') {
      printint(va_arg(ap, int), 10, 0);
    } else if (c0 == 'l' && c1 == 'u') {
      printint(va_arg(ap, u64), 10, 0);
      i += 1;
    } else if (c0 == 'l' && c1 == 'l' && c2 == 'u') {
      printint(va_arg(ap, u64), 10, 0);
      i += 2;
    } else if (c0 == 'x') {
      printint(va_arg(ap, int), 16, 0);
    } else if (c0 == 'l' && c1 == 'x') {
      printint(va_arg(ap, u64), 16, 0);
      i += 1;
    } else if (c0 == 'l' && c1 == 'l' && c2 == 'x') {
      printint(va_arg(ap, u64), 16, 0);
      i += 2;
    } else if (c0 == 'p') {
      printptr(va_arg(ap, u64));
    } else if (c0 == 's') {
      if ((s = va_arg(ap, char*)) == 0) {
        s = "(null)";
      }
      for (; *s; s++) {
        consputc(*s);
      }
    } else if (c0 == '%') {
      consputc('%');
    } else if (c0 == 0) {
      break;
    } else {
      // Print unknown % sequence to draw attention.
      consputc('%');
      consputc(c0);
    }

#if 0
    switch(c){
    case 'd':
      printint(va_arg(ap, int), 10, 1);
      break;
    case 'x':
      printint(va_arg(ap, int), 16, 1);
      break;
    case 'p':
      printptr(va_arg(ap, u64));
      break;
    case 's':
      if((s = va_arg(ap, char*)) == 0)
        s = "(null)";
      for(; *s; s++)
        consputc(*s);
      break;
    case '%':
      consputc('%');
      break;
    default:
      // Print unknown % sequence to draw attention.
      consputc('%');
      consputc(c);
      break;
    }
#endif
  }
  va_end(ap);

  if (locking) {
    release(&pr.lock);
  }

  return 0;
}

void panic(char* s) {
  pr.locking = 0;
  printf("panic: ");
  printf("%s\n", s);
  panicked = 1; // freeze uart output from other CPUs
  shutdown();
}

void tabulate(u32 n) {
  if (!pr.initialized) {
    return;
  }
  for (; n > 0; --n) {
    consputc(' ');
  }
}

void shutdown() {
  *(volatile u32*)TEST0 = TEST0_SHUTDOWN;
  for (;;)
    ;
}

void reboot() {
  *(volatile u32*)TEST0 = TEST0_REBOOT;
  for (;;)
    ;
}

void printfinit(void) {
  initlock(&pr.lock, "pr");
  pr.locking     = 1;
  pr.initialized = 1;
}
