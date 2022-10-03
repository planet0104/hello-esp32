cargo build

espflash COM4 target/riscv32imc-esp-espidf/debug/hello-esp32 --speed 921600
espmonitor COM4
