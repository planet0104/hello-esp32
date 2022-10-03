cargo build --release

espflash COM4 target/riscv32imc-esp-espidf/release/hello-esp32 --speed 921600
espmonitor COM4