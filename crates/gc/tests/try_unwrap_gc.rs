use gc::Gc;

#[test]
fn test_try_unwrap() {
    let mut gc = Gc::new(0);

    {
        let gc2 = Gc::new(Gc::clone(&gc));

        gc = Gc::try_unwrap(gc).unwrap_err();

        assert_eq!(Gc::try_unwrap(gc2), Ok(Gc::clone(&gc)));
    }

    assert_eq!(Gc::try_unwrap(gc), Ok(0));
}
