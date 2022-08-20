use stuart::fs::Node;
use stuart::Stuart;

static IN: &str = "C:/Users/willi/Develop/React/StuartPortfolio";
static OUT: &str = "C:/Users/willi/Develop/React/StuartPortfolio/dist";

fn main() {
    let fs = Node::new(IN).unwrap();
    let mut stuart = Stuart::new(fs);
    stuart.build();
    stuart.save(OUT).unwrap();
}
