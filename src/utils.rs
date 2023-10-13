use std::{str::FromStr, any::type_name};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};

/// Asks the user for an input of given type `T`. Returns the parsed 
/// input as soon as the input is valid and of type `T`. 
///
/// # Example
///
/// ```
/// let i: u32 = blocking_typed_read_line().await?;
/// let s: String = blocking_typed_read_line().await?;
/// ```
pub async fn typed_read_line_blocking<T: FromStr>() -> Result<T, Box<dyn std::error::Error>> {
    println!("Expecting input of type {}:", type_name::<T>());
    let stdin = tokio::io::stdin();
    let mut reader = FramedRead::new(stdin, LinesCodec::new());

    loop {
        let line = reader.next().await.unwrap().expect("Something went wrong reading the buffer");
        //let line = line.trim_end_matches(&['\r', '\n'][..]);
        let parsed_input = line.parse::<T>();

        match parsed_input {
            Ok(input) => return Ok(input),
            Err(_) => {
                println!("Expected type {1}. Couldn't parse '{0}' to {1}! Try different input.", line, type_name::<T>());
                continue;
            },
        }
    }
}
