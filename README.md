# 安装 pre-commit
`pre-commit` 是一个代码检查工具，可以在提交代码前进行代码检查。
```
pipx install pre-commit
```
安装成功后运行 `pre-commit install` 即可。



# 安装 CI 增强工具
- `Cargo deny` 是一个 Cargo 插件，可以用于检查依赖的安全性。
- `typos` 是一个拼写检查工具。
- `git cliff` 是一个生成 changelog 的工具。
- `cargo nextest` 是一个 Rust 增强测试工具。

```
cargo binstall cargo-deny typos-cli git-cliff cargo-nextest
```
