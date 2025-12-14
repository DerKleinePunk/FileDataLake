# File Data Lake

Bringe Ordnung in deine Datei ablagen und mach das wieder finden leichter.
Ein Ort wo alles ist und mehr Attribute zum wieder finden. Erstelle mit Plugins eigene Attribute für deine Dateiablage.

Wir Testen so weit möglich immer Windows (11) und Debian/Ubuntu

## Build

cargo build --manifest-path ./src/Cargo.toml

fdl_reader(.exe) -> über wacher des Verzeichnisses
fdl_webserver(.exe) Web UI mit Upload Möglichkeit

config Windows
C:\Users\{user}\AppData\Roaming\fdl\config

config Linux
???


## Config

## Docker

## Logging

[env_logger](https://github.com/rust-cli/env_logger)

## Python Addons

def example2(*args, **kwargs) -> dict:
    <" Your Code">
    newdict = {"testnew":"value1"}
    return newdict1


## Links

https://stackoverflow.com/questions/75848399/how-to-efficiently-use-actix-multipart-to-upload-a-single-file-to-disk
https://fasterthanli.me/series/building-a-rust-service-with-nix/part-8
