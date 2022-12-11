#include <some/library.hpp>

int main()
{
    using namespace std::string::literals;
    using namespace std::string_view::literals;
    printString("C string");
    printString("C++ string"s);
    printString("C++ string_view"sv);
    return 0;
}