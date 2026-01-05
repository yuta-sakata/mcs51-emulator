# mcs51-emulator

一个用 Rust 编写的 8051 (MCS-51) 单片机指令级仿真器。

## 功能简介
- 支持 8051 指令集的仿真执行（已实现 161/256 条指令，覆盖率 62.9%）
- 支持 Intel HEX 格式程序加载
- 支持 debug 调试输出，显示每条指令的时钟周期、内存地址、助记符、参数等
- 支持 RAM、ROM、SFR、寄存器等基本硬件结构
- 指令统计表查看功能，快速了解已实现的指令
- 模块化指令注册系统，易于扩展和维护

## 使用方法

### 编译

```sh
cargo build --release
```

### 运行

```sh
# 查看帮助信息
./target/release/mcs51-emulator --help

# 运行程序
./target/release/mcs51-emulator <hex文件>

# 调试模式运行
./target/release/mcs51-emulator <hex文件> --debug

# 查看指令实现统计
./target/release/mcs51-emulator --inst-dump

# 或使用 cargo
cargo run -- <hex文件> [--debug]
```

### 命令行选项

- `<hex文件>`：Intel HEX 格式的程序文件
- `--debug` 或 `debug`：开启详细指令执行输出
- `--inst-dump` 或 `-i`：显示已实现的指令统计表
- `--help` 或 `-h`：显示帮助信息

### 指令统计表示例

运行 `--inst-dump` 可以查看 16x16 的指令实现情况表格：

```
[mcs51-emulator][inst-dump] ===================================================================================================
[mcs51-emulator][inst-dump]       0     1     2     3     4     5     6     7     8     9     A     B     C     D     E     F
[mcs51-emulator][inst-dump] ===================================================================================================
[mcs51-emulator][inst-dump]  00   NOP  AJMP  LJMP   INC   INC   INC  ----  ----   INC   INC   INC   INC   INC   INC   INC   INC
[mcs51-emulator][inst-dump]  10  ----  ---- LCALL   RRC   DEC  ----  ----  ----   DEC   DEC   DEC   DEC   DEC   DEC   DEC   DEC
[mcs51-emulator][inst-dump]  20  ----  AJMP   RET    RL   ADD   ADD  ----  ----   ADD   ADD   ADD   ADD   ADD   ADD   ADD   ADD
[mcs51-emulator][inst-dump]  30   JNB  ----  RETI   RLC  ADDC  ----  ----  ----  ----  ----  ----  ----  ----  ----  ----  ----
[mcs51-emulator][inst-dump]  40  ----  AJMP  ----  ----   ORL  ----  ----  ----   ORL   ORL   ORL   ORL   ORL   ORL   ORL   ORL
[mcs51-emulator][inst-dump]  50  ----  ----  ----  ----  ----  ----  ----  ----   ANL   ANL   ANL   ANL   ANL   ANL   ANL   ANL
[mcs51-emulator][inst-dump]  60    JZ  AJMP  ----  ----  ----  ----  ----  ----   XRL   XRL   XRL   XRL   XRL   XRL   XRL   XRL
[mcs51-emulator][inst-dump]  70   JNZ  ----  ----  ----   MOV   MOV  ----  ----   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV
[mcs51-emulator][inst-dump]  80  SJMP  AJMP   ANL  ----   DIV   MOV  ----  ----   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV
[mcs51-emulator][inst-dump]  90   MOV  ----  ----  ----  ----  SUBB  ----  ----  SUBB  SUBB  SUBB  SUBB  SUBB  SUBB  SUBB  SUBB
[mcs51-emulator][inst-dump]  A0  ----  AJMP  ----  ----   MUL  ----  ----  ----   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV
[mcs51-emulator][inst-dump]  B0  ----  ----   CPL  ----  ----  CJNE  ----  ----  ----  ----  ----  ----  CJNE  ----  CJNE  ----
[mcs51-emulator][inst-dump]  C0  PUSH  AJMP   CLR   CLR  ----   XCH  ----  ----  ----  ----  ----  ----  ----  ----  ----  ----
[mcs51-emulator][inst-dump]  D0   POP  ----  SETB  ----  ----  DJNZ  ----  ----  DJNZ  DJNZ  DJNZ  DJNZ  DJNZ  DJNZ  DJNZ  DJNZ
[mcs51-emulator][inst-dump]  E0  MOVX  AJMP  ----  ----   CLR   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV
[mcs51-emulator][inst-dump]  F0  MOVX  ----  ----  ----   CPL   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV   MOV
[mcs51-emulator][inst-dump] ===================================================================================================
[mcs51-emulator][inst-dump] 已实现指令: 161/256 (62.9%)
```

### Debug 输出格式

```
[时钟周期][内存地址] 指令及参数                (变量参数)
[17450856][0x00c2] mov R7, 0x82                (value=40, will write to RAM[7])
[17450868][0x00c4] mov A, 0x08                 (value=8)
[17450880][0x00c6] jnz 0x00cb
[17450892][0x00cb] mov 0xf0, 0x08              (value=8)
[17450904][0x00ce] mov A, R7                   (value=40)
```

