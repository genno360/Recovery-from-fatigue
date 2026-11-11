//! Recovery from Fatigue 共通ライブラリ（入力生成・入出力パース・得点計算）
use std::fmt;

pub const N: i64 = 1000;
pub const V: i64 = 1000;
pub const MAX_PATH: usize = 100_000;
pub const PENALTY: i64 = 1_000_000_000;

pub type P = (i64, i64);

// ---------------------------------------------------------------- RNG
/// splitmix64 ベースの決定的乱数（外部クレート不要）
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// [lo, hi] の一様整数
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % ((hi - lo + 1) as u64)) as i64
    }
    /// [0, 1) の一様実数
    pub fn f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn normal(&mut self) -> f64 {
        let u1 = self.f64().max(1e-12);
        let u2 = self.f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

// ---------------------------------------------------------------- geometry
/// 点 p が円 c の内部（境界を含まない）にあるか
pub fn inside(p: P, c: (i64, i64, i64)) -> bool {
    let dx = p.0 - c.0;
    let dy = p.1 - c.1;
    dx * dx + dy * dy < c.2 * c.2
}

/// 線分 pq が円 c の内部を通るか（境界に接するのは可）。整数演算で厳密に判定。
pub fn seg_hits(p: P, q: P, c: (i64, i64, i64)) -> bool {
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

pub fn seg_ok(p: P, q: P, circles: &[(i64, i64, i64)]) -> bool {
    circles.iter().all(|&c| !seg_hits(p, q, c))
}

pub fn in_board(p: P, n: i64) -> bool {
    0 <= p.0 && p.0 <= n && 0 <= p.1 && p.1 <= n
}

// ---------------------------------------------------------------- input
#[derive(Clone, Debug)]
pub struct Input {
    pub n: i64,
    pub m: usize,
    pub v: i64,
    pub f: i64,
    pub towns: Vec<P>,
    pub fountains: Vec<P>,
    pub circles: Vec<(i64, i64, i64)>,
}

impl fmt::Display for Input {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            w,
            "{} {} {} {} {} {} {}",
            self.n,
            self.towns.len(),
            self.fountains.len(),
            self.circles.len(),
            self.m,
            self.v,
            self.f
        )?;
        for &(x, y) in &self.towns {
            writeln!(w, "{} {}", x, y)?;
        }
        for &(x, y) in &self.fountains {
            writeln!(w, "{} {}", x, y)?;
        }
        for &(x, y, r) in &self.circles {
            writeln!(w, "{} {} {}", x, y, r)?;
        }
        Ok(())
    }
}

pub fn parse_input(s: &str) -> Input {
    let mut it = s.split_ascii_whitespace().map(|t| t.parse::<i64>().unwrap());
    let mut nx = || it.next().unwrap();
    let n = nx();
    let s = nx() as usize;
    let w = nx() as usize;
    let k = nx() as usize;
    let m = nx() as usize;
    let v = nx();
    let f = nx();
    let towns = (0..s).map(|_| (nx(), nx())).collect();
    let fountains = (0..w).map(|_| (nx(), nx())).collect();
    let circles = (0..k).map(|_| (nx(), nx(), nx())).collect();
    Input { n, m, v, f, towns, fountains, circles }
}

// ---------------------------------------------------------------- generator
/// 格子点の 8 近傍 BFS で、侵入不可エリアに阻まれない連結成分を求める。
/// 戻り値: 各格子点の成分番号（-1 は円内）と最大成分の番号・サイズ
fn components(n: i64, circles: &[(i64, i64, i64)]) -> (Vec<i32>, i32, usize) {
    let w = (n + 1) as usize;
    let idx = |x: i64, y: i64| (x as usize) * w + y as usize;
    let mut comp = vec![-1i32; w * w];
    let mut free = vec![true; w * w];
    let mut near = vec![false; w * w];
    for &(cx, cy, r) in circles {
        for x in (cx - r - 2).max(0)..=(cx + r + 2).min(n) {
            for y in (cy - r - 2).max(0)..=(cy + r + 2).min(n) {
                let d2 = (x - cx) * (x - cx) + (y - cy) * (y - cy);
                if d2 < r * r {
                    free[idx(x, y)] = false;
                }
                if d2 < (r + 2) * (r + 2) {
                    near[idx(x, y)] = true;
                }
            }
        }
    }
    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    let mut best = (-1, 0usize);
    let mut cid = 0;
    let mut queue = Vec::new();
    for sx in 0..=n {
        for sy in 0..=n {
            let si = idx(sx, sy);
            if !free[si] || comp[si] >= 0 {
                continue;
            }
            comp[si] = cid;
            queue.clear();
            queue.push((sx, sy));
            let mut head = 0;
            while head < queue.len() {
                let (x, y) = queue[head];
                head += 1;
                for &(dx, dy) in &dirs {
                    let (nx, ny) = (x + dx, y + dy);
                    if !in_board((nx, ny), n) {
                        continue;
                    }
                    let ni = idx(nx, ny);
                    if !free[ni] || comp[ni] >= 0 {
                        continue;
                    }
                    if (near[idx(x, y)] || near[ni]) && !seg_ok((x, y), (nx, ny), circles) {
                        continue;
                    }
                    comp[ni] = cid;
                    queue.push((nx, ny));
                }
            }
            if queue.len() > best.1 {
                best = (cid, queue.len());
            }
            cid += 1;
        }
    }
    (comp, best.0, best.1)
}

pub fn gen(seed: u64) -> Input {
    let mut rng = Rng::new(seed);
    let n = N;
    loop {
        let f = rng.range(1, 10);
        let s = rng.range(100, 400) as usize;
        let w = rng.range(0, 4) as usize;
        let m = rng.range(1, 4) as usize;
        let k = rng.range(0, 40) as usize;

        // 侵入不可エリア: 既存の円の近くに置くことで「連なった湖」を作る
        let mut circles: Vec<(i64, i64, i64)> = Vec::new();
        while circles.len() < k {
            let (cx, cy, r);
            if !circles.is_empty() && rng.f64() < 0.6 {
                let c = circles[rng.range(0, circles.len() as i64 - 1) as usize];
                r = rng.range(20, 100);
                let ang = rng.f64() * 2.0 * std::f64::consts::PI;
                let d = (c.2 + r) as f64 * (0.5 + 0.5 * rng.f64());
                cx = c.0 + (d * ang.cos()).round() as i64;
                cy = c.1 + (d * ang.sin()).round() as i64;
            } else {
                r = rng.range(20, 150);
                cx = rng.range(0, n);
                cy = rng.range(0, n);
            }
            if in_board((cx, cy), n) {
                circles.push((cx, cy, r));
            }
        }

        let (comp, main, size) = components(n, &circles);
        if (size as f64) < 0.5 * ((n + 1) * (n + 1)) as f64 {
            continue;
        }
        let w1 = (n + 1) as usize;
        let ok = |p: P| in_board(p, n) && comp[(p.0 as usize) * w1 + p.1 as usize] == main;

        // 街: 一様配置とクラスタ配置の混合
        let nc = rng.range(0, 8) as usize;
        let centers: Vec<(f64, f64, f64)> = (0..nc)
            .map(|_| (rng.range(0, n) as f64, rng.range(0, n) as f64, rng.range(20, 80) as f64))
            .collect();
        let ratio = if nc == 0 { 0.0 } else { rng.f64() * 0.8 };
        let mut used = std::collections::HashSet::new();
        let mut towns = Vec::with_capacity(s);
        while towns.len() < s {
            let p = if rng.f64() < ratio {
                let c = centers[rng.range(0, nc as i64 - 1) as usize];
                ((c.0 + c.2 * rng.normal()).round() as i64, (c.1 + c.2 * rng.normal()).round() as i64)
            } else {
                (rng.range(0, n), rng.range(0, n))
            };
            if ok(p) && used.insert(p) {
                towns.push(p);
            }
        }
        let mut fountains = Vec::with_capacity(w);
        while fountains.len() < w {
            let p = (rng.range(0, n), rng.range(0, n));
            if ok(p) && used.insert(p) {
                fountains.push(p);
            }
        }
        return Input { n, m, v: V, f, towns, fountains, circles };
    }
}

// ---------------------------------------------------------------- output & score
#[derive(Clone, Debug)]
pub struct Output {
    pub stations: Vec<P>,
    pub path: Vec<P>,
}

pub fn parse_output(s: &str) -> Result<Output, String> {
    let toks: Vec<&str> = s.split_ascii_whitespace().collect();
    let mut pos = 0;
    let mut nx = |what: &str| -> Result<i64, String> {
        let t = *toks.get(pos).ok_or(format!("出力が途中で終わっています ({})", what))?;
        pos += 1;
        // 整数は「-」の有無と 1〜18 桁の数字のみ（「+」や小数は不可。JS 版と同じ規則）
        let digits = t.strip_prefix('-').unwrap_or(t);
        if digits.is_empty() || digits.len() > 18 || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("整数ではありません: {}", t));
        }
        Ok(t.parse::<i64>().unwrap())
    };
    let l = nx("L")?;
    if l < 0 {
        return Err(format!("L が負です: {}", l));
    }
    let mut stations = Vec::new();
    for _ in 0..l {
        stations.push((nx("給水機")?, nx("給水機")?));
    }
    let t = nx("T")?;
    if t < 0 || t as usize > MAX_PATH {
        return Err(format!("T が範囲外です: {}", t));
    }
    let mut path = Vec::new();
    for _ in 0..t {
        path.push((nx("経路")?, nx("経路")?));
    }
    if pos < toks.len() {
        return Err(format!("出力の末尾に余分な文字列があります: {}", toks[pos]));
    }
    Ok(Output { stations, path })
}

pub struct Sim {
    pub time: f64,
    pub unvisited: usize,
    pub length: f64,
}

/// 経路をシミュレートする。prefix 個目の点まで移動した状態を返す（全体は prefix = path.len()）。
pub fn simulate(input: &Input, out: &Output, prefix: usize) -> Result<Sim, String> {
    let n = input.n;
    if out.stations.len() > input.m {
        return Err(format!("給水機の数が M を超えています ({} > {})", out.stations.len(), input.m));
    }
    let mut water = std::collections::HashSet::new();
    for &p in &input.fountains {
        water.insert(p);
    }
    for (i, &p) in out.stations.iter().enumerate() {
        if !in_board(p, n) {
            return Err(format!("給水機 {} が盤面外です ({}, {})", i + 1, p.0, p.1));
        }
        if input.circles.iter().any(|&c| inside(p, c)) {
            return Err(format!("給水機 {} が侵入不可エリア内です ({}, {})", i + 1, p.0, p.1));
        }
        water.insert(p);
    }
    let town_id: std::collections::HashMap<P, usize> =
        input.towns.iter().enumerate().map(|(i, &p)| (p, i)).collect();
    let mut visited = vec![false; input.towns.len()];
    visited[0] = true;
    let (v, f) = (input.v as f64, input.f as f64);
    let mut cur = input.towns[0];
    let mut s = 0.0f64; // 前回の給水からの移動距離
    let mut time = 0.0f64;
    let mut length = 0.0f64;
    for (t, &q) in out.path.iter().enumerate().take(prefix) {
        if !in_board(q, n) {
            return Err(format!("{} 番目の点が盤面外です ({}, {})", t + 1, q.0, q.1));
        }
        if !seg_ok(cur, q, &input.circles) {
            return Err(format!(
                "{} 番目の移動 ({}, {}) -> ({}, {}) が侵入不可エリアを通過しています",
                t + 1, cur.0, cur.1, q.0, q.1
            ));
        }
        let dx = (q.0 - cur.0) as f64;
        let dy = (q.1 - cur.1) as f64;
        let d = (dx * dx + dy * dy).sqrt();
        time += v * d + f * (2.0 * s * d + d * d) / 2.0;
        s += d;
        length += d;
        if water.contains(&q) {
            s = 0.0;
        }
        if let Some(&i) = town_id.get(&q) {
            visited[i] = true;
        }
        cur = q;
    }
    if prefix == out.path.len() && cur != input.towns[0] {
        return Err(format!(
            "最終位置 ({}, {}) が街1 ({}, {}) ではありません",
            cur.0, cur.1, input.towns[0].0, input.towns[0].1
        ));
    }
    let unvisited = visited.iter().filter(|&&b| !b).count();
    Ok(Sim { time, unvisited, length })
}

pub fn compute_score(input: &Input, out: &Output) -> Result<(i64, Sim), String> {
    let sim = simulate(input, out, out.path.len())?;
    let score = sim.time.round() as i64 + PENALTY * sim.unvisited as i64;
    Ok((score, sim))
}
