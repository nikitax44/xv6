//
// Console input and output, to the uart.
// Reads are line at a time.
// Implements special input characters:
//   newline -- end of line
//   control-h -- backspace
//   control-u -- kill line
//   control-d -- end of file
//   control-p -- print process list
//

#include "defs.h"
#include "file/file.h"
#include "scheduler/proc.h"
#include "types.h"
#include "util/spinlock.h"

#define BACKSPACE 0x100
#define C(x)      ((x) - '@') // Control-x

usize size_of_last_char(char* start, char* end);

//
// send one character to the uart.
// called by printf(), and to echo input characters,
// but not from write().
//
void consputc(int c) {
  if (c == BACKSPACE) {
    // if the user typed backspace, overwrite with a space.
    uartputc_sync('\b');
    uartputc_sync(' ');
    uartputc_sync('\b');
  } else {
    uartputc_sync(c);
  }
}

struct {
  struct spinlock lock;

  // input
#define INPUT_BUF_SIZE 128
  char buf[INPUT_BUF_SIZE];
  u32  r; // Read index
  u32  w; // Write index
  u32  e; // Edit index
} cons;

//
// user write()s to the console go here.
//
int consolewrite(int user_src, u64 src, u32 n) {
  u32 i;

  for (i = 0; i < n; i++) {
    char c;
    if (either_copyin(&c, user_src, src + i, 1) == -1) {
      break;
    }
    uartputc(c);
  }

  return i;
}

//
// user read()s from the console go here.
// copy (up to) a whole input line to dst.
// user_dist indicates whether dst is a user
// or kernel address.
//
int consoleread(int user_dst, u64 dst, u32 n) {
  u32  target;
  int  c;
  char cbuf;

  target = n;
  acquire(&cons.lock);
  while (n > 0) {
    // wait until interrupt handler has put some
    // input into cons.buffer.
    while (cons.r == cons.w) {
      if (killed(myproc())) {
        release(&cons.lock);
        return -1;
      }
      sleep(&cons.r, &cons.lock);
    }

    c = cons.buf[cons.r++ % INPUT_BUF_SIZE];

    if (c == C('D')) { // end-of-file
      if (n < target) {
        // Save ^D for next time, to make sure
        // caller gets a 0-byte result.
        cons.r--;
      }
      break;
    }

    // copy the input byte to the user-space buffer.
    cbuf = c;
    if (either_copyout(user_dst, dst, &cbuf, 1) == -1) {
      break;
    }

    dst++;
    --n;

    if (c == '\n') {
      // a whole line has arrived, return to
      // the user-level read().
      break;
    }
  }
  release(&cons.lock);

  return target - n;
}

//
// the console input interrupt handler.
// uartintr() calls this for input character.
// do erase/kill processing, append to cons.buf,
// wake up consoleread() if a whole line has arrived.
//
void consoleintr(int c) {
  acquire(&cons.lock);

  switch (c) {
  case C('P'): // Print process list.
    printf("\nfree RAM: %lu blocks\n", free_pages());
    procdump();
    break;
  case C('Q'):
    printf("\nctrl+Q: shutdown\n");
    shutdown();
    break;
  case C('R'):
    printf("\nctrl+R: reboot\n");
    reboot();
    break;
  case C('N'):
    testpanic();
    break;
  case C('C'): // Kill process
    kill_all();
    // fallthrough
  case C('U'): // Kill line.
    while (cons.e != cons.w &&
           cons.buf[(cons.e - 1) % INPUT_BUF_SIZE] != '\n') {
      cons.e--;
      consputc(BACKSPACE);
    }
    break;
  case C('H'): // Backspace
  case '\x7f': // Delete key
    if (cons.e != cons.w) {
      if (cons.w < cons.e) {
        cons.e -= size_of_last_char(&cons.buf[cons.w], &cons.buf[cons.e]);
      } else {
        cons.e -= size_of_last_char(&cons.buf[0], &cons.buf[cons.e]);
      }
      cons.e %= INPUT_BUF_SIZE;
      consputc(BACKSPACE);
    }
    break;
  case '\x1b': // Escape key \e
    break;
  case '\t':
    break;
  default:
    if (c != 0 && cons.e - cons.r < INPUT_BUF_SIZE) {
      c = (c == '\r') ? '\n' : c;

      // echo back to the user.
      consputc(c);

      // store for consumption by consoleread().
      cons.buf[cons.e++ % INPUT_BUF_SIZE] = c;

      if (c == '\n' || c == C('D') || cons.e - cons.r == INPUT_BUF_SIZE) {
        // wake up consoleread() if a whole line (or end-of-file)
        // has arrived.
        cons.w = cons.e;
        wakeup(&cons.r);
      }
    }
    break;
  }

  release(&cons.lock);
}

void consoleinit(void) {
  initlock(&cons.lock, "cons");

  uartinit();

  // connect read and write system calls
  // to consoleread and consolewrite.
  devsw[CONSOLE].read  = consoleread;
  devsw[CONSOLE].write = consolewrite;
}
