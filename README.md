# Recovery from Fatigue

ヒューリスティック型（AHC 形式）の最適化問題です。

ランナーが街1から出発してすべての街を巡り、街1に戻ります。走るほど疲労がたまり、移動にかかる時間が増えていきます。
給水機の設置位置と走る経路を決めて、総移動時間をできるだけ短くしてください。盤面には侵入不可エリアがあります。

## 問題文

[index.html](index.html) をブラウザで開いてください。問題文の最後にビジュアライザが付いています。

## ファイル構成

| パス | 内容 |
|---|---|
| `index.html` | 問題文とビジュアライザ |
| `tools/` | 入力生成 `gen` と採点 `score`（Rust、外部クレートなし） |
| `tools/in/` | テストケース 100 個（seed 0〜99） |
| `solver/` | サンプル解法（Rust） |

## 使い方

Rust（cargo）が必要です。

```bash
# サンプル解法をビルドして 1 ケース解く
cd solver
cargo build --release
./target/release/solver < ../tools/in/0000.txt > ../out.txt

# 採点
cd ../tools
cargo run --release --bin score in/0000.txt ../out.txt

# 入力を生成し直す（seeds.txt の各行がシード）
cargo run --release --bin gen seeds.txt in
```

`score` は得点を `Score = 123456` の形で出力します。不正な出力のときは `Score = WA` を出力し、理由を標準エラー出力に書きます。
