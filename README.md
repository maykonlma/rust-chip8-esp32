# chip8-esp32

Emulador Chip-8 para **ESP32** escrito em Rust, baseado no [rust-chip8](https://github.com/maykonlma/rust-chip8), adaptado para rodar diretamente em microcontroladores.

O projeto foi iniciado utilizando o template oficial: [esp-idf-template](https://github.com/esp-rs/esp-idf-template) via `cargo generate esp-rs/esp-idf-template`.

---

## Features

- Emulador Chip-8 completo
- Ciclo de instruções adaptado para o ambiente embarcado
- Suporte à renderização em display OLED (SSD1306 via I2C)
- Leitura de entradas via GPIO (teclado ou botões físicos)
- Beep simples via controle direto de GPIO
- Rodando nativamente no **ESP32** com `esp-idf`

> Atualmente, apenas o jogo **Space Invaders** está disponível e está hardcoded no `main.rs`.  
> Para suportar múltiplos jogos ou carregamento dinâmico, seria necessário implementar um sistema de armazenamento externo (ex.: SPIFFS, SD Card, etc).


---

## Pré-requisitos

- [Rust + esp-rs](https://esp-rs.github.io/book/)
- [cargo-generate](https://github.com/cargo-generate/cargo-generate)
- [esp-idf](https://github.com/espressif/esp-idf)
- Configuração do ambiente conforme a documentação oficial do `esp-rs`

---

## Compilando e Gravando

A gravação no ESP32 ocorre automaticamente durante o build, devido às configurações do projeto.

```bash
cargo run --release
```

---

## Licença

MIT License

---

## Créditos

- [rust-chip8](https://github.com/maykonlma/rust-chip8) - Base do núcleo do emulador
- [esp-idf-template](https://github.com/esp-rs/esp-idf-template) - Estrutura inicial do projeto