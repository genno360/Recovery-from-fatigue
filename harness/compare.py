#!/usr/bin/env python3
"""2 つの実験を同一シードで対応ありの比較をする。

近い結果を「改善した」と言う前に統計的な根拠を出す (AHC074 参加者が Claude Code に課したルール)。
  * 共通シードのみ比較 / 勝ち・負け・引き分け / 符号検定の p 値
  * log(B/A) の平均と t 値 (得点の桁がケースで大きく違うので対数で見る)
  * 入力パラメータ (F, S, M+W, K) で層別して、どの種類のケースで効いたかを示す

使い方: python3 harness/compare.py A B
"""
import csv, math, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def load(e):
    with open(os.path.join(ROOT, "exps", f"{int(e):05d}", "scores.csv")) as f:
        return {int(r["seed"]): r for r in csv.DictReader(f)}


def sign_p(win, lose):
    n = win + lose
    if n == 0:
        return 1.0
    k = min(win, lose)
    p = sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n
    return min(1.0, 2 * p)


def summarize(pairs):
    lr = [math.log(b / a) for a, b in pairs]
    n = len(lr)
    m = sum(lr) / n
    sd = math.sqrt(sum((x - m) ** 2 for x in lr) / max(1, n - 1))
    t = m / (sd / math.sqrt(n)) if sd > 0 else float("inf") * (1 if m > 0 else -1) if m else 0.0
    win = sum(b < a for a, b in pairs)
    lose = sum(b > a for a, b in pairs)
    return m, t, win, lose, n - win - lose


def main():
    A, B = sys.argv[1], sys.argv[2]
    a, b = load(A), load(B)
    seeds = sorted(s for s in a if s in b and a[s]["score"] and b[s]["score"])
    if not seeds:
        sys.exit("no common seeds")
    pairs = [(int(a[s]["score"]), int(b[s]["score"])) for s in seeds]
    m, t, win, lose, tie = summarize(pairs)
    print(f"exp {int(A):05d} -> {int(B):05d}  ({len(seeds)} common seeds)")
    print(f"  mean A={sum(p[0] for p in pairs)/len(pairs):.0f}  B={sum(p[1] for p in pairs)/len(pairs):.0f}")
    print(f"  mean log(B/A)={m*100:+.3f}%  t={t:+.2f}  win/lose/tie={win}/{lose}/{tie}  sign-test p={sign_p(win, lose):.4f}")
    verdict = "B better" if (t < -2 and sign_p(win, lose) < 0.05) else "A better" if (t > 2 and sign_p(win, lose) < 0.05) else "inconclusive (more seeds or larger effect needed)"
    print(f"  verdict: {verdict}")
    # 層別
    feats = {
        "F": lambda r: "F<=3" if int(r["F"]) <= 3 else "F<=7" if int(r["F"]) <= 7 else "F>=8",
        "S": lambda r: "S<200" if int(r["S"]) < 200 else "S<300" if int(r["S"]) < 300 else "S>=300",
        "W+M": lambda r: f"W+M={int(r['W'])+int(r['M'])}" if int(r["W"]) + int(r["M"]) < 6 else "W+M>=6",
        "K": lambda r: "K<10" if int(r["K"]) < 10 else "K<25" if int(r["K"]) < 25 else "K>=25",
    }
    for name, fn in feats.items():
        groups = {}
        for s in seeds:
            groups.setdefault(fn(a[s]), []).append((int(a[s]["score"]), int(b[s]["score"])))
        cells = []
        for g in sorted(groups):
            mm, tt, w, l, _ = summarize(groups[g])
            cells.append(f"{g}: {mm*100:+.2f}% ({w}/{l})")
        print(f"  by {name:4s} " + "  ".join(cells))
    worst = sorted(seeds, key=lambda s: int(b[s]["score"]) / int(a[s]["score"]))[-3:]
    print("  worst seeds for B:", ", ".join(f"{s}({100*(int(b[s]['score'])/int(a[s]['score'])-1):+.1f}%)" for s in reversed(worst)))


if __name__ == "__main__":
    main()
