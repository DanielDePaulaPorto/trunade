# Trunade! — teste manual

Rode cada bloco com `repl: run` (ctrl-shift-enter).

## Saída simples

```cpp
#include <iostream>

int main() {
    std::cout << "Olá, Trunade!\n";
}
```

## Erro de compilação

```cpp
int main() {
    return x;
}
```

## Código de saída e stderr

```cpp
#include <iostream>

int main() {
    std::cerr << "algo deu errado\n";
    return 3;
}
```

## Segfault

```cpp
int main() {
    volatile int* p = nullptr;
    return *p;
}
```

## Laço infinito (deve estourar o tempo limite)

```cpp
int main() {
    for (;;) {}
}
```
