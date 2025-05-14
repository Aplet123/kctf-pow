# kctf-pow

A library and CLI to solve, check, and generate proof-of-work challenges using [kCTF](https://google.github.io/kctf/)'s scheme.

# Installation

For use as a library, add the [`kctf-pow`](https://crates.io/crates/kctf-pow) crate into your dependencies.

The CLI can be installed with `cargo install kctf-pow`, or by cloning the repository, building with `cargo build --release`, and manually copying the executable.

# CLI Usage

To solve a challenge and print the solution to stdout:
```
kctf-pow solve <challenge>
```
For example:
```bash
kctf-pow solve s.AAAAMg==.NDtqORW1uZlIgzszbdMGZA==
# Outputs s.NUH3arymnKB+ysUGdv+67ypDamn4wOKCPORB2ivWE1Yhinam2v4S6q4nAoC5LP97LScdVoq+NuFVF++Win5mNRYZS6bJAs8fk0h8XgvfcC/7JfmFISqeCIo/CIUgIucVAM+eGDjqitRULGXqIOyviJoJjW8DMouMRuJM/3eg/z18kutQHkX0N3sqPeF7Nzkk8S3Bs6aiHUORM30syUKYug==
```
To check a solution for a challenge:
```
kctf-pow check <challenge>
```
The solution is read from stdin. If the solution is correct, the program will exit with status code 0 and `correct` will be outputted. If the solution is incorrect, the program will exit with status code 1 and `incorrect` will be outputted. If the solution is malformed, the program will exit with status code 1 and an error message will be printed to stderr.

For example:
```bash
kctf-pow check s.AAAAMg==.NDtqORW1uZlIgzszbdMGZA==
# Input s.NUH3arymnKB+ysUGdv+67ypDamn4wOKCPORB2ivWE1Yhinam2v4S6q4nAoC5LP97LScdVoq+NuFVF++Win5mNRYZS6bJAs8fk0h8XgvfcC/7JfmFISqeCIo/CIUgIucVAM+eGDjqitRULGXqIOyviJoJjW8DMouMRuJM/3eg/z18kutQHkX0N3sqPeF7Nzkk8S3Bs6aiHUORM30syUKYug==
# Outputs correct and exits with status code 0
```

To randomly generate a challenge:
```
kctf-pow gen <difficulty>
```
For example:
```bash
kctf-pow gen 50
# Outputs s.AAAAMg==.NDtqORW1uZlIgzszbdMGZA==
```

To chain challenge generation and checking:
```
kctf-pow ask <difficulty>
```
For example:
```bash
kctf-pow ask 50
# Outputs s.AAAAMg==.NDtqORW1uZlIgzszbdMGZA==
# Input s.NUH3arymnKB+ysUGdv+67ypDamn4wOKCPORB2ivWE1Yhinam2v4S6q4nAoC5LP97LScdVoq+NuFVF++Win5mNRYZS6bJAs8fk0h8XgvfcC/7JfmFISqeCIo/CIUgIucVAM+eGDjqitRULGXqIOyviJoJjW8DMouMRuJM/3eg/z18kutQHkX0N3sqPeF7Nzkk8S3Bs6aiHUORM30syUKYug==
# Outputs correct and exits with status code 0
```

# Library Usage

```rust
use kctf_pow::ChallengeParams;

fn main() {
    let pow = KctfPow::new();
    // decoding then solving a challenge
    let chall = ChallengeParams::decode_challenge("s.AAAAMg==.H+fPiuL32DPbfN97cpd0nA==").unwrap();
    println!("{}", chall.solve());
    // decoding then checking a challenge
    let chall = ChallengeParams::decode_challenge("s.AAAAMg==.NDtqORW1uZlIgzszbdMGZA==").unwrap();
    let sol = "s.NUH3arymnKB+ysUGdv+67ypDamn4wOKCPORB2ivWE1Yhinam2v4S6q4nAoC5LP97LScdVoq+NuFVF++Win5mNRYZS6bJAs8fk0h8XgvfcC/7JfmFISqeCIo/CIUgIucVAM+eGDjqitRULGXqIOyviJoJjW8DMouMRuJM/3eg/z18kutQHkX0N3sqPeF7Nzkk8S3Bs6aiHUORM30syUKYug==";
    assert_eq!(chall.check(sol), Ok(true));
    assert_eq!(chall.check("s.asdf"), Ok(false));
    // generating a random challenge of difficulty 50
    let chall = ChallengeParams::generate_challenge(50);
    println!("{}", chall);
}
```

# Library Documentation

The documentation for the library is available on [docs.rs](https://docs.rs/kctf-pow).
