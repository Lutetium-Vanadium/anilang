use anilang::function::AnilangFn;
use anilang::{Diagnostics, Evaluator, Lexer, Lowerer, Parser, SourceText};
use criterion::{criterion_group, criterion_main, Criterion};

macro_rules! rand {
    ($min:expr, $max:expr; $t:ty) => {{
        let min = $min;
        (rand!() * ($max - min) as f64).trunc() as $t + min
    }};

    ($min:expr, $max:expr; $t:ty; nonzero) => {{
        let rand = rand!($min, $max; $t);
        if rand == 0 as $t {
            1 as $t
        } else {
            rand
        }
    }};

    ($max:expr; $t:ty) => {
        rand!(0, $max; $t)
    };

    () => {
        ::rand::random::<f64>()
    };
}

fn perform_test(c: &mut Criterion, id: &str, src: &str, allow_err: bool) {
    let src = SourceText::new(src);
    let mut diagnostics = Diagnostics::new(&src);
    if allow_err {
        diagnostics = diagnostics.no_print();
    }

    let mut group = c.benchmark_group(id);

    group.bench_function("full-no_optimize", |b| {
        b.iter(|| {
            let tokens = Lexer::lex(&src, &diagnostics);
            let ast = Parser::parse(tokens, &src, &diagnostics);
            let bytecode = Lowerer::lower(ast.clone(), &diagnostics, false);
            Evaluator::evaluate(&bytecode[..], &diagnostics)
        });
    });

    group.bench_function("full-optimize", |b| {
        b.iter(|| {
            let tokens = Lexer::lex(&src, &diagnostics);
            let ast = Parser::parse(tokens, &src, &diagnostics);
            let bytecode = Lowerer::lower(ast.clone(), &diagnostics, true);
            Evaluator::evaluate(&bytecode[..], &diagnostics)
        });
    });

    group.bench_function("lexer", |b| b.iter(|| Lexer::lex(&src, &diagnostics)));
    let tokens = Lexer::lex(&src, &diagnostics);

    group.bench_function("parser", |b| {
        b.iter(|| Parser::parse(tokens.clone(), &src, &diagnostics))
    });
    let ast = Parser::parse(tokens, &src, &diagnostics);

    group.bench_function("lower-no_optimize", |b| {
        b.iter(|| Lowerer::lower(ast.clone(), &diagnostics, false))
    });
    let bytecode = Lowerer::lower(ast.clone(), &diagnostics, false);
    // Bad hack to make sure iterations don't have the same scopes
    let func = AnilangFn::new(vec![], bytecode);

    group.bench_function("evaluate-no_optimize", |b| {
        b.iter(|| Evaluator::evaluate(&func.duplicate_body()[..], &diagnostics))
    });
    let v1 = Evaluator::evaluate(&func.body[..], &diagnostics);

    group.bench_function("lower-optimize", |b| {
        b.iter(|| Lowerer::lower(ast.clone(), &diagnostics, true))
    });
    let bytecode = Lowerer::lower(ast, &diagnostics, true);
    // Bad hack to make sure iterations don't have the same scopes
    let func = AnilangFn::new(vec![], bytecode);

    group.bench_function("evaluate-optimize", |b| {
        b.iter(|| Evaluator::evaluate(&func.duplicate_body()[..], &diagnostics))
    });
    let v2 = Evaluator::evaluate(&func.body[..], &diagnostics);

    group.finish();

    assert_eq!(v1, v2);
    if !allow_err {
        assert!(!diagnostics.any());
    }
}

fn basic(c: &mut Criterion) {
    perform_test(c, "basic", "1 + 2", false);
}

fn huge_arithmetic_int(c: &mut Criterion) {
    let operators = [" + ", " - ", " * ", " / ", " % ", " || ", " && "];

    let mut buf = String::with_capacity(1000 * 8);
    for _ in 0..1000 {
        buf.push_str(&format!(
            "{}{}{}",
            rand!(-100, 100; i8; nonzero),
            operators[rand!(operators.len(); usize)],
            rand!(-100, 100; i8; nonzero)
        ));
    }
    // Even though we avoid 0s in the program, mod can result in a 0
    perform_test(c, "huge_arithmetic_int", &buf, true)
}

fn huge_arithmetic_float(c: &mut Criterion) {
    let operators = [" + ", " - ", " * ", " / ", " % ", " || ", " && "];

    let mut buf = String::with_capacity(1000 * 10);
    for _ in 0..1000 {
        buf.push_str(&format!(
            "{}{}{}",
            (rand!(-10.0, 10.0; f64; nonzero) * 10.0).trunc() / 10.0,
            operators[rand!(operators.len(); usize)],
            (rand!(-10.0, 10.0; f64; nonzero) * 10.0).trunc() / 10.0,
        ));
    }
    // Even though we avoid 0s in the program, mod can result in a 0
    perform_test(c, "huge_arithmetic_float", &buf, true)
}

fn factorial(c: &mut Criterion) {
    perform_test(
        c,
        "factorial",
        "fn factorial(a) {
            if a <= 2 {
                a
            } else {
                a * factorial(a-1)
            }
        }

        factorial(10) == 3628800
        && factorial(20) == 2432902008176640000",
        false,
    )
}

fn many_blocks(c: &mut Criterion) {
    perform_test(
        c,
        "many_blocks",
        "let a = 12313
        {
            let b = { { a, } }
            let should_be_true = {
                let c = { [b] }
                a = 123132323
                c[0].a == 12313
            }

            let a = {
                b.a * ({ num: 123123 })['num']
            }

            should_be_true
        }",
        false,
    )
}

criterion_group!(
    benches,
    basic,
    huge_arithmetic_int,
    huge_arithmetic_float,
    factorial,
    many_blocks
);
criterion_main!(benches);
