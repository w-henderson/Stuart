mod function;

use stuart_core::declare_plugin;

declare_plugin! {
    name: "demo",
    version: "0.1.0",
    functions: [function::DemoParser],
    parsers: [],
}
