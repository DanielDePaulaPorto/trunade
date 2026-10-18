---
layout: ../../layouts/Docs.astro
title: Sintaxe
description: Referência de tudo o que o Trunade! entende num arquivo .md, com o Markdown e como ele aparece na página.
---

# Sintaxe

A aula é Markdown comum (CommonMark com as extensões do GitHub). O Trunade! dá significado a alguns blocos cercados (*fences*) e a dois comentários HTML. Todo o resto é texto.

Cada exemplo mostra o Markdown e, logo abaixo, como ele aparece na página do Trunade!, já executado.

## Resumo

| Escreva | Vira |
|---|---|
| ```` ```cpp ```` | bloco executável (qualquer linguagem configurada) |
| ```` ```cpp {flags="-O2" timeout=5} ```` | bloco com flags extras e tempo limite próprio |
| ```` ```cpp template=nome ```` | declara um molde escondido |
| ```` ```cpp-nome ```` | bloco que usa o molde `nome` |
| ```` ```stdin ```` | entrada fixa do bloco acima (um caso de teste) |
| ```` ```expected ```` | saída esperada do caso acima, com veredito ✓/✗ |
| ```` ```quiz ```` | pergunta de múltipla escolha |
| `<!-- step: Título -->` … `<!-- /steps -->` | exercício em passos |
| `$x$`, `$$x$$`, ```` ```math ```` | fórmula (KaTeX) |
| ```` ```mermaid ```` | diagrama |
| `> [!NOTE]`, `> [!TIP]`… | caixa de destaque |
| `<img>`, `<iframe>`, `<div>`… | HTML, como está |
| `data-reveal` num elemento HTML | escondido até a aula chegar à seção dele |

Qualquer outro fence (```` ```text ````, ```` ```bash ````, ```` ```json ````) continua sendo só código colorido, sem ▶.

## Blocos executáveis

Um fence cuja linguagem está configurada vira uma caixa editável com ▶. Cada bloco é um **programa completo**, compilado do zero; nada passa de um bloco para outro.

````markdown preview=bloco
```cpp
#include <iostream>
int main() {
    std::cout << "Olá, turma!\n";
}
```
````

| Linguagem | Nomes aceitos no fence |
|---|---|
| C++ | `cpp`, `c++`, `cxx`, `cc` |
| C | `c` |
| Python | `python`, `py`, `python3` |

Outras linguagens entram pela [configuração](/docs/configuracao/). Maiúsculas não importam: `C++` e `c++` são o mesmo.

## Atributos

Entre chaves, depois da linguagem. Só existem dois.

| Atributo | Exemplo | Efeito |
|---|---|---|
| `flags` | `flags="-O2 -DDEBUG"` | flags somadas às da configuração, separadas por espaço |
| `timeout` | `timeout=1.5` | tempo limite em segundos (só vale com entrada fixa) |

````markdown preview=atributos
```cpp {flags="-DDEBUG -O2" timeout=2}
#include <iostream>
int main() {
#ifdef DEBUG
    std::cout << "modo debug\n";
#endif
    std::cout << "pronto\n";
}
```

```expected
modo debug
pronto
```
````

Aspas são necessárias quando o valor tem espaço. As flags aparecem na barra do bloco.

## Moldes

Um molde é o código que se repete (cabeçalhos, `using namespace std`, leitura rápida, um `main` fixo). Ele tem um **nome**, é declarado uma vez e fica escondido. Só usa o molde o bloco que pede por ele, com o nome depois da linguagem: ```` ```cpp-maratona ````. Um bloco sem nome continua completo.

A linha `{code}` marca onde o bloco entra no molde:

````markdown preview=molde
```cpp template=maratona
#include <bits/stdc++.h>
using namespace std;
//{code}
```

```cpp-maratona
int main() {
    cout << gcd(12, 18) << '\n';
}
```

```cpp
#include <iostream>
int main() {
    std::cout << "este bloco é completo\n";
}
```
````

Regras:

- Declare com `template=nome` depois da linguagem. O nome vai sem espaços (letras, números, `_`).
- Use com `linguagem-nome`: ```` ```cpp-maratona ````, ```` ```c++-maratona ````, ```` ```python-io ````. Apelidos da linguagem valem.
- O molde precisa ser declarado **antes** do bloco que o usa. Declarar de novo o mesmo nome muda o molde para os blocos seguintes.
- Moldes são por linguagem: um `io` de C++ e um `io` de Python convivem.
- O marcador é uma linha só com `{code}`, que pode vir atrás de um comentário da linguagem: `//{code}`, `// {code}`, `# {code}`. Sem marcador, o molde inteiro vai **antes** do bloco.
- A declaração não aparece na página. O bloco mostra o nome do molde na barra e uma faixa discreta, **"2 linhas do molde acima"**, que abre o trecho esmaecido e só para leitura.
- A numeração das linhas é a do programa inteiro, então `main.cpp:4:5: error` aponta a linha 4 que você vê.
- Erros de digitação aparecem na própria página, numa caixa de aviso: um `template` sem nome, ou um bloco que pede um molde não declarado.

### Molde com código depois

O que vem depois de `{code}` também é compilado. Assim, o bloco pode ter só a função que importa:

````markdown preview=molde-depois
```cpp template=testes
#include <bits/stdc++.h>
using namespace std;

//{code}

int main() {
    int t;
    cin >> t;
    while (t--) solve();
}
```

```cpp-testes
void solve() {
    long long a, b;
    cin >> a >> b;
    cout << gcd(a, b) << '\n';
}
```

```stdin
2
12 18
7 5
```

```expected
6
1
```
````

### Vários moldes na mesma aula

````markdown preview=moldes
```cpp template=simples
#include <bits/stdc++.h>
using namespace std;
//{code}
```

```cpp template=rapido
#include <bits/stdc++.h>
using namespace std;
//{code}
int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    resolve();
}
```

```python template=io
import sys
input = sys.stdin.readline
# {code}
```

```cpp-simples
int main() { cout << "simples\n"; }
```

```cpp-rapido
void resolve() { cout << "rápido\n"; }
```

```python-io
print("python")
```
````

## Entrada e casos de teste

Blocos `stdin` e `expected` logo **depois** de um bloco executável, separados dele só por linhas em branco, são os casos de teste dele. Com qualquer texto no meio, viram blocos comuns.

### Entrada fixa

````markdown preview=entrada
```cpp
#include <iostream>
int main() {
    int a, b;
    std::cin >> a >> b;
    std::cout << a + b << "\n";
}
```

```stdin
2 40
```
````

Com entrada fixa, o programa roda em *pipes*: stdout e stderr aparecem separados (stderr em vermelho) e vale o tempo limite. Um `stdin` vazio roda sem entrada e com tempo limite.

### Saída esperada e vários casos

Cada `stdin` abre um caso; o `expected` seguinte completa esse caso. O programa é compilado **uma vez** e roda em cada caso, na ordem. Cada caso ganha ✓ ou ✗, e no ✗ o terminal diz onde a saída diverge, como um juiz:

````markdown preview=casos
```cpp
#include <iostream>
int main() {
    int n, soma = 0;
    std::cin >> n;
    for (int i = 0; i < n; i++) {
        int x;
        std::cin >> x;
        soma += x;
    }
    std::cout << soma << "\n";
}
```

```stdin
3
1 2 3
```

```expected
6
```

```stdin
2
2000000000 2000000000
```

```expected
4000000000
```
````

A comparação:

- ignora espaços no fim das linhas e linhas em branco no fim da saída;
- considera espaços no começo das linhas e qualquer outra diferença;
- só compara se o programa terminou com código 0. Erro de execução, sinal ou tempo limite já dão ✗.

| Sequência depois do bloco | Casos |
|---|---|
| `stdin` | 1 caso, sem veredito |
| `stdin`, `expected` | 1 caso com veredito |
| `expected` | 1 caso com entrada vazia, com veredito |
| `stdin`, `stdin` | 2 casos sem veredito |
| `stdin`, `expected`, `stdin` | 2 casos: o primeiro com veredito, o segundo sem |

### Interativo

Sem nenhum `stdin` ou `expected`, o programa roda num terminal de verdade: quem estiver no teclado digita no próprio terminal da página. Nesse modo não há tempo limite (use ■ para parar) e stdout e stderr vêm misturados, como num terminal.

````markdown preview=interativo input="Ana"
```cpp
#include <iostream>
#include <string>
int main() {
    std::string nome;
    std::cout << "Qual o seu nome? ";
    std::cin >> nome;
    std::cout << "Olá, " << nome << "!\n";
}
```
````

## Perguntas

````markdown preview=quiz reveal
```quiz
Qual a complexidade da busca binária num vetor de $n$ elementos?
- [ ] $O(n)$
- [x] $O(\log n)$
- [ ] $O(n \log n)$
```
````

- Tudo antes da lista é o enunciado, em Markdown (fórmulas e `código` valem).
- `- [x]` marca as corretas, `- [ ]` as erradas.
- Uma correta: escolha única. Mais de uma: múltipla escolha, com caixas quadradas.

````markdown preview=quiz-multipla
```quiz
Quais estruturas dão acesso ao menor elemento em $O(1)$?
- [x] `priority_queue` com `greater<>`
- [x] `set` (com `begin()`)
- [ ] `vector` não ordenado
- [ ] `unordered_set`
```
````

As respostas ficam ocultas até alguém clicar em **Ver resposta**.

## Exercício em passos

Dois comentários HTML dividem um trecho em passos, com Anterior e Próximo. A página mostra um passo por vez:

~~~markdown preview=passos
<!-- step: Enunciado -->
Leia $n$ e imprima a soma de $1$ a $n$.

<!-- step: Dica -->
Existe uma fórmula fechada: $\frac{n(n+1)}{2}$.

<!-- step: Solução -->
```cpp
#include <iostream>
int main() {
    long long n;
    std::cin >> n;
    std::cout << n * (n + 1) / 2 << "\n";
}
```

```stdin
100
```

```expected
5050
```
<!-- /steps -->
~~~

- `<!-- step: Título -->` abre um passo; o título aparece na trilha do topo.
- `<!-- /steps -->` fecha o exercício. Sem ele, os passos vão até o fim do arquivo.
- Dentro dos passos vale tudo: blocos executáveis, casos, perguntas, fórmulas.

## Fórmulas

KaTeX, desenhado localmente:

````markdown preview=formulas
A busca binária faz $\lceil \log_2 n \rceil$ comparações.

$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$

```math
T(n) = 2\,T(n/2) + O(n) \implies T(n) = O(n \log n)
```
````

## Diagramas

Mermaid, carregado só quando a aula tem diagrama e redesenhado ao trocar o tema:

````markdown preview=diagrama
```mermaid
graph LR
  A[ler n] --> B{n > 0?}
  B -- sim --> C[processar]
  C --> A
  B -- não --> D[fim]
```
````

````markdown preview=sequencia
```mermaid
sequenceDiagram
  participant P as Programa
  participant J as Juiz
  J->>P: entrada
  P->>J: saída
  J-->>J: compara com a esperada
```
````

## Caixas de destaque

````markdown preview=alertas
> [!NOTE]
> O `int` vai até 2 147 483 647.

> [!TIP] Dica de prova
> Na dúvida, `long long`.

> [!WARNING]
> `endl` esvazia o buffer a cada linha: em saídas grandes, use `'\n'`.

> [!CAUTION]
> Ler com `cin` sem `ios::sync_with_stdio(false)` pode estourar o tempo.
````

Tipos: `NOTE`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`. O texto depois do tipo vira o título da caixa.

## Markdown do GitHub

Tabelas, ~~riscado~~, links automáticos, listas de tarefas e notas de rodapé:

````markdown preview=github
| Estrutura | Inserção | Menor |
|---|---|---|
| `set` | $O(\log n)$ | $O(1)$ |
| `vector` | $O(1)$ | $O(n)$ |

- [x] ler o enunciado
- [ ] testar o caso extremo

Veja a documentação[^ref].

[^ref]: https://en.cppreference.com
````

## HTML, imagens e páginas

HTML é aceito como está. Arquivos ao lado do `.md` são servidos pelo próprio Trunade!:

```html
<img src="figuras/arvore.png" alt="Árvore de recursão" width="480">

<div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem">
  <div>Antes</div>
  <div>Depois</div>
</div>
```

Iframes ocupam a largura do texto. Uma página **local** é reduzida para caber e a altura acompanha o conteúdo. Uma página **externa** mais larga que o texto precisa dizer a largura que usa, e o Trunade! a reduz:

```html
<iframe src="visualizacao.html"></iframe>
<iframe src="https://visualgo.net/en/sorting" data-width="1200" height="640"></iframe>
```

## Revelar ao chegar na seção

Para guardar uma imagem impactante, uma resposta ou um spoiler até a hora certa, ponha `data-reveal` no elemento. Ele fica invisível, com o espaço reservado (a página não pula), até o título da seção dele chegar ao topo da tela. Aí aparece, com uma transição curta, e não some mais.

````markdown
## Seção 1

Texto que vem antes.

## Seção 2

<img src="img/impacto.gif" alt="" height="300" data-reveal>

<div data-reveal>

O **segredo** também pode ser texto em Markdown, lista, fórmula ou tabela.

</div>
````

- Vale em qualquer elemento HTML: `<img>`, `<div>`, `<svg>`, `<iframe>`, `<p>`. Uma imagem em Markdown (`![](x.gif)`) precisa virar `<img … data-reveal>`.
- A "seção" é o último título (`#`, `##` ou `###`) antes do elemento. Sem título antes, ele aparece logo.
- Na última seção da aula, que talvez não consiga chegar ao topo, o elemento aparece quando a página chega ao fim.
- Um GIF começa do primeiro quadro na hora em que aparece.
- Dentro de `<div data-reveal>`, deixe uma linha em branco depois da abertura e antes do fechamento, para o Markdown funcionar. Blocos de código não entram: eles continuam visíveis.
- No PDF tudo sai visível. No HTML exportado, o comportamento é o mesmo da aula.

Veja os [exemplos](/docs/exemplos/) para aulas inteiras montadas com tudo isso.
