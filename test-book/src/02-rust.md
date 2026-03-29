# Rust 编程语言

Rust 是一门现代的系统编程语言，由 Mozilla 开发。

## 主要特点

### 内存安全

Rust 最大的特点是**内存安全**。它通过所有权系统和借用检查器来保证内存安全，而不需要垃圾回收机制。

### 零成本抽象

Rust 提供了高级的抽象能力，但这些抽象在运行时几乎没有开销。这使得 Rust 非常适合系统编程。

### 并发安全

Rust 的类型系统保证了并发安全性。你可以在不担心数据竞争的情况下编写并发代码。

## 学习资源

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust 中文社区](https://rustcc.cn/)

## 示例代码

```rust
fn main() {
    let message = "你好，Rust!";
    println!("{}", message);
}
```

Rust 编译器会检查你的代码，确保没有内存错误和数据竞争。
