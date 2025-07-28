# Game Translator

Aplicação para tradução de aplicativos através de OCR (Reconhecimento ótico de caracteres)

## Instalação

É necessário instalar o [Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html) para reconhecimento ótico de caracteres.  

## Funcionalidades

- [x] Traduzir áreas selecionadas na aplicação escolhida
- [x] Traduzir toda na aplicação selecionada
- [x] Criação de perfis
- [x] Fazer com que texto se encaixe na área devida
- [ ] Selectionar Monitor para tradução

## Desenvolvimento

Siga as instruções para instalação do [GTK4-rs](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html), eu recomendo utilizar MSYS2.  
O aplicativo usa Libadwaita então é necessário instala-lo no sistema. Para isso basta utilizar o seguinte comando caso tenha feito a instalação seguindo MSYS2:

```sh
pacman -Syu mingw-w64-x86_64-libadwaita mingw-w64-i686-libadwaita
```

Ou pode seguir o [guia](https://gtk-rs.org/gtk4-rs/stable/latest/book/libadwaita.html#windows).
