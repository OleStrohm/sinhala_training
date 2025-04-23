#! /usr/bin/env -S nix shell nixpkgs#python312Packages.fonttools --command bash

set -ex

pyftsubset ./assets/fonts/Noto_Serif/NotoSerif-VariableFont_wdth,wght.ttf --text-file=./used_characters.txt
pyftsubset ./assets/fonts/Noto_Sans_Sinhala/NotoSansSinhala-VariableFont_wdth,wght.ttf --text-file=./used_characters.txt
pyftsubset ./assets/fonts/0xProto/0xProtoNerdFont-Regular.ttf --text-file=./used_characters.txt
