macro_rules! define_testcases {
    ($($name:ident),*) => {
        $(
            #[test]
            fn $name() {
                let testcase = Testcase::new(stringify!($name));
                testcase.run();
            }
        )*
    };
}
