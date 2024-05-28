# Game Translator (WIP)

Modulos utilizados para aplicações mobile

## Desenvolvimento

Siga as instruções para instalação do [GTK4-rs](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html), eu recomendo utilizar MSYS2.  
O aplicativo usa Libadwaita então é necessário instala-lo no sistema. Para isso basta utilizar o seguinte comando caso tenha feito a instalação seguindo MSYS2:

```sh
pacman -Syu mingw-w64-x86_64-libadwaita mingw-w64-i686-libadwaita
```

Ou pode seguir o [guia](https://gtk-rs.org/gtk4-rs/stable/latest/book/libadwaita.html#windows).  
Por enquanto é necessário carregar o arquivo de configuração.

```sh
mkdir C:/ProgramData/glib-2.0/schemas/
cp org.caioxcezar.settings_gt.gschema.xml C:/ProgramData/glib-2.0/schemas/
glib-compile-schemas C:/ProgramData/glib-2.0/schemas/
```

Por fim é necessário instalar o [Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html)
