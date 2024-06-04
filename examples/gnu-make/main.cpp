import bar;

#include <iostream>

// extern "C" auto
// b() -> char const*;

auto
main() -> int
{
  std::cout << b() << "\n";
  return 0;
}
