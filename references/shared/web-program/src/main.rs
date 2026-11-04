use std::{env, path::Path};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let op = args.get(1).ok_or("operation required")?;
    let value = args.get(2).ok_or("manifest/id required")?;
    let program = match op.as_str() {
        "apply" => injectra_web::apply(&injectra_web::Bundle::load(Path::new(value))?)?,
        "remove" => injectra_web::remove(value)?,
        _ => return Err("unsupported operation".into()),
    };
    println!("{program}");
    Ok(())
}
