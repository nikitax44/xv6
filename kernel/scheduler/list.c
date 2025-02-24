#include "kernel/scheduler/list.h"
#include "kernel/defs.h"

// double-linked, circular list. double-linked makes remove
// fast. circular simplifies code, because don't have to check for
// empty list in insert and remove.

void lst_link(struct list* a, struct list* b) {
  a->next = b;
  b->prev = a;
}

void lst_init(struct list* lst) {
  lst->next = lst;
  lst->prev = lst;
}

bool lst_empty(struct list* lst) { return lst->next == lst; }

void lst_remove(struct list* e) { lst_link(e->prev, e->next); }

struct list* lst_pop(struct list* lst) {
  if (lst_empty(lst)) {
    panic("lst_pop");
  }
  struct list* p = lst->next;
  lst_remove(p);
  return p;
}

void lst_push(struct list* lst, struct list* p) {
  lst_link(p, lst->next);
  lst_link(lst, p);
}

void lst_print(struct list* lst) {
  for (struct list* p = lst->next; p != lst; p = p->next) {
    printf(" %p", (void*)p);
  }
  printf("\n");
}

void lst_extend_move(struct list* dst, struct list* src) {
  lst_link(src->prev, dst->next);
  lst_link(dst, src->next);
  lst_init(src);
}
