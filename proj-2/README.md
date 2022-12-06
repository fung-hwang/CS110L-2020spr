# Project 2: Balancebeam

## 依赖适配 (2022.12.04)
1. **clap**
    + clap = "3.0.0-beta.1" -> clap = { version = "4.0.26", features = ["derive"] }
    + beta版本已不被支持，其命令行解析所涉及的struct属性需做较大变动，可见 [clap文档](https://docs.rs/clap/latest/clap/) 和 [适配详情](https://github.com/fung-hwang/CS110L-2020spr/commit/2c761a630b6d6a293d2d93bc5d942554711019cf)
    + 已适配，**需适配**
2. **httparse**
    + 1.3 -> 1.8
    + 已适配，可不做适配
3. **env_logger**
    + 0.7 -> 0.9
    + 已适配，可不做适配
4. **tokio**
    + tokio = { version = "0.2", features = ["full"] } -> tokio =  { version = "1", features = ["full"] }
    + sleep函数由`delay_for`改为`sleep`
    + 由于异步编程的核心是tokio，强烈建议使用最新版本的tokio库。为避免被时代抛弃，相信你也不想在整个项目中使用旧库、查阅旧API手册吧。
    + 已适配，可不做适配，建议适配
5. **rand**
    + 0.7 -> 0.8
    + gen_range函数由 `fn gen_range<T: SampleUniform, B1, B2>(&mut self, low: B1, high: B2) -> T` 变为 `fn gen_range<T, R>(&mut self, range: R) -> T`
    + 已适配，可不做适配
6. **nix**
    + 0.17 -> 0.25
    + 已适配，可不做适配
7. **hyper**
    + 0.13 -> hyper = { version = "0.14", features = ["full"] }
    + 已适配，可不做适配
8. **reqwest** 
    + 0.10 -> 0.11
    + 已适配，可不做适配
9. **其他说明**
    + 小版本的迭代不做适配，因为一般不改变API
    + 项目所有库已适配至2022.11版本，可放心食用
    + 适配详情可见 [git diff](https://github.com/fung-hwang/CS110L-2020spr/commit/2c761a630b6d6a293d2d93bc5d942554711019cf)


