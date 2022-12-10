#ifndef A_HPP
#define A_HPP

#include <string_view>

void printString(std::string_view str);

template <typename T>
T incremented(const T& obj)
{
    return T + 1;
}

#endif // A_HPP