// Includes a header from a nested subdir via -Isrc (auto-added), and calls a
// function defined in another nested .cpp (proves recursive source discovery).
#include "greet/greet.hpp"

int main() {
  greet("nested");
  return 0;
}
