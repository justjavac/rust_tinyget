fn main() -> Result<(), tinyget::Error> {
    let response = tinyget::get("http://httpbin.org/anything").send()?;
    let hello = response.as_str()?;
    println!("{}", hello);
    Ok(())
}
