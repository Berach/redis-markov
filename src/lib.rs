extern crate redis;
extern crate rand;
use self::rand::{thread_rng, sample};
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
    let mut prev = "".to_string();
    let mut cur = seed.to_string();
    let mut result = seed.to_string(); // Start our result with the seed passed to us
    let mut rng = thread_rng();

    loop {
        let mut key = make_key(&prev, &cur);
        let all_keys : Vec<String> = redis::cmd("KEYS").arg("*").query(con).unwrap();
        println!("Current Key: {:?}", key);
        if !all_keys.contains(&key) {
            key = sample(&mut rng, all_keys.iter(), 1).pop().unwrap().clone();
            let mut split : Vec<&str> = key.split(":").collect();
            cur = split.pop().unwrap().to_string();
            result.push_str(&cur);
            println!("New Key: {:?}", key);
        }
        let members : Vec<(String, i32)> = con.zrevrange_withscores(key, 0, -1).unwrap();

        let options = get_options(members);

        let choice = sample(&mut rng, options.iter(), 1).pop().unwrap();

        if choice != "\n" {
            result.push_str(" ");
            result.push_str(choice);
            prev = cur;
            cur = choice.clone();
        } else {
            break;
        }
    }

    result
}

fn get_options(members: Vec<(String, i32)>) -> Vec<String> {
    let mut options : Vec<String> = Vec::new();
    let mut prev_score = 0;

    for (member, score) in members {
        if score < prev_score {
            break;
        } else {
            options.push(member);
            prev_score = score;
        }
    }
    options
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