#/usr/bin/env bash

set -ex

pyftsubset ./assets/fonts/Noto_Serif/NotoSerif-VariableFont_wdth,wght.ttf --text-file=./dictionary.txt
pyftsubset ./assets/fonts/Noto_Sans_Sinhala/NotoSansSinhala-VariableFont_wdth,wght.ttf --text-file=./dictionary.txt
pyftsubset ./assets/fonts/0xProto/0xProtoNerdFont-Regular.ttf --text-file=./dictionary.txt
