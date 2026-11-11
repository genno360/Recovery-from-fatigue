//! 使い方: score <input> <output>
//! 得点を標準出力に、内訳を標準エラーに出す。
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: score <input> <output>");
        std::process::exit(1);
    }
    let input = tools::parse_input(&std::fs::read_to_string(&args[1]).unwrap());
    let out_s = std::fs::read_to_string(&args[2]).unwrap();
    let res = tools::parse_output(&out_s).and_then(|o| tools::compute_score(&input, &o));
    match res {
        Ok((score, sim)) => {
            eprintln!(
                "time = {:.3}, length = {:.3}, unvisited = {}",
                sim.time, sim.length, sim.unvisited
            );
            println!("Score = {}", score);
        }
        Err(e) => {
            // 得点は小さいほど良いので、WA を 0 と表示すると最良に見えてしまう
            eprintln!("WA: {}", e);
            println!("Score = WA");
            std::process::exit(1);
        }
    }
}
