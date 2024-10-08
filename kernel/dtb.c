#include "dtb.h"
#include "defs.h"
#include "string.h"
#include "types.h"

#define FDT_BEGIN_NODE        0x01000000
#define FDT_END_NODE          0x02000000
#define FDT_PROP              0x03000000
#define FDT_NOP               0x04000000
#define FDT_END               0x09000000
#define ROUNDUP(value, align) (((u64)(value) + (align) - 1) & ~((align) - 1))

#ifndef DEBUG
#define tabulate(n)
#define printf(...)
#endif

// FIXME: awful function
u32 harts(struct fdt_header* dtb) {
  char* string_base = ((char*)dtb) + bswap32(dtb->off_dt_strings);
#define STR(offset) (string_base + (offset))
  u32* ptr = (u32*)(((u8*)dtb) + bswap32(dtb->off_dt_struct));

  char* node_name;
  u32   tab = 0, len, harts = 0;

  bool parsing = true;
  while (parsing) {
    switch (*ptr++) {
    case FDT_NOP:
      break;
    case FDT_END:
      parsing = false;
      break;
    case FDT_BEGIN_NODE:
      node_name = (char*)ptr;
      ptr       = (u32*)ROUNDUP(node_name + strlen(node_name) + 1, sizeof(u32));
      tabulate(tab * 2);
      printf("%s:\n", node_name);
      tab++;
      break;
    case FDT_END_NODE:
      node_name = NULL;
      if (tab-- == 0) {
        return -3;
      }
      break;
    case FDT_PROP:
      len             = bswap32(*ptr++);
      char* prop_name = STR(bswap32(*ptr++));
      if (strcmp(prop_name, "device_type") == 0) {
        if (strcmp((char*)ptr, "cpu") == 0) {
          harts++;
        }
      }

      tabulate(tab * 2);
      printf("%s = ...(%u)\n", prop_name, len);
      ptr += ROUNDUP(len, sizeof(u32)) / sizeof(u32);

      break;
    default:
      return -1;
    }
  }

  return harts;
}
