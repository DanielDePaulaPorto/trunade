---
layout: ../../layouts/Docs.astro
title: Escrevendo aulas
description: Blocos de código, atributos e entrada padrão.
---

# Escrevendo aulas

A aula é Markdown comum. Blocos cercados com uma linguagem configurada viram programas; os demais continuam texto. Esta página é um passeio; a referência completa está em [Sintaxe](/docs/sintaxe/) e as receitas prontas, em [Exemplos](/docs/exemplos/).

## Blocos de código

````markdown
```cpp
#include <iostream>
int main() { std::cout << "Olá, turma!\n"; }
```
````

Linguagens e apelidos que vêm configurados:

| Linguagem | Fence |
|---|---|
| C++ | `cpp`, `c++`, `cxx`, `cc` |
| C | `c` |
| Python | `python`, `py`, `python3` |

## Atributos

Entre chaves, depois da linguagem:

````markdown
```cpp {flags="-O2 -g" timeout=5}
```
````

- `flags`: flags extras para o compilador, além das da configuração.
- `timeout`: tempo limite em segundos para este bloco.

## Moldes: o código que se repete

Para não repetir `#include`, `using namespace std` e a leitura rápida em todo bloco, declare um molde com nome e use-o só nos blocos que quiser. A linha `{code}` (pode vir atrás de um comentário: `//{code}`, `# {code}`) marca onde entra o bloco:

````markdown
```cpp template=maratona
#include <bits/stdc++.h>
using namespace std;
//{code}
```

```cpp-maratona
int main() { cout << gcd(12, 18) << "\n"; }
```
````

- Só usa o molde quem pede por ele: ```` ```cpp-maratona ````. Um ```` ```cpp ```` sem nome continua completo.
- A aula pode ter vários moldes, de várias linguagens.
- O molde não aparece na página. O bloco mostra o nome do molde e uma faixa discreta, "2 linhas do molde acima", que abre o trecho esmaecido e só para leitura.
- A numeração das linhas é a do programa inteiro (o bloco acima começa na linha 3), então os erros do compilador batem com o que se vê.

Todas as regras estão em [Sintaxe](/docs/sintaxe/#moldes).

## Entrada padrão

Há dois jeitos de dar entrada a um programa.

### Fixa, junto com a aula

Um bloco `stdin` logo depois do código (só linhas em branco entre eles):

````markdown
```cpp
#include <iostream>
int main() { int a, b; std::cin >> a >> b; std::cout << a + b << "\n"; }
```

```stdin
2 40
```
````

O programa roda com essa entrada, com tempo limite, e stdout e stderr aparecem separados.

### Casos de teste e saída esperada

Depois do `stdin`, um bloco `expected` diz o que o programa deveria imprimir. Vários pares `stdin`/`expected` em sequência viram vários casos: o programa é compilado uma vez e roda em cada um.

````markdown
```stdin
4 6
```

```expected
12
```

```stdin
3 5
```

```expected
15
```
````

Cada caso ganha ✓ ou ✗ ao lado da entrada, e o terminal mostra a saída de cada um seguida do veredito, como num juiz: `✗ resposta errada na linha 1: esperado «16», saiu «15»`. Espaços no fim das linhas e linhas em branco no fim da saída não contam. Um programa que termina com erro (código diferente de zero, sinal, tempo limite) recebe ✗ sem comparar a saída.

Um `expected` sem `stdin` antes roda com a entrada vazia. Um `stdin` sem `expected` só roda, sem veredito. Entrada e saída esperada também podem ser editadas na página.

### Interativa, na hora

Sem bloco `stdin`, o programa roda num terminal de verdade: a turma digita a entrada no próprio terminal da página. Nesse modo não há tempo limite (o programa pode estar esperando alguém digitar). Use ■ para parar.

## O que aparece embaixo

- **Erro de compilação:** a saída do compilador inteira, colorida.
- **Código de saída diferente de zero**, **sinais** (`Segmentation fault`), **tempo limite** e **limite de saída** aparecem depois da saída do programa.
- O tempo de compilação e o de execução ficam na barra do bloco.

## Editando durante a aula

O código de cada bloco é editável no navegador. `Shift+Enter` roda o bloco com o cursor dentro dele. A edição não grava no `.md`: quando o arquivo muda, só os blocos alterados perdem a edição.
