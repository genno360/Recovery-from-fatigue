#!/usr/bin/env python3
"""実験ランナー: ソルバーをスナップショットしてビルドし、複数シードを並列実行・採点して exps/NNNNN/ に記録する。

AHC074 参加者 (chettub 氏の gist) の運用を参考にした設計:
  * 1 実験 = 1 ディレクトリ (exps/00000, 00001, ...)。ソース・出力・得点・メモを丸ごと保存し、後から再現・比較できる
  * まず少数シードでふるい (--seeds 0-19)、通ったものだけ全 100 シード
  * 同時実行による CPU 取り合いを避けるためロックで直列化 (slotd の簡易版)
  * 結果は exps/results.csv (1 行 1 実験) と docs/RESULTS.md (人が読む表) に追記
  * 各シードの出力は seed 番号のファイル名で保存し、全実験を通じた最良出力を best/ に保持

使い方:
  python3 harness/run.py --note "SA 温度を変更" [--seeds 0-99] [--jobs 4] [--src solver]
"""
import argparse, csv, datetime, fcntl, json, os, shutil, subprocess, sys, time
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TOOLS = os.path.join(ROOT, "tools")
EXPS = os.path.join(ROOT, "exps")
BEST = os.path.join(ROOT, "best")
TIME_LIMIT = 3.0


def parse_seeds(s):
    out = []
    for part in s.split(","):
        if "-" in part:
            a, b = part.split("-")
            out += list(range(int(a), int(b) + 1))
        elif part:
            out.append(int(part))
    return out


def next_exp_id():
    os.makedirs(EXPS, exist_ok=True)
    ids = [int(d) for d in os.listdir(EXPS) if d.isdigit()]
    return max(ids, default=-1) + 1


def build(src_dir):
    r = subprocess.run(["cargo", "build", "--release", "-q"], cwd=src_dir, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit("build failed:\n" + r.stderr)
    with open(os.path.join(src_dir, "Cargo.toml")) as f:
        name = [l.split("=")[1].strip().strip('"') for l in f if l.startswith("name")][0]
    return os.path.join(src_dir, "target", "release", name)


def score_bin():
    p = os.path.join(TOOLS, "target", "release", "score")
    if not os.path.exists(p):
        subprocess.run(["cargo", "build", "--release", "-q"], cwd=TOOLS, check=True)
    return p


def run_one(binary, seed, outdir, timeout):
    inp = os.path.join(TOOLS, "in", f"{seed:04d}.txt")
    outp = os.path.join(outdir, f"{seed:04d}.txt")
    errp = os.path.join(outdir, f"{seed:04d}.err")
    t0 = time.time()
    status = "OK"
    with open(inp) as fi, open(outp, "w") as fo, open(errp, "w") as fe:
        try:
            r = subprocess.run([binary], stdin=fi, stdout=fo, stderr=fe, timeout=timeout)
            if r.returncode != 0:
                status = "RE"
        except subprocess.TimeoutExpired:
            status = "TLE"
    el = time.time() - t0
    if status == "OK" and el > TIME_LIMIT:
        status = "TLE"
    sc = subprocess.run([score_bin(), inp, outp], capture_output=True, text=True)
    line = [l for l in (sc.stdout + sc.stderr).splitlines() if l.startswith("Score")]
    score = None
    if line and "WA" not in line[0]:
        score = int(line[0].split("=")[1])
    elif status == "OK":
        status = "WA"
    with open(inp) as f:
        hdr = f.readline().split()
    n, s, w, k, m, v, ff = map(int, hdr)
    return dict(seed=seed, score=score, status=status, time=round(el, 3), S=s, W=w, K=k, M=m, F=ff)


def update_best(rows, outdir, exp_id):
    os.makedirs(BEST, exist_ok=True)
    path = os.path.join(BEST, "best.json")
    best = json.load(open(path)) if os.path.exists(path) else {}
    improved = 0
    for r in rows:
        if r["score"] is None:
            continue
        key = str(r["seed"])
        if key not in best or r["score"] < best[key]["score"]:
            best[key] = {"score": r["score"], "exp": exp_id}
            shutil.copy(os.path.join(outdir, f"{r['seed']:04d}.txt"), os.path.join(BEST, f"{r['seed']:04d}.txt"))
            improved += 1
    json.dump(best, open(path, "w"), indent=0, sort_keys=True)
    return best, improved


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", default=os.path.join(ROOT, "solver"))
    ap.add_argument("--seeds", default="0-99")
    ap.add_argument("--jobs", type=int, default=os.cpu_count())
    ap.add_argument("--note", required=True, help="何を変えたか (必須: 後で読み返すため)")
    ap.add_argument("--parent", type=int, default=None, help="比較元の実験番号")
    ap.add_argument("--keep-outputs", action="store_true", help="出力ファイルを残す (既定は best 更新後に削除して容量節約)")
    a = ap.parse_args()
    seeds = parse_seeds(a.seeds)

    lock = open(os.path.join(ROOT, "harness", ".cpu.lock"), "w")
    print("waiting for CPU lock...", file=sys.stderr)
    fcntl.flock(lock, fcntl.LOCK_EX)

    exp_id = next_exp_id()
    edir = os.path.join(EXPS, f"{exp_id:05d}")
    os.makedirs(edir)
    shutil.copytree(a.src, os.path.join(edir, "src"), ignore=shutil.ignore_patterns("target"))
    binary = build(a.src)
    outdir = os.path.join(edir, "out")
    os.makedirs(outdir)
    timeout = TIME_LIMIT * 2 + 2
    t0 = time.time()
    with ThreadPoolExecutor(a.jobs) as ex:
        rows = list(ex.map(lambda s: run_one(binary, s, outdir, timeout), seeds))
    wall = time.time() - t0

    with open(os.path.join(edir, "scores.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    best, improved = update_best(rows, outdir, exp_id)
    ok = [r for r in rows if r["score"] is not None]
    bad = [r for r in rows if r["status"] != "OK"]
    total = sum(r["score"] for r in ok)
    # 相対スコア: 全実験の最良に対する比 (100 * best/score の平均)。小さいほど良い問題なので best/score
    rel = sum(100.0 * best[str(r["seed"])]["score"] / r["score"] for r in ok) / len(seeds)
    maxt = max(r["time"] for r in rows)
    meta = dict(exp=exp_id, note=a.note, parent=a.parent, seeds=a.seeds, n=len(seeds), total=total,
                mean=total / max(1, len(ok)), rel=round(rel, 3), fails=len(bad), max_time=maxt,
                wall=round(wall, 1), date=datetime.datetime.now().isoformat(timespec="seconds"))
    json.dump(meta, open(os.path.join(edir, "meta.json"), "w"), indent=1, ensure_ascii=False)
    if not a.keep_outputs and not bad:
        shutil.rmtree(outdir)

    res = os.path.join(EXPS, "results.csv")
    new = not os.path.exists(res)
    with open(res, "a", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(meta.keys()))
        if new:
            w.writeheader()
        w.writerow(meta)
    os.makedirs(os.path.join(ROOT, "docs"), exist_ok=True)
    md = os.path.join(ROOT, "docs", "RESULTS.md")
    if not os.path.exists(md):
        open(md, "w").write("# 実験結果\n\n| exp | seeds | mean | rel(vs best) | fails | max_time | note |\n|---|---|---|---|---|---|---|\n")
    with open(md, "a") as f:
        f.write(f"| {exp_id:05d} | {a.seeds} | {meta['mean']:.0f} | {rel:.2f} | {len(bad)} | {maxt:.2f} | {a.note} |\n")

    print(f"exp {exp_id:05d}: mean={meta['mean']:.1f} rel={rel:.3f} fails={len(bad)} max_time={maxt:.2f}s "
          f"wall={wall:.0f}s best_updated={improved}")
    for r in bad[:10]:
        print(f"  seed {r['seed']}: {r['status']}  (see exps/{exp_id:05d}/out/{r['seed']:04d}.err)")
    if a.parent is not None:
        subprocess.run([sys.executable, os.path.join(ROOT, "harness", "compare.py"), str(a.parent), str(exp_id)])


if __name__ == "__main__":
    main()
