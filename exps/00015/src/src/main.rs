//! Recovery from Fatigue 解法 (harness で開発)
//! 可視グラフ最短路 → 給水機を k-medoids で街に置く → 給水訪問トークン入りの巡回列を SA
//! 1. 街・既設の給水所・円の周りの点で可視グラフを作り、障害物を避けた最短路を求める
//! 2. 最短路長を距離とした TSP を 2-opt / Or-opt で改善
//! 3. 給水間隔の二乗和が小さくなるよう、新しい給水機を経路上に等間隔に置く
use std::io::Read;
use std::time::Instant;

type P = (i64, i64);
type C = (i64, i64, i64);

fn seg_hits(p: P, q: P, c: C) -> bool {
    let (dx, dy) = (q.0 - p.0, q.1 - p.1);
    let (fx, fy) = (c.0 - p.0, c.1 - p.1);
    let r2 = c.2 * c.2;
    let len2 = dx * dx + dy * dy;
    let dot = fx * dx + fy * dy;
    if len2 == 0 || dot <= 0 {
        fx * fx + fy * fy < r2
    } else if dot >= len2 {
        let (gx, gy) = (c.0 - q.0, c.1 - q.1);
        gx * gx + gy * gy < r2
    } else {
        let cross = dx * fy - dy * fx;
        cross * cross < r2 * len2
    }
}
fn seg_ok(p: P, q: P, cs: &[C]) -> bool {
    cs.iter().all(|&c| !seg_hits(p, q, c))
}
fn inside_any(p: P, cs: &[C]) -> bool {
    cs.iter().any(|&c| (p.0 - c.0).pow(2) + (p.1 - c.1).pow(2) < c.2 * c.2)
}
fn dist(a: P, b: P) -> f64 {
    (((a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)) as f64).sqrt()
}

fn main() {
    let start = Instant::now();
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();
    let mut it = s.split_ascii_whitespace().map(|t| t.parse::<i64>().unwrap());
    let mut nx = || it.next().unwrap();
    let n = nx();
    let (ns, nw, nk, m) = (nx() as usize, nx() as usize, nx() as usize, nx() as usize);
    let (vv, ff) = (nx() as f64, nx() as f64);
    let towns: Vec<P> = (0..ns).map(|_| (nx(), nx())).collect();
    let fountains: Vec<P> = (0..nw).map(|_| (nx(), nx())).collect();
    let circles: Vec<C> = (0..nk).map(|_| (nx(), nx(), nx())).collect();

    // ---- 可視グラフのノード
    let mut nodes: Vec<P> = towns.clone();
    nodes.extend(&fountains);
    const RING: usize = 16;
    for &(cx, cy, r) in &circles {
        let rr = r as f64 / (std::f64::consts::PI / RING as f64).cos() + 1.5;
        for j in 0..RING {
            let a = 2.0 * std::f64::consts::PI * j as f64 / RING as f64;
            let p = (cx + (rr * a.cos()).round() as i64, cy + (rr * a.sin()).round() as i64);
            if 0 <= p.0 && p.0 <= n && 0 <= p.1 && p.1 <= n && !inside_any(p, &circles) {
                nodes.push(p);
            }
        }
    }
    let mut lattice_parent: Option<Vec<u32>> = None;
    let (nv, adj) = loop {
        let nv = nodes.len();
        let mut adj = vec![f64::INFINITY; nv * nv];
        for i in 0..nv {
            adj[i * nv + i] = 0.0;
            for j in i + 1..nv {
                if seg_ok(nodes[i], nodes[j], &circles) {
                    let d = dist(nodes[i], nodes[j]);
                    adj[i * nv + j] = d;
                    adj[j * nv + i] = d;
                }
            }
        }
        // 街1 から可視グラフで届かない街があれば、格子 BFS の経路を簡略化して中継点に加える
        let mut seen = vec![false; nv];
        seen[0] = true;
        let mut st = vec![0usize];
        while let Some(u) = st.pop() {
            for i in 0..nv {
                if !seen[i] && adj[u * nv + i].is_finite() {
                    seen[i] = true;
                    st.push(i);
                }
            }
        }
        let bad: Vec<usize> = (0..ns).filter(|&t| !seen[t]).collect();
        if bad.is_empty() || lattice_parent.is_some() {
            break (nv, adj);
        }
        let w = (n + 1) as usize;
        let id = |p: P| p.0 as usize * w + p.1 as usize;
        let mut par = vec![u32::MAX; w * w];
        par[id(towns[0])] = id(towns[0]) as u32;
        let mut q = std::collections::VecDeque::from([towns[0]]);
        while let Some(p) = q.pop_front() {
            for dx in -1..=1i64 {
                for dy in -1..=1i64 {
                    let r = (p.0 + dx, p.1 + dy);
                    if r.0 < 0 || r.0 > n || r.1 < 0 || r.1 > n || par[id(r)] != u32::MAX {
                        continue;
                    }
                    if inside_any(r, &circles) || !seg_ok(p, r, &circles) {
                        continue;
                    }
                    par[id(r)] = id(p) as u32;
                    q.push_back(r);
                }
            }
        }
        for t in bad {
            let mut chain = vec![towns[t]];
            while *chain.last().unwrap() != towns[0] {
                let x = par[id(*chain.last().unwrap())] as usize;
                chain.push(((x / w) as i64, (x % w) as i64));
            }
            // 見通しの利く限り先へ進む簡略化
            let mut i = 0;
            while i + 1 < chain.len() {
                let mut j = i + 1;
                while j + 1 < chain.len() && seg_ok(chain[i], chain[j + 1], &circles) {
                    j += 1;
                }
                if j + 1 < chain.len() {
                    nodes.push(chain[j]);
                }
                i = j;
            }
        }
        lattice_parent = Some(par);
    };

    // ---- 街ごとに Dijkstra（密グラフなので O(V^2)）
    let nk2 = ns + nw;
    let mut dmat = vec![0.0f64; nk2 * nk2];
    let mut prev_all = vec![0u32; nk2 * nv];
    for src in 0..nk2 {
        let mut d = vec![f64::INFINITY; nv];
        let mut done = vec![false; nv];
        let prev = &mut prev_all[src * nv..(src + 1) * nv];
        d[src] = 0.0;
        prev[src] = src as u32;
        loop {
            let mut u = usize::MAX;
            let mut best = f64::INFINITY;
            for i in 0..nv {
                if !done[i] && d[i] < best {
                    best = d[i];
                    u = i;
                }
            }
            if u == usize::MAX {
                break;
            }
            done[u] = true;
            let row = &adj[u * nv..(u + 1) * nv];
            for i in 0..nv {
                let nd = best + row[i];
                if nd < d[i] {
                    d[i] = nd;
                    prev[i] = u as u32;
                }
            }
        }
        for t in 0..nk2 {
            dmat[src * nk2 + t] = d[t];
        }
    }
    let dd = |a: usize, b: usize| dmat[a * nk2 + b];
    let tl: f64 = std::env::var("TL").ok().and_then(|x| x.parse().ok()).unwrap_or(2.85);
    let mut rng = Rng(0x1234_5678_9abc_def1);

    // ---- 給水機の位置: 街の上に置く（k-medoids, 既設給水所は固定の medoid）
    let mut waters: Vec<usize> = (ns..nk2).collect();
    let kp: i32 = std::env::var("KP").ok().and_then(|x| x.parse().ok()).unwrap_or(2);
    let near_cost = |ws: &[usize]| -> f64 {
        (0..ns).map(|t| ws.iter().map(|&w| dd(t, w)).fold(f64::INFINITY, f64::min).powi(kp)).sum::<f64>()
    };
    let mut stations: Vec<usize> = vec![];
    for _ in 0..m {
        let mut best = (f64::INFINITY, 0);
        for c in 0..ns {
            if stations.contains(&c) { continue; }
            let mut ws = waters.clone();
            ws.extend(&stations);
            ws.push(c);
            let v = near_cost(&ws);
            if v < best.0 { best = (v, c); }
        }
        stations.push(best.1);
    }
    // swap 改善
    let mut improved = true;
    while improved && start.elapsed().as_secs_f64() < 1.2 {
        improved = false;
        for k in 0..stations.len() {
            let mut ws = waters.clone();
            ws.extend(&stations);
            let cur = near_cost(&ws);
            let mut best = (cur, stations[k]);
            for c in 0..ns {
                if stations.contains(&c) { continue; }
                ws[waters.len() + k] = c;
                let v = near_cost(&ws);
                if v < best.0 - 1e-9 { best = (v, c); }
            }
            if best.1 != stations[k] { stations[k] = best.1; improved = true; }
        }
    }
    waters.extend(&stations);
    let is_water: Vec<std::cell::Cell<bool>> = (0..nk2).map(|_| std::cell::Cell::new(false)).collect();
    for &w in &waters { is_water[w].set(true); }
    let ws0 = nw; // waters[ws0..] が設置した給水機（街の番号）
    // 各街の近い街リスト（給水機移動の近傍）
    let near: Vec<Vec<usize>> = (0..ns).map(|a| {
        let mut v: Vec<usize> = (0..ns).filter(|&b| b != a).collect();
        v.sort_by(|&x, &y| dd(a, x).partial_cmp(&dd(a, y)).unwrap());
        v.truncate(24);
        v
    }).collect();
    const EX: usize = 1000;
    let nbp: u64 = std::env::var("NB").ok().and_then(|x| x.parse().ok()).unwrap_or(20);
    #[allow(non_snake_case)] let NB = nbp;
    #[allow(non_snake_case)] let OL: usize = std::env::var("OL").ok().and_then(|x| x.parse().ok()).unwrap_or(3);
    #[allow(non_snake_case)] let SM: u64 = std::env::var("SM").ok().and_then(|x| x.parse().ok()).unwrap_or(1); // 追加の給水訪問トークン（取り除ける）は +EX

    // ---- 初期解: 最近傍 TSP + 2-opt
    let mut tour = vec![0usize];
    let mut used = vec![false; ns];
    used[0] = true;
    for _ in 1..ns {
        let c = *tour.last().unwrap();
        let nxt = (0..ns).filter(|&j| !used[j]).min_by(|&a, &b| dd(c, a).partial_cmp(&dd(c, b)).unwrap()).unwrap();
        used[nxt] = true;
        tour.push(nxt);
    }
    let mut seq: Vec<usize> = tour.clone();
    seq.push(0);

    let eval = |seq: &[usize]| -> f64 {
        let mut tot = 0.0;
        let mut sq = 0.0;
        let mut g = 0.0;
        let mut prev = seq[0] % EX;
        for &x in &seq[1..] {
            let x = x % EX;
            let d = dd(prev, x);
            tot += d;
            g += d;
            if is_water[x].get() { sq += g * g; g = 0.0; }
            prev = x;
        }
        sq += g * g;
        vv * tot + ff * 0.5 * sq
    };
    // 貪欲に給水訪問を挿入（改善がある限り）
    {
        let mut cur = eval(&seq);
        loop {
            let mut best = (cur, 0, 0);
            let step = (seq.len() / 120).max(1);
            let mut i = 1;
            while i < seq.len() {
                for &w in &waters {
                    seq.insert(i, w + EX);
                    let v = eval(&seq);
                    seq.remove(i);
                    if v < best.0 { best = (v, i, w); }
                }
                i += step;
            }
            if best.0 >= cur || start.elapsed().as_secs_f64() > 1.5 { break; }
            seq.insert(best.1, best.2 + EX);
            cur = best.0;
        }
    }

    // ---- SA (差分評価: 変更範囲を囲む給水点 L..R の区間コストだけ再計算)
    let wat = |x: usize| is_water[x % EX].get();
    let region = |seq: &[usize], l: usize, r: usize| -> f64 {
        let (mut tot, mut sq, mut g) = (0.0, 0.0, 0.0);
        for k in l + 1..=r {
            let d = dd(seq[k - 1] % EX, seq[k] % EX);
            tot += d;
            g += d;
            if wat(seq[k]) || k == r { sq += g * g; g = 0.0; }
        }
        vv * tot + ff * 0.5 * sq
    };
    let lb = |seq: &[usize], mut i: usize| { while i > 0 && !wat(seq[i]) { i -= 1; } i };
    let rb = |seq: &[usize], mut i: usize| { let n = seq.len(); while i < n - 1 && !wat(seq[i]) { i += 1; } i };
    let p0: f64 = std::env::var("P0").ok().and_then(|x| x.parse().ok()).unwrap_or(2e-4);
    let p1: f64 = std::env::var("P1").ok().and_then(|x| x.parse().ok()).unwrap_or(1e-7);
    let mut cur = eval(&seq);
    let mut best_seq = seq.clone();
    let mut best = cur;
    let mut best_waters = waters.clone();
    let t_start = start.elapsed().as_secs_f64();
    let (temp0, temp1) = (cur * p0, cur * p1);
    let mut temp = temp0;
    let mut iter: u64 = 0;
    loop {
        if iter & 255 == 0 {
            let p = (start.elapsed().as_secs_f64() - t_start) / (tl - t_start);
            if p >= 1.0 { break; }
            temp = temp0 * (temp1 / temp0).powf(p);
        }
        iter += 1;
        let len = seq.len();
        let thr = -temp * rng.f64().max(1e-300).ln();
        let kind = rng.next() % 100;
        if kind < NB {
            // 近傍ベース: 街 x を近い街 y の隣へ移動 / x と y の間を反転
            let i = 1 + (rng.next() as usize) % (len - 2);
            let x = seq[i];
            if x >= EX || x == 0 { continue; }
            let y = near[x][(rng.next() as usize) % 8];
            let j0 = match seq[1..len - 1].iter().position(|&z| z == y) { Some(p) => p + 1, None => continue };
            if rng.next() % 2 == 0 {
                // 2-opt: x の次から y までを反転 (y が後ろ) / y の次から x までを反転
                let (a, b) = if j0 > i { (i + 1, j0) } else { (j0 + 1, i) };
                if a >= b { continue; }
                let (l, r) = (lb(&seq, a - 1), rb(&seq, b + 1));
                let before = region(&seq, l, r);
                seq[a..=b].reverse();
                let dlt = region(&seq, l, r) - before;
                if dlt <= thr { cur += dlt; } else { seq[a..=b].reverse(); }
            } else {
                // x を y の直後 (or 直前) へ
                let j = if rng.next() % 2 == 0 { j0 + 1 } else { j0 };
                if j == i || j == i + 1 { continue; }
                let (lo, hi) = if j > i { (i, j) } else { (j, i + 1) };
                let (l, r) = (lb(&seq, lo - 1), rb(&seq, hi));
                let before = region(&seq, l, r);
                if j > i { seq[i..j].rotate_left(1); } else { seq[j..i + 1].rotate_right(1); }
                let dlt = region(&seq, l, r) - before;
                if dlt <= thr { cur += dlt; } else {
                    if j > i { seq[i..j].rotate_right(1); } else { seq[j..i + 1].rotate_left(1); }
                }
            }
        } else if kind < 35 {
            // 2-opt
            let i = 1 + (rng.next() as usize) % (len - 2);
            let span = 1 + (rng.next() as usize) % 80;
            let j = (i + span).min(len - 2);
            if j <= i { continue; }
            let (l, r) = (lb(&seq, i - 1), rb(&seq, j + 1));
            let before = region(&seq, l, r);
            seq[i..=j].reverse();
            let dlt = region(&seq, l, r) - before;
            if dlt <= thr { cur += dlt; } else { seq[i..=j].reverse(); }
        } else if kind < 75 {
            // Or-opt: [i, i+l) を j の前へ
            let ln = 1 + (rng.next() as usize) % if rng.next() % 4 == 0 { OL } else { 3 };
            if len < ln + 3 { continue; }
            let i = 1 + (rng.next() as usize) % (len - 1 - ln);
            let off = (rng.next() as usize) % 120;
            let j = if rng.next() & 1 == 0 { i + ln + 1 + off } else { i.saturating_sub(off + 1).max(1) };
            if j > len - 1 || (j >= i && j <= i + ln) { continue; }
            let (lo, hi) = if j > i { (i, j) } else { (j, i + ln) };
            let (l, r) = (lb(&seq, lo - 1), rb(&seq, hi));
            let before = region(&seq, l, r);
            let rev = rng.next() % 2 == 0;
            if rev { seq[i..i + ln].reverse(); }
            if j > i { seq[i..j].rotate_left(ln); } else { seq[j..i + ln].rotate_right(ln); }
            let dlt = region(&seq, l, r) - before;
            if dlt <= thr { cur += dlt; } else {
                if j > i { seq[i..j].rotate_right(ln); } else { seq[j..i + ln].rotate_left(ln); }
                if rev { seq[i..i + ln].reverse(); }
            }
        } else if kind < 87 {
            // 給水訪問を挿入（前後に近い給水点を選ぶ）
            let i = 1 + (rng.next() as usize) % (len - 1);
            let (a, b) = (seq[i - 1] % EX, seq[i] % EX);
            if is_water[a].get() || is_water[b].get() { continue; }
            let w = if rng.next() % 3 == 0 { waters[(rng.next() as usize) % waters.len()] } else {
                *waters.iter().min_by(|&&x, &&y| (dd(a, x) + dd(x, b)).partial_cmp(&(dd(a, y) + dd(y, b))).unwrap()).unwrap()
            };
            let (l, r) = (lb(&seq, i - 1), rb(&seq, i));
            let before = region(&seq, l, r);
            seq.insert(i, w + EX);
            let dlt = region(&seq, l, r + 1) - before;
            if dlt < thr { cur += dlt; } else { seq.remove(i); }
        } else if kind < 95 - SM {
            // 給水訪問を削除
            let i = 1 + (rng.next() as usize) % (len - 2);
            if seq[i] < EX { continue; }
            let (l, r) = (lb(&seq, i - 1), rb(&seq, i + 1));
            let before = region(&seq, l, r);
            let x = seq.remove(i);
            let dlt = region(&seq, l, r - 1) - before;
            if dlt <= thr { cur += dlt; } else { seq.insert(i, x); }
        } else if kind < 95 + SM && ws0 < waters.len() {
            // 給水機を近くの街へ移動（全体再評価）
            let k = ws0 + (rng.next() as usize) % (waters.len() - ws0);
            let a = waters[k];
            let c = near[a][(rng.next() as usize) % near[a].len()];
            if is_water[c].get() { continue; }
            is_water[a].set(false);
            is_water[c].set(true);
            for x in seq.iter_mut() { if *x == a + EX { *x = c + EX; } }
            let v = eval(&seq);
            if v - cur <= thr { cur = v; waters[k] = c; } else {
                for x in seq.iter_mut() { if *x == c + EX { *x = a + EX; } }
                is_water[c].set(false);
                is_water[a].set(true);
            }
        } else {
            // 給水訪問の給水点を変更
            let i = 1 + (rng.next() as usize) % (len - 2);
            if seq[i] < EX { continue; }
            let (l, r) = (lb(&seq, i - 1), rb(&seq, i + 1));
            let before = region(&seq, l, r);
            let x = seq[i];
            seq[i] = waters[(rng.next() as usize) % waters.len()] + EX;
            let dlt = region(&seq, l, r) - before;
            if dlt <= thr { cur += dlt; } else { seq[i] = x; }
        }
        if cur < best - 1e-6 { best = cur; best_seq.clone_from(&seq); best_waters.clone_from(&waters); }
    }
    for w in &is_water { w.set(false); }
    for &w in &best_waters { is_water[w].set(true); }
    let chk = eval(&best_seq);
    if (chk - best).abs() > 1.0 { eprintln!("WARN: drift {} vs {}", chk, best); }
    eprintln!("sa_start={:.2}s iter={} best={:.0} waters={} visits={}", t_start, iter, best, waters.len(), best_seq.len());
    let seq = best_seq;

    // ---- 経路を点列に展開
    let mut path: Vec<P> = vec![towns[0]];
    for k in 0..seq.len() - 1 {
        let (a, b) = (seq[k] % EX, seq[k + 1] % EX);
        let prev = &prev_all[a * nv..(a + 1) * nv];
        let mut sq = vec![];
        let mut x = b;
        while x != a { sq.push(x); x = prev[x] as usize; }
        sq.reverse();
        for x in sq {
            path.push(nodes[x]);
        }
    }
    // 同一点の連続は除く（給水点で止まる必要はない: 点に到達すれば給水される）
    path.dedup();
    let st: Vec<P> = best_waters[ws0..].iter().map(|&t| towns[t]).collect();
    let mut out = String::new();
    out += &format!("{}\n", st.len());
    for p in &st { out += &format!("{} {}\n", p.0, p.1); }
    out += &format!("{}\n", path.len() - 1);
    for p in &path[1..] { out += &format!("{} {}\n", p.0, p.1); }
    print!("{}", out);
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 { self.0 ^= self.0 << 7; self.0 ^= self.0 >> 9; self.0 }
    fn f64(&mut self) -> f64 { (self.next() >> 11) as f64 / (1u64 << 53) as f64 }
}
