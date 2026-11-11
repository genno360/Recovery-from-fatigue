//! 使い方: gen seeds.txt [出力ディレクトリ(既定: in)]
//! seeds.txt の各行のシードからテストケースを生成し、in/0000.txt, in/0001.txt, ... に書き出す。
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: gen <seeds.txt> [out_dir]");
        std::process::exit(1);
    }
    let seeds = std::fs::read_to_string(&args[1]).expect("cannot read seeds file");
    let dir = args.get(2).cloned().unwrap_or_else(|| "in".to_string());
    std::fs::create_dir_all(&dir).unwrap();
    for (i, line) in seeds.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let seed: u64 = line.trim().parse().expect("seed must be an integer");
        let input = tools::gen(seed);
        let path = format!("{}/{:04}.txt", dir, i);
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{}", input).unwrap();
    }
}
