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

#[no_mangle]
pub extern fn generate(con: &redis::Connection, seed: &str) -> String {
    let mut result = seed.to_string(); // Start our result with the seed passed to us
    let mut prev = "".to_string();
    let mut cur = seed.to_string();

    loop {
        let key = make_key(&prev, &cur);
        let members : Vec<String> = con.zrevrange(key, 0, -1).unwrap();
        if members.len() > 0 && members[0] != "\n" {
            result.push_str(" ");
            result.push_str(&members[0]);
            prev = cur;
            cur = members[0].clone();
        } else {
            break;
        }
    }

    result
}

/// Takes two words and joins them with colons
fn make_key(str1: &str, str2: &str) -> String {
    str1.to_string() + ":" + str2
}

#[cfg(test)]
mod tests {
    extern crate redis;
    use redis::Commands;
    use super::*;

    #[test]
    fn add_words_to_redis() {
        let client = redis::Client::open("redis://localhost").unwrap();
        let con = client.get_connection().unwrap();
        let teststring = "test_string_please_ignore test_string_please_ignore success";
        let _ = learn(&con, teststring);
        let result : Vec<String> = con.zrevrange("test_string_please_ignore:test_string_please_ignore", 0, -1).unwrap();
        assert_eq!(result[0], "success");
    }

    #[test]
    fn generate_something() {
        let client = redis::Client::open("redis://localhost").unwrap();
        let con = client.get_connection().unwrap();
        let result = generate(&con, "test_string_please_ignore");
        assert!(result.len() > 0);
    }
}