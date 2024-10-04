#include "util.h"

u16 bswap16(u16 x) { return (x << 8) | (x >> 8); }
u32 bswap32(u32 x) {
  return ((u32)bswap16(x & 0xffff) << 16) | bswap16(x >> 16);
}
u64 bswap64(u64 x) {
  return ((u64)bswap32(x & 0xffffffff) << 32) | bswap32(x >> 32);
}
