//! Tests for the bound on how deeply the grammar will recurse.
//!
//! Without a bound these inputs overflow the stack, and a stack overflow aborts the
//! process instead of unwinding -- so a regression here does not fail politely, it
//! takes the test binary down with it. The depths below are chosen with that in
//! mind: one just past the limit, which fails cleanly if the bound is merely
//! loosened, and one far past it, which is where the abort used to happen.

use crate::ast::Program;
use crate::error::ParseError;
use crate::parser::peg::MAX_GRAMMAR_NESTING;
use crate::parser::{Parser, ParserOptions};

fn parse(input: &str) -> Result<Program, ParseError> {
    let options = ParserOptions::default();
    Parser::new(std::io::Cursor::new(input), &options).parse_program()
}

/// Returns labeled inputs that nest each recursive construct `depth` levels deep.
fn nested_inputs(depth: usize) -> Vec<(&'static str, String)> {
    vec![
        (
            "brace groups",
            format!("{} true; {}", "{ ".repeat(depth), "} ".repeat(depth)),
        ),
        (
            "if clauses",
            format!(
                "{} true {}",
                "if true; then ".repeat(depth),
                "; fi".repeat(depth)
            ),
        ),
        (
            "while clauses",
            format!(
                "{} true {}",
                "while true; do ".repeat(depth),
                "; done".repeat(depth)
            ),
        ),
        ("coprocesses", format!("{}true", "coproc ".repeat(depth))),
        (
            "subshells",
            format!("{} true {}", "( ".repeat(depth), ") ".repeat(depth)),
        ),
        (
            "process substitutions",
            format!("echo {} x {}", "<( echo ".repeat(depth), ")".repeat(depth)),
        ),
        (
            "function bodies",
            format!(
                "{} f() {{ true; }} {}",
                "{ ".repeat(depth),
                "} ".repeat(depth)
            ),
        ),
        (
            "extended test parentheses",
            format!("[[ {} x {} ]]", "( ".repeat(depth), ") ".repeat(depth)),
        ),
        (
            "extended test negations",
            format!("[[ {} x ]]", "! ".repeat(depth)),
        ),
        (
            "arithmetic parentheses",
            format!("(( {} 1 {} ))", "( ".repeat(depth), ") ".repeat(depth)),
        ),
        (
            "regex groups",
            format!("[[ x =~ {}a{} ]]", "(".repeat(depth), ")".repeat(depth)),
        ),
    ]
}

#[test]
fn deeply_nested_input_is_declined_instead_of_overflowing() {
    let limit = MAX_GRAMMAR_NESTING as usize;

    for depth in [limit + 8, 200] {
        for (label, input) in nested_inputs(depth) {
            let result = parse(&input);
            assert!(
                matches!(
                    result,
                    Err(ParseError::NestingTooDeep { limit: reported, .. })
                        if reported == MAX_GRAMMAR_NESTING
                ),
                "expected {label} nested {depth} deep to be declined, got: {result:?}"
            );
        }
    }
}

#[test]
fn ordinarily_nested_input_still_parses() {
    for (label, input) in nested_inputs(8) {
        assert!(
            parse(&input).is_ok(),
            "expected {label} nested 8 deep to parse: {input}"
        );
    }
}

#[test]
fn wide_input_is_not_mistaken_for_deep_input() {
    // Breadth costs no stack, so none of these should come anywhere near the bound
    // no matter how many terms they have.
    let count = 4 * MAX_GRAMMAR_NESTING as usize;
    let inputs = [
        (
            "sequential commands",
            format!("{} true", "true; ".repeat(count)),
        ),
        (
            "pipeline stages",
            format!("{} true", "true | ".repeat(count)),
        ),
        (
            "if clauses in sequence",
            "if true; then true; fi; ".repeat(count),
        ),
        (
            "case items",
            format!("case x in {} esac", "a) true;; ".repeat(count)),
        ),
        (
            "negated test terms",
            format!("[[ {} ! z ]]", "! a && ".repeat(count)),
        ),
        ("test terms", format!("[[ {} z ]]", "a && ".repeat(count))),
        (
            "arithmetic terms",
            format!("(( {} 1 ))", "1 + (2) + ".repeat(count)),
        ),
        (
            "regex alternatives",
            format!("[[ x =~ {}a ]]", "(b)|".repeat(count)),
        ),
        (
            "process substitutions",
            format!("echo {}", "<(true) ".repeat(count)),
        ),
    ];

    for (label, input) in inputs {
        assert!(
            parse(&input).is_ok(),
            "expected {count} {label} to parse: {input}"
        );
    }
}

#[test]
fn nesting_error_points_at_the_offending_token() {
    let input = format!("{} true; {}", "{ ".repeat(64), "} ".repeat(64));
    let result = parse(&input);

    let Err(ParseError::NestingTooDeep { position, .. }) = &result else {
        unreachable!("expected a nesting error, got: {result:?}");
    };

    // The limit is reached partway into the run of opening braces, not at the start
    // of the input and not at its end.
    assert_eq!(position.line, 1);
    assert!(
        position.column > 1 && position.column < input.len(),
        "unexpected column {}",
        position.column
    );
}
