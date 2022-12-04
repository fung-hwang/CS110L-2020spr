# CS110L Spring 2020: Safety in Systems Programming

## 课程简介
CS110L将带领我们学习**Rust**，这是一门注重**安全、性能、工程**的语言。

1. Why Rust？
    1. 我的浅显理解是：**Rust** 被设计出来旨在解决目前系统级编程的困难，其特征“**安全、性能、工程**”也是围绕系统级编程的需求而来。在事实上挑战了C语言的地位，**Rust**试图在拥有媲美C的性能的同时改善C在安全、工程上的不足。
    2. [更详细的回答1（by Rust语言圣经）](https://course.rs/into-rust.html)
    3. [更详细的回答2（by Rust程序设计语言）](https://doc.rust-lang.org/stable/book/foreword.html)

2. For Who？
    + For everyone. 别质疑，先学习！

## 参考资料
1. [2020版本 课程官网](https://reberhardt.com/cs110l/spring-2020/)
2. [2022版本 课程官网](https://web.stanford.edu/class/cs110l/)
3. [Rust 程序设计语言](https://rustwiki.org/zh-CN/book/title-page.html)
4. [Rust 语言圣经](https://course.rs/about-book.html)
5. [Rust 异步编程](https://huangjj27.github.io/async-book/index.html)
6. [PKUFlyingPig代码实现](https://github.com/PKUFlyingPig/CS110L)

## 作业&项目
1. [Exercises 1: Hello world](https://github.com/fung-hwang/CS110L-2020spr/tree/main/week1)
2. [Exercises 2: Ownership and structs](https://github.com/fung-hwang/CS110L-2020spr/tree/main/week2)
3. [Exercises 3: Error handling, I/O, and traits](https://github.com/fung-hwang/CS110L-2020spr/tree/main/week3)
4. [Exercises 4: Farm meets multithreading](https://github.com/fung-hwang/CS110L-2020spr/tree/main/week5)
5. [Exercises 5: Sharing Data by Communicating](https://github.com/fung-hwang/CS110L-2020spr/tree/main/week6)
6. [Project 1: The DEET Debugger](https://github.com/fung-hwang/CS110L-2020spr/tree/main/proj-1)
7. [Project 2: Balancebeam](https://github.com/fung-hwang/CS110L-2020spr/tree/main/proj-2)

## 一些说明
1. 当我2022年11月计划学习Rust并从[CS自学指南](https://csdiy.wiki/)发现本课程时，Thea Rossman的[2022版本CS110L](https://web.stanford.edu/class/cs110l/assignments/week-1-exercises/)似乎把Github代码库设为仅选修该课程的Stanford同学可见，并且没有公开课程视频。所以我们仍关注Ryan Eberhardt and Armin Namavari的[2020版本CS110L](https://reberhardt.com/cs110l/spring-2020/)。二者稍有区别但对Rust学习不会产生实质影响：
    + 课程内容稍有改动，涉及课程讲义、作业和项目，但仅是细节处的优化。二者可对比参照。
    + Rust在近几年快速演进，2020版本的项目所使用的库可能过于老旧而不再支持。可以预见的是Rust及Rust库会持续更新，如果CS110L继续闭源，是时可参考 [Project-1 适配说明](https://github.com/fung-hwang/CS110L-2020spr/blob/main/proj-1/README.md) 和 [Project-2 适配说明](https://github.com/fung-hwang/CS110L-2020spr/blob/main/proj-2/README.md) 而自行适配。
2. Rust库适配过程本质上是API查阅的过程，我通常的做法是：
    1. `cargo check/cargo run` 查看并分析错误。
    2. 在 [lib.rs](lib.rs) 或 [crates.io](crates.io) 上搜索库并查看版本更新、简介、使用说明、库特征等。
    3. 在API文档中查找所需要的API，并可以与旧版API进行比较。

## 写在最后
1. Rust以学习曲线陡峭著称，不可否认，但也不必担心。我想，除了语言开发团队在语言的各种特性上的取舍与平衡，我们这些使用者也在付出和得到间获得了平衡——更多的付出与更好的代码。
2. 受[Rust首次被合入Linux](https://en.wikipedia.org/wiki/Rust_for_Linux)的鼓舞，也许Rust生态真的会蓬勃发展吧:)

