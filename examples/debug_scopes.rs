use syntect::parsing::{SyntaxSet, ScopeStackOp};

fn main() {
    let ss = SyntaxSet::load_defaults_newlines();
    let syntax = ss.find_syntax_by_token("rust").unwrap();
    let mut state = syntect::parsing::ParseState::new(syntax);

    let code = r#"fn main() {
    let message = "Hello, terminal!";
    println!("{}", message);
}"#;

    for line in code.lines() {
        let ops = state.parse_line(line, &ss).unwrap();
        let mut pos = 0;
        let mut stack: Vec<syntect::parsing::Scope> = Vec::new();

        for (offset, op) in &ops {
            if *offset > pos {
                let text = &line[pos..*offset];
                let scope_str = stack.last().map(|s| s.build_string()).unwrap_or_default();
                if !text.trim().is_empty() {
                    println!("{:30} -> {}", format!("{:?}", text), scope_str);
                }
                pos = *offset;
            }
            match op {
                ScopeStackOp::Push(scope) => stack.push(*scope),
                ScopeStackOp::Pop(n) => { for _ in 0..*n { stack.pop(); } },
                ScopeStackOp::Restore => stack.clear(),
                _ => {}
            }
        }
        if pos < line.len() {
            let text = &line[pos..];
            let scope_str = stack.last().map(|s| s.build_string()).unwrap_or_default();
            if !text.trim().is_empty() {
                println!("{:30} -> {}", format!("{:?}", text), scope_str);
            }
        }
    }
}
