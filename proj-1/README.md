# Project 1: The DEET Debugger

## 依赖适配 (2022.12.04)
1. **nix** 
    + 0.17.0 -> 0.26
    + `nix::sys::ptrace::write` 变为了`unsafe`函数，需用`unsafe{}`包裹
    + 已适配，可不做适配
2. **libc** 
    + 0.2.68 -> 0.2.138
    + 无需改动
    + 已适配，可不做适配
3. **rustyline** 
    + 6.1.2 -> 10
    + `Editor::<()>::new()` 返回值由`Self`变为`Result<Self>`
    + 已适配，可不做适配
4. **gimli** 
    + git = "https://github.com/gimli-rs/gimli", rev = "ad23cdb2" -> 0.27.0
    + 未适配，可不做适配
5. **object**
    + 0.17.0 -> 0.30.0
    + 未适配，可不做适配
6. **memmap**
    + memmap = "0.7" -> memmap2 = "0.5.8"
    + 未适配，可不做适配
7. **addr2line**
    + 0.11.0 -> 0.19.0
    + 未适配，可不做适配
8. **其他说明**
    + 未适配的 gimli, object, memmap, addr2line 只在 dwarf_data 与 gimli_wrapper 中出现。由于我对 dwarf 不甚了解，且这四个库关联性较大、适配比较困难，所以未进行适配。
