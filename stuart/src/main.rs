use stuart::fs::Node;
use stuart::Stuart;

static IN: &str = "C:/Users/willi/OneDrive/StuartPortfolio";
static OUT: &str = "C:/Users/willi/OneDrive/StuartPortfolio/dist";

fn main() {
    let start = std::time::Instant::now();
    let fs = Node::new(IN).unwrap();
    println!("{:#?}", fs);
    let mut stuart = Stuart::new(fs);
    stuart.build();
    stuart.save(OUT).unwrap();
    let duration = start.elapsed().as_micros();
    println!("took {}us", duration);
}
