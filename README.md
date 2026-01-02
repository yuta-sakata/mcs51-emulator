# mcs51-emulator

一个用 Rust 编写的 8051 (MCS-51) 单片机指令级仿真器。

## 功能简介
- 支持 8051 指令集的仿真执行
- 支持 Intel HEX 格式程序加载
- 支持 debug 调试输出，显示每条指令的时钟周期、内存地址、助记符、参数等
- 支持 RAM、ROM、SFR、寄存器等基本硬件结构

## 使用方法

### 编译

```sh
cargo build --release
```

### 运行

```sh
cargo run -- <hex文件> [--debug]
# 或
./target/release/mcs51-emulator <hex文件> [--debug]
```

- `<hex文件>`：Intel HEX 格式的程序文件
- `--debug`：可选，开启详细指令执行输出

### Debug 输出格式

```
[时钟周期][内存地址] 指令及参数                (变量参数)
[17450856][0x00c2] mov R7, 0x82                (value=40, will write to RAM[7])
[17450868][0x00c4] mov A, 0x08                 (value=8)
[17450880][0x00c6] jnz 0x00cb
[17450892][0x00cb] mov 0xf0, 0x08              (value=8)
[17450904][0x00ce] mov A, R7                   (value=40)
```

