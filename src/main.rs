mod abs;

use abs::prelude::*;

fn main() {
    let tank = Tank::new("abs.toml").unwrap();
    tank.print_sections();
}