# Project 1: The DEET Debugger

## 依赖适配 (2023.03.08)
1. **nix** 
    + 0.17.0 -> 0.26.2
    + `nix::sys::ptrace::write` 变为了`unsafe`函数，需用`unsafe{}`包裹
    + 已适配，可不做适配
2. **libc** 
    + 0.2.68 -> 0.2.139
    + 无需改动
    + 已适配，可不做适配
3. **rustyline** 
    + 6.1.2 -> 11.0.0
    + `Editor::<()>::new()` 返回值由`Self`变为`Result<Self>`
    + `Editor` 类型的泛型参数从一个变为两个，可以指定为`Editor<(), FileHistory>`，其中 `FileHistory` 为 `use rustyline::history::FileHistory;`
    + `Editor::add_history_entry` 返回值由 `bool` 变为 `Result<bool>`
    + 已适配，可不做适配
4. **gimli** 
    + git = "https://github.com/gimli-rs/gimli", rev = "ad23cdb2" -> 0.27.2
    + `gimli::Dwarf::load` 参数从两个变为一个，把原来的第二个参数去掉即可
    + `gimli::read::line::LineRow::line()` 返回值由 `Option<u64>` 变为 `Option<NonZeroU64>`，需要把 `NonZeroU64` 给提出来变成普通的 `u64`
    + `gimli_wrapper.rs::get_attr_value()` 缺少对 `gimli::AttributeValue::DebugLineStrRef(offset)` 的解析，需要补上
    + 已适配，可不做适配
5. **object**
    + 0.17.0 -> 0.30.0
    + `section_data_by_name` 方法被删除，需要用 `section_by_name` 和 `uncompressed_data` 组合来实现原有的功能（参考 gimli 仓库提供的 example）
    + 已适配，可不做适配
6. **memmap**
    + memmap = "0.7" -> memmap2 = "0.5.10"
    + `memmap` -> `memmap2`，换个名字就行
    + 已适配，可不做适配
7. **addr2line**
    + 0.11.0 -> 0.19.0
    + `addr2line::gimli::Error` 和 `gimli::Error` 现在已经变成了一个类型，需要删除 `impl From<addr2line::gimli::Error> for Error `
    + 已适配，可不做适配
8. **其他说明**
    + ~~未适配的 gimli, object, memmap, addr2line 只在 dwarf_data 与 gimli_wrapper 中出现。由于我对 dwarf 不甚了解，且这四个库关联性较大、适配比较困难，所以未进行适配。~~（已经完成适配）
