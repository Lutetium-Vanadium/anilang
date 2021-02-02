mod common;
use common::*;

#[test]
fn empty_interface() {
    assert_eq!(
        execute(
            "interface I {}
            I()"
        )
        .unwrap(),
        v::o(vec![]),
    );
}

#[test]
fn interface_with_no_constructor() {
    let o = execute(
        "interface I {
            v = 123
            fn a(self) {
                self.v
            }

            fn make(v) {
                { v, }
            }
        }

        assert(
            I::v == 123,
            I::a
        )

        let i = I()
                         /*    BROKEN     */
        assert(i.v == 123/*, i.a() == 123 */)

        i = I::make(456)
        assert(i.v == 456)
        i",
    )
    .unwrap();
    assert!(o.clone().get_at(v::s("make")).is_err());
    // Even though, a is declared, with self in args, this static method doesn't include it in its
    // return value
    assert!(o.get_at(v::s("a")).is_err());
}

#[test]
fn interface_with_constructor() {
    assert_eq!(
        execute(
            "interface I {
                v = 123

                fn a(self) {
                    self.v
                }

                I(v) {
                    self.v = v
                }
            }

            assert(
                I::v == 123,
                I::a
            )

            let i = I(456)
                               /*    BROKEN     */
            assert(i.v == 456, /* i.a() == 456 ,*/ I::a(i) == 456)",
        )
        .unwrap(),
        v::n()
    );

    assert_eq!(
        execute(
            "interface I {
                v = 123

                // Optional fn
                fn I(v) {
                    self.v = v
                }
            }

            assert(I::v == 123)

            let i = I(456)
            i.v"
        )
        .unwrap(),
        v::i(456)
    );
}

#[test]
fn interface_err_with_multiple_constructors() {
    assert!(execute(
        "interface I {
            I() {}
            I(v) {}
        }"
    )
    .is_err());
}
