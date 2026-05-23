use marcli::{render, RenderOptions};

fn main() {
    let md = r#"# Marcli

**CommonMark Markdown to ANSI-escaped terminal output.**

Supports *italic*, **bold**, ~~strikethrough~~, and `inline code`.

## Bullet List

- First item
- Second with **bold**
- Third with `code`

## Ordered List

1. One
2. Two
3. Three

## Code Block

```rust
fn main() {
    let message = "Hello, terminal!";
    println!("{}", message);
}
```

## Block Quote

> The best way to predict the future is to invent it.
> -- Alan Kay

---

## Table

| Language | Paradigm    | Year |
|----------|-------------|------|
| Rust     | Systems     | 2010 |
| Elixir   | Functional  | 2011 |
| Zig      | Systems     | 2016 |

## Links

Check out [Marcli on GitHub](https://github.com/Oeditus/marcli-rust) for more.
"#;

    let output = render(md, &RenderOptions::default());
    println!("{}", output);
}
