extern crate redis;
extern crate rand;
use self::rand::{thread_rng, sample};
use redis::Commands;


/// Adds the input text to our redis brain
#[no_mangle]
pub extern fn learn(con: &redis::Connection, input: &str) -> redis::RedisResult<()> {
    let mut prev = "";
    let sep = " ";
    let mut it = input.split(sep).peekable();

    loop {
        let word = match it.next() {
            Some(x) => x,
            None => break, // If we don't have anymore words, exit the loop
        };

        let key = make_key(prev, word);

        // Add our key and member to redis
        if it.peek().is_some() {
            let _ : () = try!(con.zincr(key, *it.peek().unwrap(), 1));
        } else {
            // If there is no next value, add a \n as the member instead
            let _ : () = try!(con.zincr(key, "\n" , 1));
        }

        prev = word;
    }
    Ok(())
}

/// Generates text using the redis brain from the seed given
#[no_mangle]
pub extern fn generate(con: &redis::Connection, seed: &str, bias: &str) -> String {
    let mut prev = "".to_string();
    let mut cur = seed.to_string();
    let mut result = seed.to_string(); // Start our result with the seed passed to us

    loop {
        let mut key = make_key(&prev, &cur);
        let all_keys : Vec<String> = redis::cmd("KEYS").arg("*").query(con).unwrap();

        // If our key doesn't exist in the db, clear the result and
        // grab a new random key to start generation
        if !all_keys.contains(&key) {
            result.clear();
            key = choice(all_keys);
            let mut s : Vec<&str> = key.split(":").collect();
            cur = s.pop().unwrap().to_string();
            result.push_str(&cur);
        }

        // Query redis for our key and choose the next word from the members
        let members : Vec<(String, i32)> = con.zrevrange_withscores(key, 0, -1).unwrap();
        let options = get_options(members, bias);
        let next = choice(options);

        if next != "\n" { // Stop generation if the next character is EOL
            result.push_str(" ");
            result.push_str(&next);
            prev = cur;
            cur = next.clone();
        } else {
            break;
        }
    }

    result
}

/// Returns the members with the top score
fn get_options(members: Vec<(String, i32)>, bias: &str) -> Vec<String> {
    let mut options : Vec<String> = Vec::new();
    let mut prev_score = 0;

    // Add members in our bias to the options
    let bias : Vec<&str> = bias.split_whitespace().collect();
    for (member, _) in members.clone() {
        if bias.contains(&&*member) {
            options.push(member)
        }
    }

    // Return early if we found a word in our bias
    if options.len() > 0 {
        return options
    }

    for (member, score) in members {
        // Continue to add members as long as they have the same joint top score
        if score < prev_score {
            break;
        } else {
            options.push(member);
            prev_score = score;
        }
    }
    options
}

/// Takes two words and joins them with colons for redis serialisation
fn make_key(str1: &str, str2: &str) -> String {
    str1.to_string() + ":" + str2
}

// Takes a vector and chooses one variable from it
fn choice<T: Clone>(v: Vec<T>) -> T {
    let mut rng = thread_rng();
    sample(&mut rng, v.iter(), 1).pop().unwrap().clone()
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
        let teststring = "£ test_string_please_ignore success";
        let _ = learn(&con, teststring);
        let result : Vec<String> = con.zrevrange("@:test_string_please_ignore", 0, -1).unwrap();
        assert_eq!(result[0], "success");
    }

    #[test]
    fn generate_something() {
        let client = redis::Client::open("redis://localhost").unwrap();
        let con = client.get_connection().unwrap();
        let result = generate(&con, "£", "");
        assert!(result.len() > 0);
    }
}