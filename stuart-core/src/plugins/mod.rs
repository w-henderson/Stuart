use crate::functions::FunctionParser;

pub trait Manager {
    fn plugins(&self) -> &[Plugin];
}

pub struct Plugin {
    pub name: String,
    pub version: String,
    pub functions: Vec<Box<dyn FunctionParser>>,
}

impl<T> Manager for T
where
    T: AsRef<[Plugin]>,
{
    fn plugins(&self) -> &[Plugin] {
        self.as_ref()
    }
}

#[macro_export]
macro_rules! declare_plugin {
    (
        name: $name:expr,
        version: $version:expr,
        functions: [
            $($function:expr),*
        ],
    ) => {
        #[no_mangle]
        pub extern "C" fn _stuart_plugin_init() -> *mut ::stuart_core::plugins::Plugin {
            let plugin = ::stuart_core::plugins::Plugin {
                name: $name.into(),
                version: $version.into(),
                functions: vec![
                    $(
                        Box::new($function)
                    ),*
                ],
            };

            Box::into_raw(Box::new(plugin))
        }
    };
}
