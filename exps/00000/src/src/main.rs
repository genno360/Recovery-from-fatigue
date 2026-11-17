//! Recovery from Fatigue サンプル解法
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
    let (_v, _f) = (nx(), nx());
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
    let mut dmat = vec![0.0f64; ns * ns];
    let mut prev_all = vec![0u32; ns * nv];
    for src in 0..ns {
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
        for t in 0..ns {
            dmat[src * ns + t] = d[t];
        }
    }
    let dd = |a: usize, b: usize| dmat[a * ns + b];

    // ---- TSP: 最近傍法 + 2-opt + Or-opt
    let mut tour = vec![0usize];
    let mut used = vec![false; ns];
    used[0] = true;
    for _ in 1..ns {
        let c = *tour.last().unwrap();
        let nxt = (0..ns).filter(|&j| !used[j]).min_by(|&a, &b| dd(c, a).partial_cmp(&dd(c, b)).unwrap()).unwrap();
        used[nxt] = true;
        tour.push(nxt);
    }
    let tl = 1.5;
    let mut improved = true;
    while improved && start.elapsed().as_secs_f64() < tl {
        improved = false;
        // 2-opt
        for i in 0..ns - 1 {
            for j in i + 2..ns {
                let (a, b) = (tour[i], tour[i + 1]);
                let (c, d) = (tour[j], tour[(j + 1) % ns]);
                if a == d {
                    continue;
                }
                if dd(a, c) + dd(b, d) < dd(a, b) + dd(c, d) - 1e-9 {
                    tour[i + 1..=j].reverse();
                    improved = true;
                }
            }
        }
        // Or-opt（長さ 1〜3 の区間を移動）
        for len in 1..=3usize {
            let mut i = 1;
            while i + len <= ns {
                let p = tour[i - 1];
                let a = tour[i];
                let b = tour[i + len - 1];
                let q = tour[(i + len) % ns];
                let gain_remove = dd(p, a) + dd(b, q) - dd(p, q);
                let mut moved = false;
                for j in 0..ns {
                    if j + 1 >= i && j < i + len {
                        continue;
                    }
                    let (x, y) = (tour[j], tour[(j + 1) % ns]);
                    if (j + 1) % ns == i {
                        continue;
                    }
                    let add1 = dd(x, a) + dd(b, y) - dd(x, y);
                    let add2 = dd(x, b) + dd(a, y) - dd(x, y);
                    if add1.min(add2) < gain_remove - 1e-9 {
                        let mut seg: Vec<usize> = tour.drain(i..i + len).collect();
                        if add2 < add1 {
                            seg.reverse();
                        }
                        let pos = tour.iter().position(|&t| t == x).unwrap() + 1;
                        tour.splice(pos..pos, seg);
                        moved = true;
                        improved = true;
                        break;
                    }
                }
                if !moved {
                    i += 1;
                }
            }
        }
        // 街1 を先頭に保つ
        let z = tour.iter().position(|&t| t == 0).unwrap();
        tour.rotate_left(z);
    }

    // ---- 経路を点列に展開
    let mut path: Vec<P> = vec![towns[0]];
    for k in 0..ns {
        let (a, b) = (tour[k], tour[(k + 1) % ns]);
        // b から a へ prev を辿ると a -> b の順になる（a を始点とした最短路木）
        let prev = &prev_all[a * nv..(a + 1) * nv];
        let mut seq = vec![];
        let mut x = b;
        while x != a {
            seq.push(x);
            x = prev[x] as usize;
        }
        seq.reverse();
        for x in seq {
            if *path.last().unwrap() != nodes[x] {
                path.push(nodes[x]);
            }
        }
    }

    // ---- 給水機の配置
    // 給水点で区切った区間ごとに、追加する給水機の数を貪欲に割り当てる（区間長 G を k+1 等分するとコスト G^2/(k+1)）
    let water: std::collections::HashSet<P> = fountains.iter().cloned().collect();
    let mut cum = vec![0.0f64; path.len()];
    for i in 1..path.len() {
        cum[i] = cum[i - 1] + dist(path[i - 1], path[i]);
    }
    let mut breaks = vec![0usize];
    for i in 1..path.len() - 1 {
        if water.contains(&path[i]) {
            breaks.push(i);
        }
    }
    breaks.push(path.len() - 1);
    let gaps: Vec<(f64, f64)> = breaks.windows(2).map(|w| (cum[w[0]], cum[w[1]] - cum[w[0]])).collect();
    let mut cnt = vec![0usize; gaps.len()];
    for _ in 0..m {
        let gain = |g: usize, k: usize| gaps[g].1 * gaps[g].1 * (1.0 / (k + 1) as f64 - 1.0 / (k + 2) as f64);
        let g = (0..gaps.len()).max_by(|&a, &b| gain(a, cnt[a]).partial_cmp(&gain(b, cnt[b])).unwrap()).unwrap();
        cnt[g] += 1;
    }
    // 目標の弧長位置
    let mut targets: Vec<f64> = vec![];
    for (g, &(st, len)) in gaps.iter().enumerate() {
        for k in 1..=cnt[g] {
            targets.push(st + len * k as f64 / (cnt[g] + 1) as f64);
        }
    }
    targets.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut stations: Vec<P> = vec![];
    let mut newpath: Vec<P> = vec![path[0]];
    let mut ti = 0;
    for i in 1..path.len() {
        let (a, b) = (path[i - 1], path[i]);
        while ti < targets.len() && targets[ti] <= cum[i] {
            let t = ((targets[ti] - cum[i - 1]) / (cum[i] - cum[i - 1]).max(1e-9)).clamp(0.0, 1.0);
            let fx = a.0 as f64 + (b.0 - a.0) as f64 * t;
            let fy = a.1 as f64 + (b.1 - a.1) as f64 * t;
            let last = *newpath.last().unwrap();
            // 近くの整数点で、前後の移動が合法なものを探す
            let mut best: Option<(f64, P)> = None;
            for ox in -2..=2i64 {
                for oy in -2..=2i64 {
                    let p = (fx.round() as i64 + ox, fy.round() as i64 + oy);
                    if p.0 < 0 || p.0 > n || p.1 < 0 || p.1 > n || inside_any(p, &circles) {
                        continue;
                    }
                    if !seg_ok(last, p, &circles) || !seg_ok(p, b, &circles) {
                        continue;
                    }
                    let c = dist(last, p) + dist(p, b);
                    if best.map_or(true, |(bc, _)| c < bc) {
                        best = Some((c, p));
                    }
                }
            }
            let p = best.map(|x| x.1).unwrap_or(last);
            if p != last {
                newpath.push(p);
            }
            stations.push(p);
            ti += 1;
        }
        newpath.push(b);
    }
    stations.sort();
    stations.dedup();

    // ---- 出力
    let mut out = String::new();
    out += &format!("{}\n", stations.len());
    for p in &stations {
        out += &format!("{} {}\n", p.0, p.1);
    }
    out += &format!("{}\n", newpath.len() - 1);
    for p in &newpath[1..] {
        out += &format!("{} {}\n", p.0, p.1);
    }
    print!("{}", out);
}
