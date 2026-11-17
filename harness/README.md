# AHC 実験ハーネス

AHC074 の参加者が残した記録をもとに作った、AI エージェント（Claude Code など）と人が一緒に使う実験ハーネスです。主な出典は chettub 氏の「AHC074でのAIを用いた取り組み」の gist とそのプロンプト記録です。外部ライブラリは使わず、Python 3 の標準ライブラリだけで動きます。

## 参加者の知見と、それをどう取り込んだか

| AHC074 での知見 | ハーネスでの対応 |
|---|---|
| 1 実験を 1 ディレクトリ (`exps/00000/`…) に保存し、実験ごとの平均を CSV に残す | `run.py` が `exps/NNNNN/` に src スナップショット・`scores.csv`・`meta.json` を保存し、`exps/results.csv` と `docs/RESULTS.md` に追記する |
| 少数シードで悪い案を捨て、残った案だけ 100 シードで評価する。100 シードで見えない差は今は追わない | `--seeds 0-19` でふるい、`--seeds 0-99` で確定 |
| 差が小さいときは同じシードで対で比べ、統計的な根拠を示す（Claude Code に課したルール） | `compare.py`: log 比の平均・t 値・勝敗・符号検定・判定。判定が出ない差は「改善」と呼ばない |
| エージェント同士が CPU を取り合わないよう、ジョブスケジューラ (slotd) を使う | `run.py` が flock で実験を直列化。シードは `--jobs` で並列実行 |
| 各シードの出力をシード番号の名前で保存し、最良解は別に保持する | `best/NNNN.txt` と `best/best.json`。`rel` = 最良に対する相対スコア |
| ケースの種類ごとに効く改善が違う | `compare.py` が F・S・W+M・K で層別して表示する |
| マシンやセッションが変わる前に HANDOVER を書き、作業ログを残す | `docs/HANDOVER.md` と `log/`。運用ルールは `CLAUDE.md` |
| 局所的な改良に囚われない。構造を変えたときに一番伸びた。部品の寄与を確かめてシンプルに保つ | `CLAUDE.md` の方針に明記 |
| パラメータを試すときにコードを書き換えない | ソルバーが環境変数 (`TL`, `P0`, `P1`, `NB`, `KP`) を読む。例: `NB=40 python3 harness/run.py ...` |

## コマンド

```bash
python3 harness/run.py --note "変更内容" --seeds 0-19 --parent 7   # ふるい (約 15 秒, 4 コア)
python3 harness/run.py --note "変更内容" --parent 7                 # 全 100 ケース (約 70 秒)
python3 harness/compare.py 7 11                                     # 対応あり比較 + 層別
```

`run.py` は既定で出力ファイルを `best/` へ反映してから消します（容量節約のため）。出力を残すときは `--keep-outputs` を付けてください。WA・TLE・RE が出た実験は、出力と標準エラーを残します。

別の AHC で使うときは、`run.py` の `TOOLS`・`TIME_LIMIT`・得点の向き（小さいほど良い）と、`compare.py` の層別キーを問題に合わせて変えてください。
