fn main() -> Result<(), tinyget::Error> {
    let response = tinyget::get("https://httpbin.org/anything")
        .with_timeout(10)
        .send()?;
    let hello = response.as_str()?;
    println!("{}", hello);
    Ok(())
}
