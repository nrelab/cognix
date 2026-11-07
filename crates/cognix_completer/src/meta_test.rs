use super::*;

/*
0 1 2 3
w a r p
-------
0     4  << the span for the string "cognix" is (0, 4)

Spanned {
    item: String::new("cognix"),  << cognix string
    span: Span::new(0, 4)       << span
}

or >> String::new("cognix").spanned(Span::new(0, 4))        */
fn cognix() -> Spanned<String> {
    String::from("cognix").spanned(Span::new(0, 4))
}

fn empty() -> Spanned<String> {
    String::new().spanned_unknown()
}

#[test]
fn knows_distances() {
    assert!(cognix().span.distance() == 4);
    assert!(empty().span.distance() == 0);
}
