---
layout: ../../layouts/Docs.astro
title: Na sala
description: Atalhos de teclado, tema, fonte e largura do texto.
---

# Na sala

A página da aula foi pensada para o telão: fundo branco no tema claro, nenhuma barra de ferramentas fixa, e tudo no teclado.

## Atalhos

| Tecla | Ação |
|---|---|
| <kbd>T</kbd> | índice de seções (<kbd>→</kbd> abre as subseções) |
| <kbd>Espaço</kbd>, <kbd>↓</kbd> ou <kbd>Page Down</kbd> | próxima seção, com rolagem suave |
| <kbd>⇧ Espaço</kbd>, <kbd>↑</kbd> ou <kbd>Page Up</kbd> | seção anterior |
| <kbd>+</kbd> <kbd>−</kbd> <kbd>0</kbd> | aumentar, diminuir, restaurar a fonte |
| <kbd>W</kbd> | largura do texto: estreita ou página inteira |
| <kbd>F</kbd> | tela cheia (<kbd>Esc</kbd> ou <kbd>F</kbd> sai) |
| <kbd>Z</kbd> | amplia o bloco de código em uso (ou o mais perto do meio da tela) até ocupar a tela; <kbd>Esc</kbd> volta |
| <kbd>R</kbd> | rodar tudo: todos os blocos, em ordem |
| <kbd>D</kbd> | tema: automático, claro, escuro |
| <kbd>⌘P</kbd> ou <kbd>Ctrl+P</kbd> | exportar PDF |
| <kbd>O</kbd> | configurações (também no ☰) |
| <kbd>⇧ Enter</kbd> | rodar o bloco, com o cursor no código |
| <kbd>Esc</kbd> | sair do código ou do terminal |

Os atalhos não valem dentro do código, do terminal e de campos de texto.

## Configurações

O ☰ no canto abre o painel com:

- **Tamanho da fonte.** Muda só o conteúdo da página. O zoom do navegador continua funcionando e se multiplica com ele.
- **Tema.** Automático (segue o sistema), claro ou escuro.
- **Largura do texto.** Estreita, mais confortável para ler, ou a página inteira, para código largo.
- **Cores do código.** Monokai escuro (padrão), Monokai claro, ou seguir o tema da página.
- **Fontes.** Títulos: Gloock (padrão), IBM Plex Mono, Fraunces, Cormorant Garamond, Young Serif. Texto: JetBrains Mono (padrão), Fira Code, Inter, Open Sans. Código: JetBrains Mono (padrão) ou Fira Code.
- **Idioma.** Português ou inglês.

As escolhas ficam em `~/.config/trunade/preferences.toml` e valem para todas as aulas, em qualquer porta. O arquivo pode ser editado à mão.

## Rodar tudo

Em ☰ › Rodar e exportar, **Rodar tudo** (ou <kbd>R</kbd>) roda em ordem todos os blocos, para conferir a aula antes da sala ou antes de exportar. Blocos sem `stdin` rodam uma vez com a entrada vazia e com tempo limite: quem não lê nada roda normalmente, e quem esperaria alguém digitar termina em vez de travar a fila. Uma barra embaixo mostra o progresso e, no fim, quantos passaram e quais falharam (compilação, erro de execução, saída diferente da esperada); clicar na linha leva ao bloco, abrindo o passo se ele estiver dentro de um.

## Exportar PDF

No ☰, **Exportar PDF** (ou <kbd>⌘P</kbd>) abre a impressão do navegador; escolha "Salvar como PDF". O arquivo sai como a aula está na tela, com as cores, o código editado e as saídas já executadas, além de todos os passos abertos e as respostas das perguntas marcadas. O nome sugerido é o do `.md`.

## Exportar HTML

Em ☰ › Rodar e exportar, **Exportar HTML** grava um único arquivo `.html` ao lado do `.md` (com sufixo `-1`, `-2`… se o nome já existir), sem perguntar nada. Ele abre em qualquer navegador, sem o Trunade! e sem internet: a página, as fontes em uso, fórmulas, diagramas e imagens vão dentro do arquivo (cerca de 2 MB).

- O ▶ reproduz a última execução de cada bloco, com as cores e os vereditos, como se rodasse. Um bloco que não foi rodado fica sem saída e avisa ao clicar.
- O tema do arquivo é **claro** por padrão; escuro é uma opção ao lado do botão.
- Com **Rodar todos os blocos antes** (marcada por padrão), todo bloco roda antes de gravar, então nenhum fica sem saída. Desmarcada, vai a última execução de cada bloco, e um bloco nunca rodado fica sem saída.
- O arquivo fica como estava na tela, como um PDF: mesmo tema, fontes e largura, código com as edições, sem painel de configurações e sem edição. Valem a navegação (<kbd>T</kbd>, <kbd>Espaço</kbd>, setas), <kbd>F</kbd>, <kbd>Z</kbd> e <kbd>R</kbd>.
- As respostas das perguntas continuam ocultas até clicar, como na aula.

## O símbolo no canto

O símbolo no cabeçalho mostra se há programa rodando: o rotor gira e os olhos acendem enquanto compila ou executa, e dá uma volta para trás quando termina.
