extern crate redis;
use redis::Commands;

#[no_mangle]
pub extern fn learn(con: &redis::Connection, input: &str) -> redis::RedisResult<()> {
    let mut prev = "";
    let sep = " ";
    let mut it = input.split(sep).peekable();

    loop {
        let word = match it.next() {
            Some(x) => x,
            None => break,
        };

        let key = make_key(prev, word);

        if it.peek().is_some() {
            let _ : () = try!(con.zincr(key, *it.peek().unwrap(), 1));
        } else {
            let _ : () = try!(con.zincr(key, "\n" , 1));
        }

        prev = word;
    }
    Ok(())
}

/// Takes two words and joins them with colons
fn make_key(str1: &str, str2: &str) -> String {
    str1.to_string() + ":" + str2
}

#[cfg(test)]
mod tests {
    extern crate redis;
    use redis::Commands;
    use super::learn;

    #[test]
    fn add_words_to_redis() {
        let client = redis::Client::open("redis://localhost").unwrap();
        let con = client.get_connection().unwrap();
        let teststring = "xyyzzzyyyasd xyyzzzyyyasd blah";
        let _ = learn(&con, teststring);
        let result : Vec<String> = con.zrange("xyyzzzyyyasd:xyyzzzyyyasd", 0, -1).unwrap();
        assert_eq!(result[0], "blah");
    }
}