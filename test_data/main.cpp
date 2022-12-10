#include "inner/a.hpp"
int main()
{
    using namespace std::string_literals;
    using namespace std::string_view_literals;
    printString("Hello world");
    printString("Constructed string"s);
    printString("String view"sv);
    return 0;
}