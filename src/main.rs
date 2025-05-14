use kctf_pow::ChallengeParams;

fn gen_usage(name: &str) -> String {
    format!(
        "Could not parse arguments
Usage:
    To solve a challenge: {name} solve <challenge>
    To check a challenge: {name} check <challenge>
    To randomly generate a challenge: {name} gen <difficulty>
    To chain generation with checking: {name} ask <difficulty>"
    )
}

fn actual_main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let name = args.first().map(|x| x as _).unwrap_or("kctf-pow");
    if args.len() < 3 {
        return Err(gen_usage(name));
    }
    match &args[1] as _ {
        "solve" => {
            let chall = ChallengeParams::decode_challenge(&args[2])?;
            println!("{}", chall.solve());
        }
        "check" => {
            let chall = ChallengeParams::decode_challenge(&args[2])?;
            let mut inp = String::new();
            std::io::stdin()
                .read_line(&mut inp)
                .map_err(|_| "Could not read from stdin")?;
            let res = chall.check(inp.trim())?;
            if res {
                println!("correct");
            } else {
                println!("incorrect");
                return Err("Challenge verification failed".into());
            }
        }
        "gen" => {
            let difficulty: u32 = args[2]
                .parse()
                .map_err(|_| "Difficulty is not a valid 32-bit unsigned integer")?;
            println!("{}", ChallengeParams::generate_challenge(difficulty));
        }
        "ask" => {
            let difficulty: u32 = args[2]
                .parse()
                .map_err(|_| "Difficulty is not a valid 32-bit unsigned integer")?;
            let chall = ChallengeParams::generate_challenge(difficulty);
            println!("{chall}");
            let mut inp = String::new();
            std::io::stdin()
                .read_line(&mut inp)
                .map_err(|_| "Could not read from stdin")?;
            let res = chall.check(inp.trim())?;
            if res {
                println!("correct");
            } else {
                println!("incorrect");
                return Err("Challenge verification failed".into());
            }
        }
        _ => {
            return Err(gen_usage(name));
        }
    }
    Ok(())
}

fn main() {
    if let Err(s) = actual_main() {
        eprintln!("Error: {s}");
        std::process::exit(1);
    }
}
