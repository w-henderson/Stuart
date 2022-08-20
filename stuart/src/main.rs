mod fs;
mod parse;

fn main() {
    let content =
        std::fs::read_to_string("C:/Users/willi/Develop/React/StuartPortfolio/content/blog.html")
            .unwrap();
    let parsed = parse::parse(&content).unwrap();

    println!("{:#?}", parsed);
}
