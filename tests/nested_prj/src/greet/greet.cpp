#include "greet/greet.hpp"
#include <iostream>

void greet(std::string_view who) {
  std::cout << "hello, " << who << "\n";
}
