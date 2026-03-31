use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike};
use futures::future::join_all;
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// 单行行情（与前端列 id 对应的数据字段）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRow {
    pub code: String,
    pub name: String,
    pub change_pct: f64,
    pub price: f64,
    pub prev_close: f64,
    pub open: f64,
    pub volume: u64,
    pub turnover: f64,
    pub turnover_rate: f64,
    pub commission_ratio: f64,
    pub total_pl: f64,
    pub daily_pl: f64,
    pub high: f64,
    pub low: f64,
    pub bid1: f64,
    pub ask1: f64,
    /// 买一量（手），列表/快照兜底盘口用；涨停封单等
    #[serde(default)]
    pub bid1_vol: u64,
    /// 卖一量（手）
    #[serde(default)]
    pub ask1_vol: u64,
    /// 东财 `ulist` 量比 `f10`（盯盘列）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_ratio: Option<f64>,
    /// 东财行业板块名 `f100`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,
    /// 同行业板块当日涨跌幅（%），由 `clist` 行业列表按名称映射
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sector_change_pct: Option<f64>,
    /// 东财行业板块代码（`BK`+数字，如 `BK0482`），仅服务端填板块涨幅用，不下发前端
    #[serde(skip)]
    pub industry_board_code: Option<String>,
}

/// 搜索联想结果（前端添加自选用）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestItem {
    pub code: String,
    pub name: String,
    pub quote_id: String,
    /// 东财 `ulist` 行业板块（`f100`，如「光伏设备」）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,
    /// 东财 `ulist` 现价 `f2`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// 东财 `ulist` 涨跌幅 `f3`（%）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_pct: Option<f64>,
    /// 东财股吧人气榜名次（仅人气推荐列表有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<i32>,
}

/// 行情数据源抽象
pub trait QuoteProvider {
    fn fetch(&self) -> Vec<QuoteRow>;
}

pub struct MockQuoteProvider {
    base: Vec<QuoteRow>,
}

impl MockQuoteProvider {
    pub fn new() -> Self {
        Self {
            base: vec![
                row("sh000001", "上证指数", 3050.0, 0.35),
                row("sz399001", "深证成指", 9450.0, -0.12),
                row("sh000300", "沪深300", 3500.0, 0.08),
                row("sz399006", "创业板指", 1850.0, -0.45),
                row("sh510300", "沪深300ETF", 4.02, 0.22),
                row("sh601015", "陕西黑猫", 5.1, -0.35),
            ],
        }
    }
}

fn row(code: &str, name: &str, price: f64, change_pct: f64) -> QuoteRow {
    let prev = price / (1.0 + change_pct / 100.0);
    let open = prev * 1.001;
    QuoteRow {
        code: code.into(),
        name: name.into(),
        change_pct,
        price,
        prev_close: prev,
        open,
        volume: 1_200_000_000,
        turnover: 3.2e10,
        turnover_rate: 0.86,
        commission_ratio: 0.12,
        total_pl: 1280.5,
        daily_pl: 42.3,
        high: price * 1.008,
        low: price * 0.995,
        bid1: price - 0.01,
        ask1: price + 0.01,
        bid1_vol: 12_000,
        ask1_vol: 12_000,
        volume_ratio: Some(1.25),
        sector: Some("示例板块".into()),
        sector_change_pct: Some(0.88),
        industry_board_code: None,
    }
}

impl QuoteProvider for MockQuoteProvider {
    fn fetch(&self) -> Vec<QuoteRow> {
        self.base.clone()
    }
}

pub fn mock_provider() -> MockQuoteProvider {
    MockQuoteProvider::new()
}

/// 按代码列表过滤 Mock 结果；未在 Mock 中的代码返回占位行
pub fn mock_for_codes(codes: &[String]) -> Vec<QuoteRow> {
    let all = mock_provider().fetch();
    let mut out: Vec<QuoteRow> = Vec::new();
    for c in codes {
        let key = c.trim().to_lowercase();
        if let Some(r) = all.iter().find(|r| r.code.to_lowercase() == key) {
            out.push(r.clone());
        } else {
            out.push(QuoteRow {
                code: c.trim().to_lowercase(),
                name: c.trim().to_string(),
                change_pct: 0.0,
                price: 0.0,
                prev_close: 0.0,
                open: 0.0,
                volume: 0,
                turnover: 0.0,
                turnover_rate: 0.0,
                commission_ratio: 0.0,
                total_pl: 0.0,
                daily_pl: 0.0,
                high: 0.0,
                low: 0.0,
                bid1: 0.0,
                ask1: 0.0,
                bid1_vol: 0,
                ask1_vol: 0,
                volume_ratio: None,
                sector: None,
                sector_change_pct: None,
                industry_board_code: None,
            });
        }
    }
    out
}

fn em_client() -> Result<reqwest::Client, String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("http://quote.eastmoney.com/"),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())
}

/// 盘口异动接口（`push2ex.eastmoney.com`）需行情中心异动页 Referer，与 `em_client` 分列避免影响其它请求。
fn em_changes_client() -> Result<reqwest::Client, String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://quote.eastmoney.com/changes/"),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())
}

/// `eq142xsc2605` → `142.sc2605`（东财期货/全球品种 QuoteID 转 secid）
fn eq_internal_to_secid(code: &str) -> Option<String> {
    let c = code.strip_prefix("eq")?;
    let x_pos = c.find('x')?;
    let mkt = &c[..x_pos];
    if mkt.is_empty() || !mkt.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let sym_part = c.get(x_pos + 1..)?;
    if sym_part.is_empty() {
        return None;
    }
    let sym = sym_part.replace('x', ".").to_uppercase();
    Some(format!("{mkt}.{sym}"))
}

/// 横条/搜索快捷：`code` 已小写，映射到东财主用 `secid`（`fetch_one_em` 可再试备用 secid）
fn glob_code_primary_secid_lc(c: &str) -> Option<&'static str> {
    match c {
        "glob_n225" => Some("100.NKY"),
        "glob_cl" => Some("102.CL00Y"),
        "glob_jm" => Some("114.jm888"),
        "glob_j" => Some("114.j888"),
        "glob_zc" => Some("220.zc888"),
        _ => None,
    }
}

/// `sh600519` / `sz000001` / `bj920174` / `hkhsi`（恒生指数）→ 东财 `secid`
pub fn code_to_secid(code: &str) -> Option<String> {
    let c = code.trim().to_lowercase();
    if let Some(s) = glob_code_primary_secid_lc(&c) {
        return Some(s.to_string());
    }
    if let Some(s) = eq_internal_to_secid(&c) {
        return Some(s);
    }
    // 恒生指数：WAP/行情页多为 `100.HSI`；`116.HSI` 为部分旧接口，见 `fetch_hk_hsi_em` 回退
    if c == "hkhsi" {
        return Some("100.HSI".into());
    }
    if c == "fsa50" {
        return Some("104.CN00Y".into());
    }
    if c == "ndx" {
        return Some("100.NDX".into());
    }
    let (prefix, num) = if let Some(rest) = c.strip_prefix("sh") {
        ("1", rest)
    } else if let Some(rest) = c.strip_prefix("sz") {
        ("0", rest)
    } else if let Some(rest) = c.strip_prefix("bj") {
        ("0", rest)
    } else {
        return None;
    };
    if num.is_empty() || !num.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some(format!("{}.{}", prefix, num))
}

/// 内部 `sh600519` / `sz000001` / `bj920174` → 东财 `QuoteID`（与股吧、搜索接口一致）
fn internal_to_quote_id(code: &str) -> Option<String> {
    let c = code.trim().to_lowercase();
    let (mkt, raw_num) = if let Some(r) = c.strip_prefix("sh") {
        ("1", r)
    } else if let Some(r) = c.strip_prefix("sz") {
        ("0", r)
    } else if let Some(r) = c.strip_prefix("bj") {
        ("116", r)
    } else {
        return None;
    };
    if !raw_num.chars().all(|ch| ch.is_ascii_digit()) || raw_num.is_empty() {
        return None;
    }
    let n: u32 = raw_num.parse().ok()?;
    let padded = format!("{:06}", n);
    Some(format!("{}.{}", mkt, padded))
}

/// 内部 `sh601899` / `sz002309` / `bj920174` → `ulist` 的 `secids` 段（与 `em_guba_sc_to_ulist_sec` 一致）
fn internal_code_to_ulist_sec(code: &str) -> Option<String> {
    let c = code.trim().to_lowercase();
    if let Some(s) = glob_code_primary_secid_lc(&c) {
        return Some(s.to_string());
    }
    if let Some(s) = eq_internal_to_secid(&c) {
        return Some(s);
    }
    if c == "hkhsi" {
        return Some("100.HSI".into());
    }
    if c == "fsa50" {
        return Some("104.CN00Y".into());
    }
    if c == "ndx" {
        return Some("100.NDX".into());
    }
    let (head, num_raw) = if let Some(r) = c.strip_prefix("sh") {
        ("SH", r)
    } else if let Some(r) = c.strip_prefix("sz") {
        ("SZ", r)
    } else if let Some(r) = c.strip_prefix("bj") {
        ("BJ", r)
    } else {
        return None;
    };
    if num_raw.is_empty() || !num_raw.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let n: u32 = num_raw.parse().ok()?;
    let padded = format!("{:06}", n);
    match head {
        "SH" => Some(format!("1.{}", padded)),
        "SZ" | "BJ" => Some(format!("0.{}", padded)),
        _ => None,
    }
}

fn em_nonempty_string_field(obj: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    let v = obj.get(key)?;
    if let Some(s) = v.as_str() {
        let t = s.trim();
        if t.is_empty() || t == "-" {
            return None;
        }
        return Some(t.to_string());
    }
    if let Some(n) = v.as_i64() {
        return (n != 0).then(|| n.to_string());
    }
    None
}

fn sector_from_em_f100(diff: &Value) -> Option<String> {
    let obj = diff.as_object()?;
    em_nonempty_string_field(obj, "f100")
}

/// `BK` + 数字 → 映射表用小写键 `bk0482`
fn normalize_em_bk_code(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.len() < 4 {
        return None;
    }
    let u = t.to_ascii_uppercase();
    if !u.starts_with("BK") {
        return None;
    }
    let num = &u[2..];
    if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("bk{}", num))
}

/// 个股 `ulist`/`stock/get` 行：从常见字段里取行业板块代码
fn em_industry_bk_from_stock_row(obj: &serde_json::Map<String, Value>) -> Option<String> {
    for key in ["f101", "f102", "f127", "f128", "f130", "f132"] {
        if let Some(v) = obj.get(key) {
            if let Some(s) = v.as_str() {
                if let Some(bk) = normalize_em_bk_code(s) {
                    return Some(bk);
                }
            }
        }
    }
    None
}

/// 行业 `clist` 行：`f12` 为板块代码（如 `BK0482`）
fn em_board_code_f12_clist(obj: &serde_json::Map<String, Value>) -> Option<String> {
    let v = obj.get("f12")?;
    let s = v.as_str()?;
    normalize_em_bk_code(s)
}

fn json_f64_from_value(val: &Value) -> Option<f64> {
    if let Some(n) = val.as_f64() {
        return Some(n);
    }
    if let Some(n) = val.as_i64() {
        return Some(n as f64);
    }
    val.as_str()?.parse().ok()
}

/// 东财 `ulist` 行情：`f2` 现价、`f3` 涨跌幅（%）
fn em_ulist_price_change(diff: &Value) -> (Option<f64>, Option<f64>) {
    let price = diff
        .get("f2")
        .and_then(json_f64_from_value)
        .filter(|p| p.is_finite() && *p > 0.0);
    let chg = diff
        .get("f3")
        .and_then(json_f64_from_value)
        .filter(|c| c.is_finite());
    (price, chg)
}

fn em_ulist_enrich_from_diff(diff: &Value, item: &mut SuggestItem) {
    if let Some(sec) = sector_from_em_f100(diff) {
        item.sector = Some(sec);
    }
    let (p, c) = em_ulist_price_change(diff);
    if p.is_some() {
        item.price = p;
    }
    if c.is_some() {
        item.change_pct = c;
    }
}

/// 批量补全行业、现价、涨跌幅（`secids` 顺序与 `items` 中有效项顺序一致）
async fn attach_sectors_ulist(client: &reqwest::Client, items: &mut [SuggestItem]) {
    if items.is_empty() {
        return;
    }
    let mut marks: Vec<String> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();
    for (i, it) in items.iter().enumerate() {
        if let Some(m) = internal_code_to_ulist_sec(&it.code) {
            marks.push(m);
            indices.push(i);
        }
    }
    if marks.is_empty() {
        return;
    }
    let secids = marks.join(",");
    let list_url = format!(
        "http://push2.eastmoney.com/api/qt/ulist.np/get?ut=f057cbcbce2a86e2866ab8877db1d059&fltt=2&invt=2&fields=f12,f100,f2,f3&secids={}",
        urlencoding::encode(&secids)
    );
    let Ok(res) = client.get(&list_url).send().await else {
        return;
    };
    let Ok(list_text) = res.text().await else {
        return;
    };
    let Ok(list_v) = serde_json::from_str::<Value>(&list_text) else {
        return;
    };
    if list_v.get("rc").and_then(|v| v.as_i64()).unwrap_or(-1) != 0 {
        return;
    }
    let Some(diffs) = list_v
        .pointer("/data/diff")
        .and_then(|d| d.as_array())
        .cloned()
    else {
        return;
    };
    let n = indices.len().min(diffs.len());
    for j in 0..n {
        let idx = indices[j];
        em_ulist_enrich_from_diff(&diffs[j], &mut items[idx]);
    }
}

/// 股吧返回的 `SH601899` / `SZ002309` / `BJ920174` → `push2` ulist 用的 `secids` 段（如 `1.601899`）
fn em_guba_sc_to_ulist_sec(sc: &str) -> Option<String> {
    internal_code_to_ulist_sec(&em_guba_sc_to_internal(sc)?)
}

/// 股吧 `sc` → 内部代码 `sh601899`
fn em_guba_sc_to_internal(sc: &str) -> Option<String> {
    let s = sc.trim().to_ascii_uppercase();
    if s.len() < 8 {
        return None;
    }
    let head = &s[0..2];
    let num = s[2..].to_lowercase();
    if !num.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    match head {
        "SH" => Some(format!("sh{}", num)),
        "SZ" => Some(format!("sz{}", num)),
        "BJ" => Some(format!("bj{}", num)),
        _ => None,
    }
}

/// 东财 `QuoteID`（如 `1.600519`）→ 内部 `sh600519`；期货/外盘如 `142.sc2605` → `eq142xsc2605`
pub fn quote_id_to_code(quote_id: &str) -> Option<String> {
    let q = quote_id.trim();
    let (mkt, rest) = q.split_once('.')?;
    match mkt {
        "1" | "0" | "116" => {
            let n: u32 = rest.parse().ok()?;
            let padded = format!("{:06}", n);
            let code = match mkt {
                "1" => format!("sh{padded}"),
                "0" => format!("sz{padded}"),
                "116" => format!("bj{padded}"),
                _ => unreachable!(),
            };
            Some(code)
        }
        _ => {
            if !mkt.chars().all(|ch| ch.is_ascii_digit()) {
                return None;
            }
            if rest.is_empty() {
                return None;
            }
            let sym_norm = rest.trim().to_lowercase().replace('.', "x");
            Some(format!("eq{mkt}x{sym_norm}"))
        }
    }
}

/// 单票快照：`f43` 最新价（与东财 App/PC 一致）；`f60` 昨收；`f17` 今开；`f19`/`f31` 买一卖一。
/// 勿用 `f46` 作现价（与 `ulist` 的 `f2` 不同源时易与今开等字段混淆）。
/// `f101`/`f127` 等：部分环境下带行业板块 `BK` 代码，供 `clist` 映射兜底。
const STOCK_GET_FIELDS: &str =
    "f10,f17,f19,f24,f31,f36,f43,f46,f57,f58,f60,f100,f101,f127,f128,f130,f44,f45,f47,f48,f71,f168,f169,f170,f171";

fn json_f64(obj: &serde_json::Map<String, Value>, key: &str) -> Option<f64> {
    obj.get(key).and_then(|v| {
        if let Some(n) = v.as_f64() {
            return Some(n);
        }
        if let Some(n) = v.as_i64() {
            return Some(n as f64);
        }
        v.as_str()?.parse().ok()
    })
}

fn json_u64(obj: &serde_json::Map<String, Value>, key: &str) -> u64 {
    obj.get(key)
        .and_then(|v| {
            if let Some(n) = v.as_u64() {
                return Some(n);
            }
            if let Some(n) = v.as_i64() {
                return Some(n.max(0) as u64);
            }
            if let Some(n) = v.as_f64() {
                return Some(n as u64);
            }
            let s = v.as_str()?;
            let t = s.trim().replace(',', "");
            t.parse().ok()
        })
        .unwrap_or(0)
}

/// 东财 `ulist` 行里 `f12` 可能为字符串或数字
fn json_code_f12(obj: &serde_json::Map<String, Value>) -> Option<String> {
    let v = obj.get("f12")?;
    if let Some(s) = v.as_str() {
        let t = s.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    if let Some(n) = v.as_u64() {
        return Some(n.to_string());
    }
    if let Some(n) = v.as_i64() {
        return Some(n.to_string());
    }
    v.as_f64().map(|x| format!("{}", x.round() as i64))
}

/// `ulist.np/get` 与 `clist.get` 单行字段一致（见 AKShare `stock_zh_a_spot_em`）：`f2` 最新价、`f3` 涨跌幅、`f18` 昨收…
const ULIST_QUOTE_FIELDS: &str =
    "f2,f3,f5,f6,f8,f10,f12,f13,f14,f15,f16,f17,f18,f19,f24,f31,f36,f100,f101,f127,f128,f130";

/// 东财 `ulist.np` 与 `stock/get` 字段语义不一致：实测 `f31` 多为买一价，`f19` 常为非价字段（如小整数）；
/// 当 `f19`、`f31` 均在现价附近时按 bid≤ask 与现价位置自动配对；否则以 `f31` 为主推断买一，并据现价补卖一。
fn parse_ulist_bid_ask(last: f64, f19: Option<f64>, f31: Option<f64>) -> (f64, f64) {
    const NEAR_PCT: f64 = 0.2;
    let near_last = |x: f64| {
        last.is_finite()
            && last > 1e-9
            && x.is_finite()
            && x > 0.0
            && ((x - last).abs() / last) <= NEAR_PCT
    };
    let pf19 = f19.filter(|&x| near_last(x));
    let pf31 = f31.filter(|&x| near_last(x));
    let tol = last.max(0.01) * 0.002 + 1e-6;

    let mut bid1 = 0.0_f64;
    let mut ask1 = 0.0_f64;

    match (pf19, pf31) {
        (Some(a), Some(b)) if a <= b && a <= last + tol && last <= b + tol => {
            bid1 = a;
            ask1 = b;
        }
        (Some(a), Some(b)) if b <= a && b <= last + tol && last <= a + tol => {
            bid1 = b;
            ask1 = a;
        }
        (_, Some(b)) => {
            bid1 = b;
        }
        (Some(a), None) => {
            bid1 = a;
        }
        (None, None) => {}
    }

    if bid1 > 1e-9 && ask1 < 1e-9 {
        let tick = if last >= 10.0 { 0.01 } else if last >= 1.0 { 0.01 } else { 0.001 };
        ask1 = if last > bid1 + 1e-9 {
            last
        } else {
            (bid1 + tick).max(last)
        };
    }

    if bid1 < 1e-9 && last > 1e-9 {
        bid1 = (last - 0.01).max(0.01);
    }
    if ask1 < 1e-9 && last > 1e-9 {
        if bid1 > 1e-9 {
            let tick = if last >= 10.0 { 0.01 } else { 0.001 };
            ask1 = if last + 1e-9 < bid1 {
                last
            } else {
                (bid1 + tick).max(last)
            };
        } else {
            ask1 = last + 0.01;
        }
    }

    (bid1, ask1)
}

fn code_digits_match_f12(internal_lower: &str, f12: &str) -> bool {
    let t = f12.trim();
    if internal_lower == "hkhsi" && t.eq_ignore_ascii_case("HSI") {
        return true;
    }
    if internal_lower == "fsa50" && t.eq_ignore_ascii_case("CN00Y") {
        return true;
    }
    if internal_lower == "ndx" && t.eq_ignore_ascii_case("NDX") {
        return true;
    }
    if let Some(sym) = internal_lower.strip_prefix("eq").and_then(|rest| rest.find('x').map(|p| &rest[p + 1..])) {
        let sym_lc = sym.replace('x', ".").to_lowercase();
        let tl = t.to_lowercase();
        if tl == sym_lc || tl.ends_with(&sym_lc) || sym_lc.ends_with(&tl) {
            return true;
        }
    }
    if internal_lower == "glob_n225" && t.eq_ignore_ascii_case("NKY") {
        return true;
    }
    if internal_lower == "glob_cl" && t.eq_ignore_ascii_case("CL00Y") {
        return true;
    }
    if internal_lower == "glob_jm" && t.eq_ignore_ascii_case("jm888") {
        return true;
    }
    if internal_lower == "glob_j" && t.eq_ignore_ascii_case("j888") {
        return true;
    }
    if internal_lower == "glob_zc" && t.eq_ignore_ascii_case("zc888") {
        return true;
    }
    let digits: String = internal_lower
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    if t.is_empty() || digits.is_empty() {
        return false;
    }
    if digits == t {
        return true;
    }
    t.parse::<u32>()
        .ok()
        .map(|n| digits == format!("{n:06}"))
        .unwrap_or(false)
        || t.parse::<u64>().ok().map(|n| digits == n.to_string()).unwrap_or(false)
}

fn parse_em_ulist_diff_row(internal_code_lower: &str, diff: &Value) -> Option<QuoteRow> {
    let obj = diff.as_object()?;
    let f12 = json_code_f12(obj)?;
    if !code_digits_match_f12(internal_code_lower, &f12) {
        return None;
    }
    let name = obj
        .get("f14")
        .and_then(|v| v.as_str())
        .unwrap_or(internal_code_lower)
        .to_string();
    let price = json_f64(obj, "f2").filter(|p| p.is_finite() && *p > 0.0)?;
    let change_pct = json_f64(obj, "f3").unwrap_or_else(|| {
        let prev = json_f64(obj, "f18").unwrap_or(0.0);
        if prev > 1e-9 {
            (price - prev) / prev * 100.0
        } else {
            0.0
        }
    });
    let mut prev_close = json_f64(obj, "f18").filter(|p| p.is_finite() && *p > 0.0).unwrap_or(0.0);
    if prev_close < 1e-9 && price > 1e-9 && change_pct.is_finite() {
        prev_close = price / (1.0 + change_pct / 100.0);
    }
    let open = json_f64(obj, "f17")
        .filter(|o| o.is_finite() && *o > 0.0)
        .unwrap_or(prev_close);
    let high = json_f64(obj, "f15")
        .filter(|h| h.is_finite() && *h > 0.0)
        .unwrap_or(price);
    let low = json_f64(obj, "f16")
        .filter(|l| l.is_finite() && *l > 0.0)
        .unwrap_or(price);
    let volume = json_u64(obj, "f5");
    let turnover = json_f64(obj, "f6").unwrap_or(0.0);
    let turnover_rate = json_f64(obj, "f8")
        .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 100.0)
        .unwrap_or(0.0);
    let raw19 = json_f64(obj, "f19").filter(|x| x.is_finite());
    let raw31 = json_f64(obj, "f31").filter(|x| x.is_finite());
    let (bid1, ask1) = parse_ulist_bid_ask(price, raw19, raw31);
    let bid1_vol = json_u64(obj, "f24");
    let ask1_vol = json_u64(obj, "f36");
    let volume_ratio = json_f64(obj, "f10").filter(|v| v.is_finite() && *v >= 0.0 && *v < 1_000_000.0);
    let sector = em_nonempty_string_field(obj, "f100");
    let industry_board_code = em_industry_bk_from_stock_row(obj);
    Some(QuoteRow {
        code: internal_code_lower.to_string(),
        name,
        change_pct,
        price,
        prev_close,
        open,
        volume,
        turnover,
        turnover_rate,
        commission_ratio: 0.0,
        total_pl: 0.0,
        daily_pl: 0.0,
        high,
        low,
        bid1,
        ask1,
        bid1_vol,
        ask1_vol,
        volume_ratio,
        sector,
        sector_change_pct: None,
        industry_board_code,
    })
}

/// 批量 `ulist.np/get`（与东财沪深京 A 股列表同源字段），顺序与 `secids` 一致；校验 `f12` 与内部代码。
async fn fetch_quotes_eastmoney_ulist(
    client: &reqwest::Client,
    codes: &[String],
) -> Result<std::collections::HashMap<String, QuoteRow>, String> {
    const CHUNK: usize = 80;
    let mut pairs: Vec<(String, String)> = Vec::with_capacity(codes.len());
    for c in codes {
        let internal = c.trim().to_lowercase();
        if let Some(sec) = code_to_secid(c.trim()) {
            pairs.push((internal, sec));
        }
    }
    if pairs.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let mut map = std::collections::HashMap::new();
    for chunk in pairs.chunks(CHUNK) {
        let secids = chunk
            .iter()
            .map(|(_, s)| s.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "http://push2.eastmoney.com/api/qt/ulist.np/get?ut=f057cbcbce2a86e2866ab8877db1d059&fltt=2&invt=2&fields={}&secids={}",
            ULIST_QUOTE_FIELDS,
            urlencoding::encode(&secids)
        );
        let text = client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                println!("ulist reqwest error: {}", e);
                e.to_string()
            })?
            .text()
            .await
            .map_err(|e| {
                println!("ulist text error: {}", e);
                e.to_string()
            })?;
        // println!("ulist response: {}", text);
        let v: Value = serde_json::from_str(&text).map_err(|e| {
            println!("ulist json error: {}", e);
            e.to_string()
        })?;
        if v.get("rc").and_then(|x| x.as_i64()).unwrap_or(-1) != 0 {
            println!("ulist rc != 0: {}", text);
            return Err("东财 ulist 行情 rc!=0".into());
        }
        let diff = v
            .pointer("/data/diff")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();
        for item in diff {
            let Some(obj) = item.as_object() else { continue; };
            let Some(f12) = json_code_f12(obj) else {
                continue;
            };
            let mut matched_internal = None;
            for (internal, _sec) in chunk.iter() {
                if code_digits_match_f12(internal, &f12) {
                    matched_internal = Some(internal.clone());
                    break;
                }
            }
            if let Some(internal) = matched_internal {
                if let Some(q) = parse_em_ulist_diff_row(&internal, &item) {
                    map.insert(internal, q);
                }
            }
        }
    }
    Ok(map)
}

/// 东财 `push2` 单票快照 → `QuoteRow`（`internal_code` 为配置里的 `sh/sz/bj` 代码）
fn parse_em_stock_json(internal_code: &str, body: &Value) -> Result<QuoteRow, String> {
    let rc = body.get("rc").and_then(|v| v.as_i64()).unwrap_or(-1);
    if rc != 0 {
        return Err(format!("东财 rc={}", rc));
    }
    let data = body
        .get("data")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "东财 data 缺失".to_string())?;

    let name = data
        .get("f58")
        .and_then(|v| v.as_str())
        .unwrap_or(internal_code)
        .to_string();
    let price = json_f64(data, "f43")
        .filter(|p| p.is_finite() && *p > 0.0)
        .or_else(|| json_f64(data, "f46").filter(|p| p.is_finite() && *p > 0.0))
        .unwrap_or(0.0);
    let mut prev_close = json_f64(data, "f60")
        .filter(|p| p.is_finite() && *p > 0.0)
        .unwrap_or(0.0);
    if prev_close < 1e-9 && price > 1e-9 {
        prev_close = json_f64(data, "f170")
            .filter(|x| x.is_finite())
            .map(|pct| price / (1.0 + pct / 100.0))
            .filter(|p| p.is_finite() && *p > 1e-9)
            .unwrap_or(price);
    }
    let high = json_f64(data, "f44").unwrap_or(price);
    let low = json_f64(data, "f45").unwrap_or(price);
    let volume = json_u64(data, "f47");
    let turnover = json_f64(data, "f48").unwrap_or(0.0);
    let mut bid1 = json_f64(data, "f19")
        .filter(|b| b.is_finite() && *b > 0.0)
        .unwrap_or(0.0);

    let change_pct = if prev_close > 1e-9 && price > 1e-9 {
        (price - prev_close) / prev_close * 100.0
    } else {
        json_f64(data, "f170").unwrap_or(0.0)
    };

    let turnover_rate = json_f64(data, "f171")
        .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 100.0)
        .unwrap_or(0.0);

    let mut ask1 = json_f64(data, "f31")
        .filter(|a| a.is_finite() && *a > 0.0)
        .unwrap_or(0.0);
    if ask1 < 1e-9 && price > 0.0 {
        if bid1 > 1e-9 {
            ask1 = if price >= bid1 {
                price
            } else {
                (bid1 + 0.01).min(price + 0.05).max(price)
            };
        } else {
            ask1 = price + 0.01;
        }
    }
    if bid1 < 1e-9 && price > 0.0 {
        bid1 = (price - 0.01).max(0.01);
    }

    let open = json_f64(data, "f17")
        .filter(|o| o.is_finite() && *o > 0.0)
        .unwrap_or(prev_close);

    let bid1_vol = json_u64(data, "f24");
    let ask1_vol = json_u64(data, "f36");
    let volume_ratio = json_f64(data, "f10").filter(|v| v.is_finite() && *v >= 0.0 && *v < 1_000_000.0);
    let sector = em_nonempty_string_field(data, "f100");
    let industry_board_code = em_industry_bk_from_stock_row(data);

    Ok(QuoteRow {
        code: internal_code.trim().to_lowercase(),
        name,
        change_pct,
        price,
        prev_close,
        open,
        volume,
        turnover,
        turnover_rate,
        commission_ratio: 0.0,
        total_pl: 0.0,
        daily_pl: 0.0,
        high,
        low,
        bid1,
        ask1,
        bid1_vol,
        ask1_vol,
        volume_ratio,
        sector,
        sector_change_pct: None,
        industry_board_code,
    })
}

/// 恒生指数单独 `stock/get`：`ulist` 混 A 股时易失败或 `f2` 不满足解析条件
const HK_HSI_STOCK_GET_SECIDS: &[&str] = &["100.HSI", "116.HSI"];

async fn fetch_hk_hsi_em(client: &reqwest::Client) -> QuoteRow {
    for secid in HK_HSI_STOCK_GET_SECIDS {
        let url = format!(
            "http://push2.eastmoney.com/api/qt/stock/get?invt=2&fltt=2&secid={}&fields={}",
            urlencoding::encode(secid),
            STOCK_GET_FIELDS
        );
        let res = match client.get(&url).send().await {
            Ok(r) => r,
            Err(_) => continue,
        };
        let text = match res.text().await {
            Ok(t) => t,
            Err(_) => continue,
        };
        let v: Value = match serde_json::from_str(&text) {
            Ok(j) => j,
            Err(_) => continue,
        };
        if let Ok(row) = parse_em_stock_json("hkhsi", &v) {
            if row.price > 1e-6 || row.prev_close > 1e-6 || row.change_pct != 0.0 {
                return row;
            }
        }
    }
    error_row("hkhsi", "恒生指数无数据")
}

/// 期货/外盘主连 secid 可能调整：按列表依次试 `stock/get`
async fn fetch_one_em_try_secids(
    client: &reqwest::Client,
    internal_code: &str,
    secids: &[&str],
) -> QuoteRow {
    let ic = internal_code.trim().to_lowercase();
    for secid in secids {
        let url = format!(
            "http://push2.eastmoney.com/api/qt/stock/get?invt=2&fltt=2&secid={}&fields={}",
            urlencoding::encode(secid),
            STOCK_GET_FIELDS
        );
        let Ok(res) = client.get(&url).send().await else {
            continue;
        };
        let Ok(text) = res.text().await else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        if let Ok(row) = parse_em_stock_json(&ic, &v) {
            if row.price > 1e-9 || row.prev_close > 1e-9 || row.change_pct.abs() > 1e-6 {
                return row;
            }
        }
    }
    error_row(&ic, "无数据")
}

async fn fetch_one_em(client: &reqwest::Client, internal_code: &str) -> QuoteRow {
    if internal_code.trim().eq_ignore_ascii_case("hkhsi") {
        return fetch_hk_hsi_em(client).await;
    }
    let ic = internal_code.trim().to_lowercase();
    match ic.as_str() {
        "glob_n225" => {
            return fetch_one_em_try_secids(
                client,
                &ic,
                &["100.NKY", "100.N225", "100.NIKKEI225"],
            )
            .await;
        }
        "glob_cl" => {
            return fetch_one_em_try_secids(
                client,
                &ic,
                &["102.CL00Y", "103.CL00Y", "104.CL00Y", "152.CL00Y"],
            )
            .await;
        }
        "glob_jm" => {
            return fetch_one_em_try_secids(
                client,
                &ic,
                &["114.jm888", "114.jm9999", "114.jm0001", "114.JM888"],
            )
            .await;
        }
        "glob_j" => {
            return fetch_one_em_try_secids(
                client,
                &ic,
                &["114.j888", "114.j9999", "114.j0001", "114.J888"],
            )
            .await;
        }
        "glob_zc" => {
            return fetch_one_em_try_secids(
                client,
                &ic,
                &["220.zc888", "179.zc888", "114.zc888"],
            )
            .await;
        }
        _ => {}
    }
    let Some(secid) = code_to_secid(internal_code) else {
        return error_row(internal_code, "代码格式无效");
    };
    let url = format!(
        "http://push2.eastmoney.com/api/qt/stock/get?invt=2&fltt=2&secid={}&fields={}",
        urlencoding::encode(&secid),
        STOCK_GET_FIELDS
    );
    let res = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            println!("fetch_one_em reqwest error for {}: {}", internal_code, e);
            return error_row(internal_code, &e.to_string());
        }
    };
    let text = match res.text().await {
        Ok(t) => t,
        Err(e) => {
            println!("fetch_one_em text error for {}: {}", internal_code, e);
            return error_row(internal_code, &e.to_string());
        }
    };
    // println!("fetch_one_em response for {}: {}", internal_code, text);
    let v: Value = match serde_json::from_str(&text) {
        Ok(j) => j,
        Err(e) => {
            println!("fetch_one_em json error for {}: {}", internal_code, e);
            return error_row(internal_code, &e.to_string());
        }
    };
    match parse_em_stock_json(internal_code, &v) {
        Ok(row) => row,
        Err(e) => {
            println!("fetch_one_em parse error for {}: {}", internal_code, e);
            error_row(internal_code, &e)
        }
    }
}

fn error_row(internal_code: &str, _msg: &str) -> QuoteRow {
    QuoteRow {
        code: internal_code.trim().to_lowercase(),
        name: internal_code.trim().to_string(),
        change_pct: 0.0,
        price: 0.0,
        prev_close: 0.0,
        open: 0.0,
        volume: 0,
        turnover: 0.0,
        turnover_rate: 0.0,
        commission_ratio: 0.0,
        total_pl: 0.0,
        daily_pl: 0.0,
        high: 0.0,
        low: 0.0,
        bid1: 0.0,
        ask1: 0.0,
        bid1_vol: 0,
        ask1_vol: 0,
        volume_ratio: None,
        sector: None,
        sector_change_pct: None,
        industry_board_code: None,
    }
}

/// 主路径：`ulist.np/get` 批量（与东财 A 股列表/APP 同源 `f2` 现价、`f18` 昨收）；缺项再 `stock/get`（`f43` 现价）。
pub async fn fetch_quotes_eastmoney(codes: &[String]) -> Result<Vec<QuoteRow>, String> {
    if codes.is_empty() {
        return Ok(vec![]);
    }
    let client = em_client()?;
    let sector_map = fetch_industry_board_name_to_change(&client).await;
    let unique: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        let mut v = Vec::new();
        for c in codes {
            let k = c.trim().to_lowercase();
            if seen.insert(k.clone()) {
                v.push(c.trim().to_string());
            }
        }
        v
    };
    let ulist_map = fetch_quotes_eastmoney_ulist(&client, &unique)
        .await
        .unwrap_or_default();

    let mut need: Vec<String> = Vec::new();
    let mut by_code: std::collections::HashMap<String, QuoteRow> =
        std::collections::HashMap::new();
    for c in &unique {
        let k = c.trim().to_lowercase();
        if let Some(r) = ulist_map.get(&k) {
            if r.price > 1e-9 || r.prev_close > 1e-9 {
                by_code.insert(k, r.clone());
                continue;
            }
        }
        need.push(c.clone());
    }

    if !need.is_empty() {
        let futs: Vec<_> = need
            .iter()
            .map(|c| {
                let c = c.clone();
                let cl = client.clone();
                async move { fetch_one_em(&cl, &c).await }
            })
            .collect();
        for r in futures::future::join_all(futs).await {
            by_code.insert(r.code.to_lowercase(), r);
        }
    }

    for r in by_code.values_mut() {
        attach_sector_change_from_map(r, &sector_map);
    }

    Ok(codes
        .iter()
        .map(|c| {
            let k = c.trim().to_lowercase();
            by_code
                .get(&k)
                .cloned()
                .unwrap_or_else(|| error_row(c, "无数据"))
        })
        .collect())
}

#[derive(Debug, serde::Deserialize)]
struct SuggestDataRow {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "QuoteID")]
    quote_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct SuggestTable {
    #[serde(rename = "Data")]
    data: Option<Vec<SuggestDataRow>>,
}

#[derive(Debug, serde::Deserialize)]
struct SuggestRoot {
    #[serde(rename = "QuotationCodeTable")]
    quotation_code_table: Option<SuggestTable>,
}

/// 中文/英文关键词 → 横条同源快捷代码（搜索框可直达）
fn search_inject_ribbon_aliases(query: &str) -> Vec<SuggestItem> {
    let q = query.trim();
    if q.is_empty() {
        return vec![];
    }
    let ql = q.to_lowercase();
    let mut v: Vec<SuggestItem> = Vec::new();
    let mut push = |code: &str, name: &str, qid: &str| {
        v.push(SuggestItem {
            code: code.into(),
            name: name.into(),
            quote_id: qid.into(),
            sector: None,
            price: None,
            change_pct: None,
            rank: None,
        });
    };
    if q.contains("日经") || ql.contains("nikkei") || ql == "n225" {
        push("glob_n225", "日经225", "100.NKY");
    }
    if q.contains("原油")
        || q.contains("美油")
        || q.contains("WTI")
        || ql.contains("wti")
        || q.contains("NYMEX")
        || ql.contains("nymex")
        || q.contains("石油")
    {
        push("glob_cl", "NYMEX原油(主连)", "102.CL00Y");
    }
    if q.contains("焦煤") {
        push("glob_jm", "焦煤(主连)", "114.jm888");
    }
    if q.contains("焦炭") && !q.contains("焦煤") {
        push("glob_j", "焦炭(主连)", "114.j888");
    }
    if q.contains("动力煤") || (q.contains("郑煤") && q.contains("期货")) {
        push("glob_zc", "动力煤(主连)", "220.zc888");
    }
    v
}

async fn search_suggest_fetch_rows(client: &reqwest::Client, q: &str, type_mask: &str) -> Vec<SuggestDataRow> {
    let url = format!(
        "https://searchadapter.eastmoney.com/api/suggest/get?input={}&type={}&count=24",
        urlencoding::encode(q),
        urlencoding::encode(type_mask)
    );
    let Ok(res) = client.get(&url).send().await else {
        return vec![];
    };
    let Ok(text) = res.text().await else {
        return vec![];
    };
    let Ok(root) = serde_json::from_str::<SuggestRoot>(&text) else {
        return vec![];
    };
    root.quotation_code_table
        .and_then(|t| t.data)
        .unwrap_or_default()
}

/// 东财搜索联想（A 股、ETF、指数、期货等；`type=14` 偏股票，再并一条宽类型提高期货命中率）
pub async fn search_securities_eastmoney(query: &str) -> Result<Vec<SuggestItem>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let client = em_client()?;
    let (rows_a, rows_b) = futures::join!(
        search_suggest_fetch_rows(&client, q, "14"),
        search_suggest_fetch_rows(&client, q, "8191"),
    );
    let mut seen_quote: HashSet<String> = HashSet::new();
    let mut out: Vec<SuggestItem> = Vec::new();
    for inj in search_inject_ribbon_aliases(q) {
        seen_quote.insert(inj.quote_id.to_lowercase());
        out.push(inj);
    }
    for r in rows_a.into_iter().chain(rows_b) {
        let qid = r.quote_id.trim().to_string();
        if qid.is_empty() || !seen_quote.insert(qid.to_lowercase()) {
            continue;
        }
        let Some(internal) = quote_id_to_code(&qid) else {
            continue;
        };
        out.push(SuggestItem {
            code: internal,
            name: r.name,
            quote_id: qid,
            sector: None,
            price: None,
            change_pct: None,
            rank: None,
        });
    }
    attach_sectors_ulist(&client, &mut out).await;
    Ok(out)
}

/// 东方财富股吧「个股人气榜」当前排名（`https://guba.eastmoney.com/rank/` 同源接口）
pub async fn fetch_hot_stocks_eastmoney(limit: usize) -> Result<Vec<SuggestItem>, String> {
    let page_size = limit.clamp(1, 100);
    let client = em_client()?;
    let rank_url = "https://emappdata.eastmoney.com/stockrank/getAllCurrentList";
    let body = serde_json::json!({
        "appId": "appId01",
        "globalId": "786e4c21-70dc-435a-93bb-38",
        "marketType": "",
        "pageNo": 1,
        "pageSize": page_size,
    });
    let rank_text = client
        .post(rank_url)
        .header("Origin", "https://guba.eastmoney.com")
        .header("Referer", "https://guba.eastmoney.com/rank/")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let rank_v: Value = serde_json::from_str(&rank_text).map_err(|e| e.to_string())?;
    if rank_v.get("status").and_then(|v| v.as_i64()).unwrap_or(-1) != 0 {
        let msg = rank_v
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("人气榜接口异常");
        return Err(msg.to_string());
    }
    let rows = rank_v
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    if rows.is_empty() {
        return Ok(vec![]);
    }

    let mut marks: Vec<String> = Vec::new();
    let mut metas: Vec<(String, i32)> = Vec::new();
    for row in &rows {
        let sc = row
            .get("sc")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let rk = row.get("rk").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let Some(sec) = em_guba_sc_to_ulist_sec(&sc) else {
            continue;
        };
        let Some(internal) = em_guba_sc_to_internal(&sc) else {
            continue;
        };
        marks.push(sec);
        metas.push((internal, rk));
    }
    if marks.is_empty() {
        return Ok(vec![]);
    }

    let secids = marks.join(",");
    let list_url = format!(
        "http://push2.eastmoney.com/api/qt/ulist.np/get?ut=f057cbcbce2a86e2866ab8877db1d059&fltt=2&invt=2&fields=f12,f14,f100,f2,f3&secids={}",
        urlencoding::encode(&secids)
    );
    let list_text = client
        .get(&list_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let list_v: Value = serde_json::from_str(&list_text).map_err(|e| e.to_string())?;
    if list_v.get("rc").and_then(|v| v.as_i64()).unwrap_or(-1) != 0 {
        return Err("人气榜行情补全失败".into());
    }
    let diffs = list_v
        .pointer("/data/diff")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    let n = metas.len().min(diffs.len());
    let mut out: Vec<SuggestItem> = Vec::with_capacity(n);
    for i in 0..n {
        let (internal, rk) = &metas[i];
        let diff = &diffs[i];
        let name = diff
            .get("f14")
            .and_then(|v| v.as_str())
            .unwrap_or(internal)
            .to_string();
        let quote_id = internal_to_quote_id(internal).unwrap_or_default();
        let sector = sector_from_em_f100(diff);
        let (price, change_pct) = em_ulist_price_change(diff);
        out.push(SuggestItem {
            code: internal.clone(),
            name,
            quote_id,
            sector,
            price,
            change_pct,
            rank: Some(*rk),
        });
    }
    Ok(out)
}

fn name_looks_like_raw_code(name: &str, code: &str) -> bool {
    let n = name.trim().to_lowercase();
    let c = code.trim().to_lowercase();
    n == c || n.is_empty()
}

/// 东财失败或返回空行情时，用同代码的 Mock 行兜底；名称仍像代码时用 Mock 中文名
fn merge_em_row_with_mock(em: QuoteRow, mock: &QuoteRow) -> QuoteRow {
    let mut out = em;
    if name_looks_like_raw_code(&out.name, &out.code)
        && !name_looks_like_raw_code(&mock.name, &mock.code)
    {
        out.name = mock.name.clone();
    }
    out
}

pub async fn get_quotes_impl(codes: Vec<String>, quote_source: &str) -> Result<Vec<QuoteRow>, String> {
    println!("get_quotes_impl called with source: {}", quote_source);
    match quote_source {
        "mock" => Ok(mock_for_codes(&codes)),
        "tencent" => fetch_quotes_tencent(&codes).await,
        _ => {
            let em_rows = match fetch_quotes_eastmoney(&codes).await {
                Ok(rows) => rows,
                Err(e) => {
                    println!("fetch_quotes_eastmoney failed: {}", e);
                    return Err(e);
                }
            };
            let mock_rows = mock_for_codes(&codes);
            let merged: Vec<QuoteRow> = em_rows
                .into_iter()
                .zip(mock_rows.iter())
                .map(|(em, m)| merge_em_row_with_mock(em, m))
                .collect();
            Ok(merged)
        }
    }
}

// --- 个股详情：分时 / K 线 / 五档（东财 push2his / stock/get 扩展字段）---

/// 东财 `stock/get` 五档 + `f43` 最新价；买一价 `f19`（`f71` 为均价，勿作买一）
const ORDER_BOOK_FIELDS: &str =
    "f19,f20,f21,f22,f23,f24,f25,f26,f27,f28,f31,f32,f33,f34,f35,f36,f37,f38,f39,f40,f43,f46,f60,f71";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradayPoint {
    pub time: i64,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
    /// 该分钟成交额（元），东财逗号行第 7 段
    #[serde(default)]
    pub turnover: f64,
    /// 累计成交均价（分时均价线），东财逗号行第 8 段
    #[serde(default)]
    pub avg_price: f64,
}

/// 集合竞价撮合结果（`iscr=1` 时 9:25 后首笔带量分钟，一般为 9:26 档）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradayAuctionSummary {
    pub match_price: f64,
    /// 成交量（手）
    pub match_volume_lots: u64,
    /// 撮合成交额（元）
    pub match_turnover: f64,
    /// 较昨收涨跌幅（%）
    pub pct_vs_pre_close: f64,
    /// 上一交易日 9:30 首分钟成交额（元），与竞价额口径不同，仅作对照
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_ref_turnover: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradaySeries {
    pub pre_close: f64,
    pub points: Vec<IntradayPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auction: Option<IntradayAuctionSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KlinePoint {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    /// 成交额（元），东财日 K 行第 7 段
    #[serde(default)]
    pub turnover: f64,
    /// 换手率（%），东财日 K 行第 11 段
    #[serde(default)]
    pub turnover_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KlineSeries {
    pub points: Vec<KlinePoint>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBookLevel {
    pub price: f64,
    pub volume: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    /// 卖五 → 卖一（上到下，远离现价 → 靠近现价）
    pub asks: Vec<OrderBookLevel>,
    /// 买一 → 买五
    pub bids: Vec<OrderBookLevel>,
    /// 有效档位数（买卖两侧较大值），供前端提示「当前为 N 档」
    pub max_levels: u32,
}

fn shanghai_offset() -> FixedOffset {
    FixedOffset::east_opt(8 * 3600).expect("CST offset")
}

/// `"2026-03-20 09:30"` → Unix 秒（按东八区）
fn parse_shanghai_minute_ts(s: &str) -> Option<i64> {
    let naive = NaiveDateTime::parse_from_str(s.trim(), "%Y-%m-%d %H:%M").ok()?;
    shanghai_offset()
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.timestamp())
}

/// 日线日期 `"2026-03-18"` → 当日 0 点（东八区）Unix 秒
fn parse_kline_day_ts(s: &str) -> Option<i64> {
    let d = NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()?;
    let naive = d.and_time(NaiveTime::from_hms_opt(0, 0, 0)?);
    shanghai_offset()
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.timestamp())
}

/// trends2 单行：`时间,开,收,高,低,分成交量,分成交额,均价,...,累计量`
fn parse_trend_line_simple(s: &str) -> Option<IntradayPoint> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() < 6 {
        return None;
    }
    let ts = parse_shanghai_minute_ts(parts[0])?;
    let open = parts[1].parse::<f64>().ok()?;
    let close = parts[2].parse::<f64>().ok()?;
    let high = parts[3].parse::<f64>().ok()?;
    let low = parts[4].parse::<f64>().ok()?;
    let vol = parts[5].parse::<u64>().ok().unwrap_or(0);
    let turnover = parts
        .get(6)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|x| x.is_finite())
        .unwrap_or(0.0);
    let avg_price = parts
        .get(7)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|x| x.is_finite() && *x > 0.0)
        .unwrap_or(0.0);
    if !open.is_finite() {
        return None;
    }
    Some(IntradayPoint {
        time: ts,
        open,
        close,
        high,
        low,
        volume: vol,
        turnover,
        avg_price,
    })
}

fn minutes_since_midnight_shanghai(ts: i64) -> Option<u32> {
    let dt = shanghai_offset().timestamp_opt(ts, 0).single()?;
    let t = dt.time();
    Some(t.hour() * 60 + t.minute())
}

/// 9:25 及之后首根「有成交量」的分钟：东财写入集合竞价撮合量额
fn summarize_intraday_auction(points: &[IntradayPoint], pre_close: f64) -> Option<IntradayAuctionSummary> {
    const AFTER_MATCH: u32 = 9 * 60 + 25;
    for p in points {
        let mm = minutes_since_midnight_shanghai(p.time)?;
        if mm < AFTER_MATCH || p.volume == 0 {
            continue;
        }
        let pct = if pre_close > 1e-9 && pre_close.is_finite() {
            (p.close - pre_close) / pre_close * 100.0
        } else {
            0.0
        };
        return Some(IntradayAuctionSummary {
            match_price: p.close,
            match_volume_lots: p.volume,
            match_turnover: p.turnover,
            pct_vs_pre_close: pct,
            prev_ref_turnover: None,
        });
    }
    None
}

/// `ndays=2` 时数据按交易日正序；取上一交易日首根分时（多为 9:30）的成交额，作竞价额对照
fn prev_trading_day_first_bar_turnover(body: &Value) -> Option<f64> {
    let trends = body
        .get("data")?
        .get("trends")?
        .as_array()?;
    if trends.len() < 2 {
        return None;
    }
    let first = trends.first()?.as_str()?;
    let last = trends.last()?.as_str()?;
    let d0 = first.get(0..10)?;
    let d1 = last.get(0..10)?;
    if d0 == d1 {
        return None;
    }
    let parts: Vec<&str> = first.split(',').collect();
    parts
        .get(6)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|x| x.is_finite())
}

fn parse_em_intraday_json(body: &Value) -> Result<IntradaySeries, String> {
    let rc = body.get("rc").and_then(|v| v.as_i64()).unwrap_or(-1);
    if rc != 0 {
        return Err(format!("东财分时 rc={}", rc));
    }
    let data = body
        .get("data")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "东财分时 data 缺失".to_string())?;
    let pre_close = json_f64(data, "preClose")
        .or_else(|| json_f64(data, "f60"))
        .unwrap_or(0.0);
    let trends = data
        .get("trends")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut points = Vec::new();
    for t in trends {
        let Some(s) = t.as_str() else {
            continue;
        };
        if let Some(p) = parse_trend_line_simple(s) {
            points.push(p);
        }
    }
    let auction = summarize_intraday_auction(&points, pre_close);
    Ok(IntradaySeries {
        pre_close,
        points,
        auction,
    })
}

/// K 线逗号行：`日期,开,收,高,低,成交量,成交额,振幅,涨跌幅,涨跌额,换手率`
fn parse_kline_csv_row(s: &str) -> Option<KlinePoint> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() < 6 {
        return None;
    }
    let ts = parse_kline_day_ts(parts[0])?;
    let open = parts[1].parse::<f64>().ok()?;
    let close = parts[2].parse::<f64>().ok()?;
    let high = parts[3].parse::<f64>().ok()?;
    let low = parts[4].parse::<f64>().ok()?;
    let volume = parts[5].parse::<u64>().ok().unwrap_or(0);
    let turnover = parts
        .get(6)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|x| x.is_finite())
        .unwrap_or(0.0);
    let turnover_rate = parts
        .get(10)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|x| x.is_finite())
        .unwrap_or(0.0);
    if !open.is_finite() {
        return None;
    }
    Some(KlinePoint {
        time: ts,
        open,
        high,
        low,
        close,
        volume,
        turnover,
        turnover_rate,
    })
}

fn parse_em_kline_json(body: &Value) -> Result<KlineSeries, String> {
    let rc = body.get("rc").and_then(|v| v.as_i64()).unwrap_or(-1);
    if rc != 0 {
        return Err(format!("东财K线 rc={}", rc));
    }
    let data = body
        .get("data")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "东财K线 data 缺失".to_string())?;
    let klines = data
        .get("klines")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut points = Vec::new();
    for row in klines {
        let Some(s) = row.as_str() else {
            continue;
        };
        if let Some(p) = parse_kline_csv_row(s) {
            points.push(p);
        }
    }
    Ok(KlineSeries { points })
}

fn build_order_book_from_map(data: &serde_json::Map<String, Value>) -> OrderBook {
    let mut asks: Vec<OrderBookLevel> = Vec::new();
    for i in (1..=5).rev() {
        let pk = format!("f{}", 30 + i);
        let vk = format!("f{}", 35 + i);
        let p = json_f64(data, &pk).unwrap_or(0.0);
        let v = json_u64(data, &vk);
        if p > 1e-9 && p.is_finite() {
            asks.push(OrderBookLevel { price: p, volume: v });
        }
    }
    let mut bids: Vec<OrderBookLevel> = Vec::new();
    for i in 1..=5 {
        let pk = format!("f{}", 18 + i);
        let vk = format!("f{}", 23 + i);
        let p = json_f64(data, &pk).unwrap_or(0.0);
        let v = json_u64(data, &vk);
        if p > 1e-9 && p.is_finite() {
            bids.push(OrderBookLevel { price: p, volume: v });
        }
    }
    let price = json_f64(data, "f43")
        .filter(|p| p.is_finite() && *p > 0.0)
        .or_else(|| json_f64(data, "f46"))
        .unwrap_or(0.0);
    let bid1 = json_f64(data, "f19")
        .filter(|p| p.is_finite() && *p > 0.0)
        .or_else(|| json_f64(data, "f71").filter(|p| p.is_finite() && *p > 0.0))
        .unwrap_or(0.0);
    if bids.is_empty() && bid1 > 1e-9 {
        let v1 = json_u64(data, "f24");
        bids.push(OrderBookLevel {
            price: bid1,
            volume: v1,
        });
    }
    if asks.is_empty() && price > 1e-9 && bid1 > 1e-9 {
        let ask1 = if price >= bid1 {
            price
        } else {
            (bid1 + 0.01).min(price + 0.05).max(price)
        };
        if ask1 > 1e-9 {
            let av = json_u64(data, "f36");
            asks.push(OrderBookLevel {
                price: ask1,
                volume: av,
            });
        }
    } else if asks.is_empty() && price > 1e-9 {
        asks.push(OrderBookLevel {
            price: (price + 0.01).max(0.01),
            volume: json_u64(data, "f36"),
        });
    }
    let max_levels = asks.len().max(bids.len()) as u32;
    OrderBook {
        asks,
        bids,
        max_levels: max_levels.max(1),
    }
}

// --- 腾讯 qt.gtimg.cn（GBK）：列表买一卖一/五档与常见券商 App 一致 ---

fn tencent_symbol_for_internal(internal_lower: &str) -> Option<String> {
    let c = internal_lower.trim().to_lowercase();
    if c.len() < 3 {
        return None;
    }
    let (pfx, rest) = c.split_at(2);
    if !matches!(pfx, "sh" | "sz" | "bj") {
        return None;
    }
    if rest.chars().all(|ch| ch.is_ascii_digit()) {
        Some(format!("{pfx}{rest}"))
    } else {
        None
    }
}

fn parse_gtimg_var_segment(seg: &str) -> Option<(String, Vec<&str>)> {
    let rest = seg.strip_prefix("v_")?;
    let eq = rest.find("=\"")?;
    let sym = rest[..eq].to_lowercase();
    let payload = &rest[eq + 2..];
    let end = payload.rfind('"')?;
    let inner = &payload[..end];
    let parts: Vec<&str> = inner.split('~').collect();
    if parts.len() < 36 {
        return None;
    }
    Some((sym, parts))
}

fn quote_row_from_gtimg_parts(sym_lower: &str, parts: &[&str]) -> Option<QuoteRow> {
    let price = parts.get(3)?.parse::<f64>().ok().filter(|p| *p > 0.0)?;
    let name = parts.get(1).unwrap_or(&"").trim().to_string();
    let change_pct = parts
        .get(33)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let prev_close = parts
        .get(4)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|p| *p > 0.0)
        .unwrap_or_else(|| {
            if change_pct.abs() > 1e-9 && price > 1e-9 {
                price / (1.0 + change_pct / 100.0)
            } else {
                price
            }
        });
    let open = parts
        .get(5)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|o| *o > 0.0)
        .unwrap_or(prev_close);
    let volume = parts
        .get(6)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let turnover = parts
        .get(36)
        .and_then(|s| s.split('/').nth(2))
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|x| x.is_finite())
        .unwrap_or(0.0);
    let high = parts
        .get(34)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|h| *h > 0.0)
        .unwrap_or(price);
    let low = parts
        .get(35)
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|l| *l > 0.0)
        .unwrap_or(price);
    let bid1 = parts.get(9).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let ask1 = parts
        .get(19)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let bid1_vol = parts
        .get(10)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let ask1_vol = parts
        .get(20)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    Some(QuoteRow {
        code: sym_lower.to_string(),
        name,
        change_pct,
        price,
        prev_close,
        open,
        volume,
        turnover,
        turnover_rate: 0.0,
        commission_ratio: 0.0,
        total_pl: 0.0,
        daily_pl: 0.0,
        high,
        low,
        bid1,
        ask1,
        bid1_vol,
        ask1_vol,
        volume_ratio: None,
        sector: None,
        sector_change_pct: None,
        industry_board_code: None,
    })
}

fn order_book_from_gtimg_parts(parts: &[&str]) -> Option<OrderBook> {
    if parts.len() < 20 {
        return None;
    }
    let mut bids: Vec<OrderBookLevel> = Vec::with_capacity(5);
    for level in 0..5 {
        let pi = 9 + level * 2;
        let vi = 10 + level * 2;
        let Some(ps) = parts.get(pi).copied() else {
            break;
        };
        let Ok(p) = ps.parse::<f64>() else {
            continue;
        };
        if p > 1e-9 && p.is_finite() {
            let v = parts
                .get(vi)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            bids.push(OrderBookLevel { price: p, volume: v });
        }
    }
    let mut asks: Vec<OrderBookLevel> = Vec::with_capacity(5);
    for level in (0..5).rev() {
        let pi = 19 + level * 2;
        let vi = 20 + level * 2;
        let Some(ps) = parts.get(pi).copied() else {
            continue;
        };
        let Ok(p) = ps.parse::<f64>() else {
            continue;
        };
        if p > 1e-9 && p.is_finite() {
            let v = parts
                .get(vi)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            asks.push(OrderBookLevel { price: p, volume: v });
        }
    }
    if asks.is_empty() && bids.is_empty() {
        return None;
    }
    let max_levels = asks.len().max(bids.len()).max(1) as u32;
    Some(OrderBook {
        asks,
        bids,
        max_levels,
    })
}

async fn merge_gtimg_response_into_map(
    client: &reqwest::Client,
    symbols_csv: &str,
    out: &mut std::collections::HashMap<String, QuoteRow>,
) -> Result<(), String> {
    let url = format!("https://qt.gtimg.cn/q={symbols_csv}");
    let res = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    let (text, _, _) = encoding_rs::GBK.decode(&bytes);
    let text = text.into_owned();
    for seg in text.split(';') {
        let s = seg.trim();
        if s.is_empty() {
            continue;
        }
        let Some((sym, parts)) = parse_gtimg_var_segment(s) else {
            continue;
        };
        if let Some(row) = quote_row_from_gtimg_parts(&sym, &parts) {
            out.insert(sym, row);
        }
    }
    Ok(())
}

async fn fetch_quotes_tencent_gtimg(
    client: &reqwest::Client,
    codes: &[String],
) -> Result<std::collections::HashMap<String, QuoteRow>, String> {
    const CHUNK: usize = 40;
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for c in codes {
        let internal = c.trim().to_lowercase();
        if !seen.insert(internal.clone()) {
            continue;
        }
        if let Some(sym) = tencent_symbol_for_internal(&internal) {
            pairs.push((internal, sym));
        }
    }
    let mut out: std::collections::HashMap<String, QuoteRow> =
        std::collections::HashMap::new();
    for chunk in pairs.chunks(CHUNK) {
        let csv = chunk
            .iter()
            .map(|(_, s)| s.as_str())
            .collect::<Vec<_>>()
            .join(",");
        if csv.is_empty() {
            continue;
        }
        merge_gtimg_response_into_map(client, &csv, &mut out).await?;
    }
    Ok(out)
}

pub async fn fetch_quotes_tencent(codes: &[String]) -> Result<Vec<QuoteRow>, String> {
    if codes.is_empty() {
        return Ok(vec![]);
    }
    let client = em_client()?;
    let t_map = fetch_quotes_tencent_gtimg(&client, codes).await?;
    let mock_rows = mock_for_codes(codes);
    let mut out = Vec::with_capacity(codes.len());
    for (i, c) in codes.iter().enumerate() {
        let k = c.trim().to_lowercase();
        let t = t_map
            .get(&k)
            .cloned()
            .unwrap_or_else(|| error_row(c, "无数据"));
        let mock = mock_rows.get(i).cloned().unwrap_or_else(|| error_row(c, ""));
        out.push(merge_em_row_with_mock(t, &mock));
    }
    Ok(out)
}

async fn fetch_order_book_tencent_gtimg(client: &reqwest::Client, internal_code: &str) -> OrderBook {
    let empty = || OrderBook {
        asks: vec![],
        bids: vec![],
        max_levels: 1,
    };
    let Some(sym) = tencent_symbol_for_internal(internal_code) else {
        return empty();
    };
    let url = format!("https://qt.gtimg.cn/q={sym}");
    let Ok(res) = client.get(&url).send().await else {
        return empty();
    };
    let Ok(bytes) = res.bytes().await else {
        return empty();
    };
    let (text, _, _) = encoding_rs::GBK.decode(&bytes);
    let text = text.into_owned();
    let key = internal_code.trim().to_lowercase();
    for seg in text.split(';') {
        let s = seg.trim();
        let Some((s2, parts)) = parse_gtimg_var_segment(s) else {
            continue;
        };
        if s2 == key {
            return order_book_from_gtimg_parts(&parts).unwrap_or_else(empty);
        }
    }
    empty()
}

async fn fetch_intraday_eastmoney(client: &reqwest::Client, internal_code: &str) -> IntradaySeries {
    let Some(secid) = code_to_secid(internal_code) else {
        return IntradaySeries {
            pre_close: 0.0,
            points: vec![],
            auction: None,
        };
    };
    // iscr=1：含 9:15–9:25 竞价轨迹；fields1 扩到 f20 以带上 tradePeriods 等
    let url_today = format!(
        "http://push2his.eastmoney.com/api/qt/stock/trends2/get?secid={}&fields1=f1,f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12,f13,f14,f15,f16,f17,f18,f19,f20&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61&iscr=1&ndays=1",
        urlencoding::encode(&secid)
    );
    let text = match client.get(&url_today).send().await {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => {
                return IntradaySeries {
                    pre_close: 0.0,
                    points: vec![],
                    auction: None,
                };
            }
        },
        Err(_) => {
            return IntradaySeries {
                pre_close: 0.0,
                points: vec![],
                auction: None,
            };
        }
    };
    let v: Value = match serde_json::from_str(&text) {
        Ok(j) => j,
        Err(_) => {
            return IntradaySeries {
                pre_close: 0.0,
                points: vec![],
                auction: None,
            };
        }
    };
    let mut series = parse_em_intraday_json(&v).unwrap_or(IntradaySeries {
        pre_close: 0.0,
        points: vec![],
        auction: None,
    });
    if let Some(ref mut a) = series.auction {
        let url_2d = format!(
            "http://push2his.eastmoney.com/api/qt/stock/trends2/get?secid={}&fields1=f1,f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12,f13,f14,f15,f16,f17,f18,f19,f20&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61&iscr=1&ndays=2",
            urlencoding::encode(&secid)
        );
        if let Ok(r2) = client.get(&url_2d).send().await {
            if let Ok(t2) = r2.text().await {
                if let Ok(v2) = serde_json::from_str::<Value>(&t2) {
                    a.prev_ref_turnover = prev_trading_day_first_bar_turnover(&v2);
                }
            }
        }
    }
    series
}

fn klt_for_period(period: &str) -> &'static str {
    match period.trim().to_lowercase().as_str() {
        "week" | "102" | "wk" | "周k" => "102",
        "month" | "103" | "月k" => "103",
        _ => "101",
    }
}

fn parse_u64_loose(s: &str) -> Option<u64> {
    let t = s.trim().replace(',', "");
    if t.is_empty() {
        return None;
    }
    t.parse().ok()
}

/// 东财五档常为价有量无（尤其涨停封单）；用腾讯 `gtimg` 同价位补量（手）
fn merge_order_book_volumes_from_gtimg(book: &mut OrderBook, parts: &[String]) {
    let nb = book.bids.len().min(5);
    for j in 0..nb {
        if book.bids[j].volume > 0 {
            continue;
        }
        let vi = 10 + j * 2;
        if let Some(s) = parts.get(vi) {
            if let Some(v) = parse_u64_loose(s) {
                if v > 0 {
                    book.bids[j].volume = v;
                }
            }
        }
    }
    let na = book.asks.len().min(5);
    for j in 0..na {
        if book.asks[j].volume > 0 {
            continue;
        }
        let k = na - j;
        if !(1..=5).contains(&k) {
            continue;
        }
        let vi = 20 + (k - 1) * 2;
        if let Some(s) = parts.get(vi) {
            if let Some(v) = parse_u64_loose(s) {
                if v > 0 {
                    book.asks[j].volume = v;
                }
            }
        }
    }
}

async fn fetch_gtimg_tilde_parts(
    client: &reqwest::Client,
    internal_code: &str,
) -> Option<Vec<String>> {
    let sym = tencent_symbol_for_internal(internal_code)?;
    let url = format!("https://qt.gtimg.cn/q={sym}");
    let res = client.get(&url).send().await.ok()?;
    let bytes = res.bytes().await.ok()?;
    let (cow, _, _) = encoding_rs::GBK.decode(&bytes);
    let text = cow.into_owned();
    let key = internal_code.trim().to_lowercase();
    for seg in text.split(';') {
        let s = seg.trim();
        let Some((s2, parts)) = parse_gtimg_var_segment(s) else {
            continue;
        };
        if s2 == key {
            return Some(parts.iter().map(|x| (*x).to_string()).collect());
        }
    }
    None
}

async fn fetch_kline_eastmoney(
    client: &reqwest::Client,
    internal_code: &str,
    period: &str,
) -> KlineSeries {
    let Some(secid) = code_to_secid(internal_code) else {
        return KlineSeries { points: vec![] };
    };
    let klt = klt_for_period(period);
    let url = format!(
        "http://push2his.eastmoney.com/api/qt/stock/kline/get?secid={}&fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61&klt={}&fqt=1&lmt=180&end=20500101",
        urlencoding::encode(&secid),
        klt
    );
    let text = match client.get(&url).send().await {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => return KlineSeries { points: vec![] },
        },
        Err(_) => return KlineSeries { points: vec![] },
    };
    let v: Value = match serde_json::from_str(&text) {
        Ok(j) => j,
        Err(_) => return KlineSeries { points: vec![] },
    };
    parse_em_kline_json(&v).unwrap_or(KlineSeries { points: vec![] })
}

async fn fetch_order_book_eastmoney(client: &reqwest::Client, internal_code: &str) -> OrderBook {
    let Some(secid) = code_to_secid(internal_code) else {
        return OrderBook {
            asks: vec![],
            bids: vec![],
            max_levels: 1,
        };
    };
    let url = format!(
        "http://push2.eastmoney.com/api/qt/stock/get?invt=2&fltt=2&secid={}&fields={}",
        urlencoding::encode(&secid),
        ORDER_BOOK_FIELDS
    );
    let text = match client.get(&url).send().await {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => {
                return OrderBook {
                    asks: vec![],
                    bids: vec![],
                    max_levels: 1,
                };
            }
        },
        Err(_) => {
            return OrderBook {
                asks: vec![],
                bids: vec![],
                max_levels: 1,
            };
        }
    };
    let v: Value = match serde_json::from_str(&text) {
        Ok(j) => j,
        Err(_) => {
            return OrderBook {
                asks: vec![],
                bids: vec![],
                max_levels: 1,
            };
        }
    };
    let data = match v.get("data").and_then(|d| d.as_object()) {
        Some(d) => d,
        None => {
            return OrderBook {
                asks: vec![],
                bids: vec![],
                max_levels: 1,
            };
        }
    };
    let mut book = build_order_book_from_map(data);
    let vol_sum: u64 = book
        .asks
        .iter()
        .map(|x| x.volume)
        .sum::<u64>()
        .saturating_add(book.bids.iter().map(|x| x.volume).sum());
    let has_levels = !book.asks.is_empty() || !book.bids.is_empty();
    if has_levels && vol_sum == 0 {
        if let Some(parts) = fetch_gtimg_tilde_parts(client, internal_code).await {
            merge_order_book_volumes_from_gtimg(&mut book, &parts);
        }
    }
    book
}

fn mock_intraday_for_code(code: &str) -> IntradaySeries {
    let rows = mock_for_codes(&[code.to_string()]);
    let base = rows.first().cloned().unwrap_or_else(|| error_row(code, ""));
    let pre = base.prev_close;
    let mut rng = rand::thread_rng();
    let am_start = parse_shanghai_minute_ts("2026-03-20 09:30").unwrap_or(0);
    let pm_start = parse_shanghai_minute_ts("2026-03-20 13:00").unwrap_or(0);
    let mut points = Vec::new();
    let mut sum_turn = 0.0_f64;
    let mut sum_lots = 0u64;
    
    let mut current_price = base.open;
    if current_price < 1e-9 {
        current_price = pre;
    }

    // 上午 9:30–11:30、下午 13:00–15:00（各 121 根分钟），中间无数据 → 前端插入午休断点
    for i in 0..121 {
        let ts = am_start + i as i64 * 60;
        let jitter: f64 = rng.gen_range(-0.002..0.002);
        current_price = current_price * (1.0 + jitter);
        let close = current_price;
        let vol = rng.gen_range(10_000u64..200_000);
        let turnover: f64 = rng.gen_range(5e6..8e7);
        sum_turn += turnover;
        sum_lots += vol;
        let avg_price = if sum_lots > 0 && sum_turn > 0.0 {
            sum_turn / (sum_lots as f64 * 100.0)
        } else {
            close
        };
        points.push(IntradayPoint {
            time: ts,
            open: close - 0.01,
            close,
            high: close + 0.02,
            low: close - 0.02,
            volume: vol,
            turnover,
            avg_price,
        });
    }
    for i in 0..121 {
        let ts = pm_start + i as i64 * 60;
        let jitter: f64 = rng.gen_range(-0.002..0.002);
        current_price = current_price * (1.0 + jitter);
        let close = current_price;
        let vol = rng.gen_range(10_000u64..200_000);
        let turnover: f64 = rng.gen_range(5e6..8e7);
        sum_turn += turnover;
        sum_lots += vol;
        let avg_price = if sum_lots > 0 && sum_turn > 0.0 {
            sum_turn / (sum_lots as f64 * 100.0)
        } else {
            close
        };
        points.push(IntradayPoint {
            time: ts,
            open: close - 0.01,
            close,
            high: close + 0.02,
            low: close - 0.02,
            volume: vol,
            turnover,
            avg_price,
        });
    }
    let first = points.first().cloned();
    let auction = first.map(|p| {
        let pct = if pre > 1e-9 && pre.is_finite() {
            (p.close - pre) / pre * 100.0
        } else {
            0.0
        };
        IntradayAuctionSummary {
            match_price: p.close,
            match_volume_lots: p.volume,
            match_turnover: p.turnover,
            pct_vs_pre_close: pct,
            prev_ref_turnover: Some(p.turnover * 0.88),
        }
    });
    IntradaySeries {
        pre_close: pre,
        points,
        auction,
    }
}

fn mock_kline_for_code(code: &str, period: &str) -> KlineSeries {
    let rows = mock_for_codes(&[code.to_string()]);
    let base = rows.first().cloned().unwrap_or_else(|| error_row(code, ""));
    let mut rng = rand::thread_rng();
    let n = match klt_for_period(period) {
        "103" => 36,
        "102" => 52,
        _ => 90,
    };
    let mut price = base.prev_close.max(0.01);
    let mut points = Vec::new();
    for i in 0..n {
        let day = NaiveDate::from_ymd_opt(2025, 12, 1)
            .unwrap()
            .checked_add_signed(chrono::Duration::days(i as i64))
            .unwrap();
        let ts = parse_kline_day_ts(&day.format("%Y-%m-%d").to_string()).unwrap_or(0);
        let chg: f64 = rng.gen_range(-0.03..0.03);
        let open = price;
        let close = (open * (1.0 + chg)).max(0.01);
        let high = open.max(close) * 1.01;
        let low = open.min(close) * 0.99;
        let vol = rng.gen_range(100_000u64..9_000_000);
        let turnover = close * vol as f64 * rng.gen_range(80.0..120.0);
        let turnover_rate = rng.gen_range(0.8..12.0f64);
        points.push(KlinePoint {
            time: ts,
            open,
            high,
            low,
            close,
            volume: vol,
            turnover,
            turnover_rate,
        });
        price = close;
    }
    KlineSeries { points }
}

fn mock_order_book_for_code(code: &str) -> OrderBook {
    let rows = mock_for_codes(&[code.to_string()]);
    let base = rows.first().cloned().unwrap_or_else(|| error_row(code, ""));
    let p = base.price.max(0.01);
    let tick = 0.01f64;
    let mut bids = Vec::new();
    let mut asks = Vec::new();
    for i in 1..=5 {
        let bp = (p - tick * i as f64).max(0.01);
        bids.push(OrderBookLevel {
            price: bp,
            volume: 10000 * (6 - i) as u64,
        });
    }
    for i in (1..=5).rev() {
        let ap = p + tick * i as f64;
        asks.push(OrderBookLevel {
            price: ap,
            volume: 8000 * (6 - i) as u64,
        });
    }
    OrderBook {
        max_levels: 5,
        bids,
        asks,
    }
}

// --- 底部市场信息条：主要股指 + 涨跌家数 + 两市成交额（东财 push2 / push2his）---

/// 主要指数一项（底栏盯盘：前端按涨跌/波动筛选展示）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RibbonIndexItem {
    /// 稳定键，如 `sh000001`、`glob_n225`，供前端做「久无波动则弱化」
    pub id: String,
    pub name: String,
    pub change_pct: f64,
}

/// 行业板块一项（市场快览浮层领涨/领跌）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSectorBrief {
    pub name: String,
    pub change_pct: f64,
    /// 行业 `clist` 的 `f12`（`BKxxxx`），仅构建映射用
    #[serde(skip)]
    pub board_code: Option<String>,
}

/// 与 `get_market_ribbon` 前端一致
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketRibbonSnapshot {
    pub indices: Vec<RibbonIndexItem>,
    pub up_count: u32,
    pub down_count: u32,
    /// 沪深两市今日成交额之和（元，来自指数快照 `turnover`）
    pub turnover_today: f64,
    /// 上一交易日沪深两市成交额之和（元，来自日 K 倒数第二根 `成交额` 列）
    pub turnover_yesterday: f64,
    /// 沪深行业板块涨跌幅前六（东财 `clist` `m:90+t:2`）
    pub sector_gainers: Vec<MarketSectorBrief>,
    /// 沪深行业板块涨跌幅后六（跌幅最大）
    pub sector_losers: Vec<MarketSectorBrief>,
}

fn json_u64_field(obj: &serde_json::Map<String, Value>, key: &str) -> u64 {
    obj.get(key)
        .and_then(|v| {
            if let Some(n) = v.as_u64() {
                return Some(n);
            }
            if let Some(n) = v.as_i64() {
                return Some(n as u64);
            }
            if let Some(n) = v.as_f64() {
                return Some(n as u64);
            }
            v.as_str()?.parse().ok()
        })
        .unwrap_or(0)
}

/// 日 K 行：`日期,开,收,高,低,成交量,成交额,…`（与东财网页一致）
fn parse_kline_line_amount_yuan(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() > 6 {
        parts[6].parse::<f64>().ok().filter(|a| a.is_finite())
    } else {
        None
    }
}

async fn fetch_index_yesterday_turnover_from_kline(
    client: &reqwest::Client,
    internal_code: &str,
) -> f64 {
    let Some(secid) = code_to_secid(internal_code) else {
        return 0.0;
    };
    let url = format!(
        "http://push2his.eastmoney.com/api/qt/stock/kline/get?secid={}&fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61&klt=101&fqt=1&lmt=8&end=20500101",
        urlencoding::encode(&secid)
    );
    let text = match client.get(&url).send().await {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => return 0.0,
        },
        Err(_) => return 0.0,
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return 0.0;
    };
    let Some(arr) = v.pointer("/data/klines").and_then(|x| x.as_array()) else {
        return 0.0;
    };
    if arr.len() < 2 {
        return arr
            .last()
            .and_then(|x| x.as_str())
            .and_then(parse_kline_line_amount_yuan)
            .unwrap_or(0.0);
    }
    let idx = arr.len().saturating_sub(2);
    arr.get(idx)
        .and_then(|x| x.as_str())
        .and_then(parse_kline_line_amount_yuan)
        .unwrap_or(0.0)
}

/// 东财 `qt/stock/get` 里指数 `f104`/`f105` 长期为 `"-"`，解析后恒为 0；改用与列表同源的 `ulist.np` 批量拉上证/深证成指。
const MARKET_BREADTH_ULIST_URL: &str = "https://push2.eastmoney.com/api/qt/ulist.np/get?fltt=2&secids=1.000001,0.399001&fields=f104,f105";

async fn fetch_market_breadth_up_down(client: &reqwest::Client) -> (u32, u32) {
    let text = match client.get(MARKET_BREADTH_ULIST_URL).send().await {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => return fetch_market_breadth_up_down_stock_get(client).await,
        },
        Err(_) => return fetch_market_breadth_up_down_stock_get(client).await,
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return fetch_market_breadth_up_down_stock_get(client).await;
    };
    let Some(diff) = v.pointer("/data/diff").and_then(|d| d.as_array()) else {
        return fetch_market_breadth_up_down_stock_get(client).await;
    };
    let mut up: u64 = 0;
    let mut down: u64 = 0;
    for item in diff {
        let Some(obj) = item.as_object() else {
            continue;
        };
        up += json_u64_field(obj, "f104");
        down += json_u64_field(obj, "f105");
    }
    if diff.is_empty() {
        return fetch_market_breadth_up_down_stock_get(client).await;
    }
    (up.min(u32::MAX as u64) as u32, down.min(u32::MAX as u64) as u32)
}

/// 回退：`stock/get`（多数时候 `f104`/`f105` 为 `"-"`，仅在网络或 ulist 异常时兜底）
async fn fetch_market_breadth_up_down_stock_get(client: &reqwest::Client) -> (u32, u32) {
    let mut up: u64 = 0;
    let mut down: u64 = 0;
    for secid in ["1.000001", "0.399001"] {
        let url = format!(
            "http://push2.eastmoney.com/api/qt/stock/get?invt=2&fltt=2&secid={}&fields=f104,f105",
            secid
        );
        let text = match client.get(&url).send().await {
            Ok(r) => match r.text().await {
                Ok(t) => t,
                Err(_) => continue,
            },
            Err(_) => continue,
        };
        let Ok(v) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(data) = v.get("data").and_then(|d| d.as_object()) else {
            continue;
        };
        up += json_u64_field(data, "f104");
        down += json_u64_field(data, "f105");
    }
    (up.min(u32::MAX as u64) as u32, down.min(u32::MAX as u64) as u32)
}

/// 东财行业板块 `fs`：**必须用 `+` 连接**（`urlencoding` 会编成 `%2B`）。若写成空格再编码为 `%20`，服务端会扩成数万只标的（杠杆 ETF 等），`f3` 会出现 500% 级假数据。
const EM_INDUSTRY_BOARD_FS: &str = "m:90+t:2+f:!50";

/// 东财 `clist` 多省分流域名（AKShare 等常用 `17.push2`；根域名在部分网络下 `clist` 会空数据）
const EM_PUSH2_CLIST_HOSTS: &[&str] = &[
    "https://17.push2.eastmoney.com",
    "https://82.push2.eastmoney.com",
    "https://push2.eastmoney.com",
    "http://17.push2.eastmoney.com",
    "http://82.push2.eastmoney.com",
    "http://push2.eastmoney.com",
];

fn parse_industry_board_clist_diff(text: &str) -> Vec<MarketSectorBrief> {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return vec![];
    };
    // 东财 clist 接口有时 rc=0 但 diff 仍为空（网络/域名级问题），也可能在非 0 时仍有有效数据；
    // 不以 rc 为唯一过滤条件，让 diff 解析说话。
    let Some(diff) = v.pointer("/data/diff").and_then(|d| d.as_array()) else {
        return vec![];
    };
    let mut rows: Vec<MarketSectorBrief> = Vec::new();
    for item in diff {
        let Some(obj) = item.as_object() else {
            continue;
        };
        let Some(name) = em_nonempty_string_field(obj, "f14") else {
            continue;
        };
        let change_pct = json_f64(obj, "f3").unwrap_or(0.0);
        if !change_pct.is_finite() {
            continue;
        }
        let board_code = em_board_code_f12_clist(obj);
        rows.push(MarketSectorBrief {
            name,
            change_pct,
            board_code,
        });
    }
    rows
}

/// 东财 `clist`：`po=1` 按 `f3` 降序（涨幅榜），`po=0` 升序（跌幅榜）。领跌不可只在 `po=1` 的一页里取最小值，否则跌幅最大者可能不在该页。
async fn fetch_industry_board_clist_page(client: &reqwest::Client, po: u8) -> String {
    let fs = urlencoding::encode(EM_INDUSTRY_BOARD_FS);
    let qs = format!(
        "pn=1&pz=50&po={}&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&fltt=2&invt=2&fid=f3&fields=f14,f3&fs={}",
        po, fs
    );
    let mut last_err = String::new();
    for host in EM_PUSH2_CLIST_HOSTS {
        let url = format!("{}/api/qt/clist/get?{}", host, qs);
        let Ok(res) = client.get(&url).send().await else {
            continue;
        };
        let Ok(text) = res.text().await else {
            continue;
        };
        // 只要 diff 解析有结果就用（rc 非 0 情况下也可能有数据）
        if !text.trim().is_empty() && parse_industry_board_clist_diff(&text).is_empty() {
            // 解析为空，记录错误但不立即返回，继续试其他域名
            last_err = text.chars().take(120).collect();
            continue;
        }
        if !parse_industry_board_clist_diff(&text).is_empty() {
            return text;
        }
    }
    // 所有域名都失败，返回最后一个收到的响应（可能含部分数据或错误信息）
    last_err
}

/// 东财 `clist` 沪深行业板块，用于市场快览领涨/领跌（与网页排序一致：直接取接口顺序前 6）
async fn fetch_industry_sector_leaders(
    client: &reqwest::Client,
) -> (Vec<MarketSectorBrief>, Vec<MarketSectorBrief>) {
    let (text_desc, text_asc) =
        futures::join!(fetch_industry_board_clist_page(client, 1), fetch_industry_board_clist_page(client, 0));
    let gainers: Vec<MarketSectorBrief> = parse_industry_board_clist_diff(&text_desc)
        .into_iter()
        .take(6)
        .collect();
    let losers: Vec<MarketSectorBrief> = parse_industry_board_clist_diff(&text_asc)
        .into_iter()
        .take(6)
        .collect();
    (gainers, losers)
}

fn sector_name_normalize(s: &str) -> String {
    s.trim()
        .replace('\u{3000}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// 东财行业板块名常见后缀（L2 相对个股 f100 可能多「Ⅱ」等）
fn strip_board_tier_suffix(s: &str) -> String {
    let t = s.trim();
    for suf in ["Ⅲ", "III", "Ⅱ", "II", "Ⅰ", "I"] {
        if let Some(p) = t.strip_suffix(suf) {
            return p
                .trim_end_matches(|c: char| c == ' ' || c == '\u{3000}')
                .to_string();
        }
    }
    t.to_string()
}

fn board_rest_is_tier_suffix_only(rest: &str) -> bool {
    let t = rest
        .trim_matches(|c: char| c == ' ' || c == '\u{3000}')
        .trim();
    t.is_empty() || ["Ⅱ", "II", "Ⅲ", "III", "Ⅰ", "I"].iter().any(|s| t == *s)
}

fn attach_sector_change_from_map(row: &mut QuoteRow, map: &HashMap<String, f64>) {
    if let Some(ref bk) = row.industry_board_code {
        if let Some(&pct) = map.get(bk) {
            row.sector_change_pct = Some(pct);
            return;
        }
    }
    let Some(ref sec) = row.sector else {
        return;
    };
    let s = sector_name_normalize(sec);
    if s.is_empty() {
        return;
    }
    if let Some(&pct) = map.get(&s) {
        row.sector_change_pct = Some(pct);
        return;
    }
    let s_short = strip_board_tier_suffix(&s);
    if !s_short.is_empty() && s_short != s {
        if let Some(&pct) = map.get(&s_short) {
            row.sector_change_pct = Some(pct);
            return;
        }
    }
    for (k, &pct) in map.iter() {
        if k.len() >= s.len() && k.starts_with(&s) && board_rest_is_tier_suffix_only(&k[s.len()..]) {
            row.sector_change_pct = Some(pct);
            return;
        }
    }
    for (k, &pct) in map.iter() {
        if s.len() >= k.len() && s.starts_with(k) && board_rest_is_tier_suffix_only(&s[k.len()..]) {
            row.sector_change_pct = Some(pct);
            return;
        }
    }
}

/// 沪深行业板块 `clist` 全量分页 → 板块名 / `BK` 代码 → 当日涨跌幅（%）
async fn fetch_industry_board_name_to_change(client: &reqwest::Client) -> HashMap<String, f64> {
    const CLIST_PZ: u32 = 200;
    let fs = urlencoding::encode(EM_INDUSTRY_BOARD_FS);
    for host in EM_PUSH2_CLIST_HOSTS {
        let mut map: HashMap<String, f64> = HashMap::new();
        let mut got_page = false;
        for pn in 1u32..=30u32 {
            let url = format!(
                "{}/api/qt/clist/get?pn={}&pz={}&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&fltt=2&invt=2&fid=f3&fields=f12,f14,f3&fs={}",
                host, pn, CLIST_PZ, fs
            );
            let Ok(res) = client.get(&url).send().await else {
                break;
            };
            let Ok(text) = res.text().await else {
                break;
            };
            let rows = parse_industry_board_clist_diff(&text);
            if rows.is_empty() {
                if pn == 1 && !got_page {
                    break;
                }
                break;
            }
            got_page = true;
            let page_n = rows.len();
            for r in rows {
                let k = sector_name_normalize(&r.name);
                if !k.is_empty() {
                    map.insert(k.clone(), r.change_pct);
                    let short = strip_board_tier_suffix(&k);
                    if !short.is_empty() && short != k {
                        map.entry(short).or_insert(r.change_pct);
                    }
                }
                if let Some(ref bk) = r.board_code {
                    map.insert(bk.clone(), r.change_pct);
                }
            }
            if page_n < CLIST_PZ as usize {
                break;
            }
        }
        if !map.is_empty() {
            return map;
        }
    }
    HashMap::new()
}

/// 底部条：A 股核心宽基 + 港股（超短操盘手最需要盯的 7 个）
const RIBBON_MAJOR_INDICES: &[(&str, &str)] = &[
    ("sh000001", "上证"),
    ("sz399001", "深证"),
    ("sh000300", "沪深300"),
    ("sz399006", "创业板"),
    ("sh000688", "科创50"),
    ("hkhsi", "恒生"),
    ("fsa50", "富时A50"),
];

async fn fetch_ribbon_major_indices(client: &reqwest::Client) -> Vec<RibbonIndexItem> {
    // 全部走 ulist 批量（A 股 + 港股均可）
    let codes: Vec<String> = RIBBON_MAJOR_INDICES
        .iter()
        .map(|(c, _)| (*c).to_string())
        .collect();

    let map = fetch_quotes_eastmoney_ulist(client, &codes)
        .await
        .unwrap_or_default();

    let mut out = Vec::with_capacity(RIBBON_MAJOR_INDICES.len());
    for (code, label) in RIBBON_MAJOR_INDICES.iter().copied() {
        let k = code.to_lowercase();
        let change_pct = if let Some(q) = map.get(&k) {
            q.change_pct
        } else if k == "hkhsi" {
            fetch_hk_hsi_em(client).await.change_pct
        } else {
            fetch_one_em(client, code).await.change_pct
        };
        out.push(RibbonIndexItem {
            id: k,
            name: label.to_string(),
            change_pct,
        });
    }
    out
}

fn mock_market_ribbon() -> MarketRibbonSnapshot {
    MarketRibbonSnapshot {
        indices: vec![
            RibbonIndexItem {
                id: "sh000001".into(),
                name: "上证".into(),
                change_pct: 0.35,
            },
            RibbonIndexItem {
                id: "sz399001".into(),
                name: "深证".into(),
                change_pct: 0.52,
            },
            RibbonIndexItem {
                id: "sh000300".into(),
                name: "沪深300".into(),
                change_pct: 0.28,
            },
            RibbonIndexItem {
                id: "sz399006".into(),
                name: "创业板".into(),
                change_pct: 1.12,
            },
            RibbonIndexItem {
                id: "sh000688".into(),
                name: "科创50".into(),
                change_pct: -0.82,
            },
            RibbonIndexItem {
                id: "hkhsi".into(),
                name: "恒生".into(),
                change_pct: -0.21,
            },
            RibbonIndexItem {
                id: "fsa50".into(),
                name: "富时A50".into(),
                change_pct: 0.41,
            },
        ],
        up_count: 2650,
        down_count: 2180,
        turnover_today: 1.18e12,
        turnover_yesterday: 1.09e12,
        sector_gainers: vec![
            MarketSectorBrief {
                name: "光伏设备".into(),
                change_pct: 2.35,
                board_code: None,
            },
            MarketSectorBrief {
                name: "半导体".into(),
                change_pct: 1.82,
                board_code: None,
            },
            MarketSectorBrief {
                name: "通信设备".into(),
                change_pct: 1.41,
                board_code: None,
            },
        ],
        sector_losers: vec![
            MarketSectorBrief {
                name: "房地产".into(),
                change_pct: -1.92,
                board_code: None,
            },
            MarketSectorBrief {
                name: "煤炭".into(),
                change_pct: -1.05,
                board_code: None,
            },
            MarketSectorBrief {
                name: "银行".into(),
                change_pct: -0.38,
                board_code: None,
            },
        ],
    }
}

async fn fetch_market_ribbon_eastmoney(client: &reqwest::Client) -> MarketRibbonSnapshot {
    let (
        indices,
        (up_count, down_count),
        (sector_gainers, sector_losers),
        sh,
        sz,
        y_sh,
        y_sz,
    ) = futures::join!(
        fetch_ribbon_major_indices(client),
        fetch_market_breadth_up_down(client),
        fetch_industry_sector_leaders(client),
        fetch_one_em(client, "sh000001"),
        fetch_one_em(client, "sz399001"),
        fetch_index_yesterday_turnover_from_kline(client, "sh000001"),
        fetch_index_yesterday_turnover_from_kline(client, "sz399001"),
    );
    let turnover_today = sh.turnover + sz.turnover;
    let turnover_yesterday = y_sh + y_sz;
    MarketRibbonSnapshot {
        indices,
        up_count,
        down_count,
        turnover_today,
        turnover_yesterday,
        sector_gainers,
        sector_losers,
    }
}

/// 底部工具栏：主要股指 + 涨跌家数 + 成交额/昨收额
pub async fn get_market_ribbon_impl(quote_source: &str) -> Result<MarketRibbonSnapshot, String> {
    if quote_source == "mock" {
        return Ok(mock_market_ribbon());
    }
    let client = em_client()?;
    Ok(fetch_market_ribbon_eastmoney(&client).await)
}

// --- 股市异动（东财行情中心·盘口异动，与 akshare `stock_pankou_em` 同源 `getAllStockChanges`）---

/// 单条异动下的个股标签
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketMoveStock {
    pub code: String,
    pub name: String,
    pub change_pct: f64,
}

/// 一条异动（时间 + 摘要 + 关联股；实盘接口目前多为单股一条）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketMoveItem {
    /// 展示用 `HH:mm`
    pub time: String,
    /// 如「协创数据 · 火箭发射」
    pub text: String,
    pub stocks: Vec<MarketMoveStock>,
}

fn internal_code_from_em_stock_change(c: &str) -> String {
    let c = c.trim();
    if c.len() != 6 || !c.chars().all(|ch| ch.is_ascii_digit()) {
        return format!("sz{c}");
    }
    if c.starts_with('6') {
        format!("sh{c}")
    } else if c.starts_with("92") || c.starts_with("93") {
        format!("bj{c}")
    } else {
        format!("sz{c}")
    }
}

/// `i` 字段：`量,价,涨跌幅(小数),成交额` 或前三项；涨跌幅为小数时 ×100 为百分比。
fn parse_em_stock_change_i_pct(i: &str) -> f64 {
    let p: Vec<&str> = i.split(',').collect();
    if p.len() >= 3 {
        p[2].parse::<f64>().unwrap_or(0.0) * 100.0
    } else {
        0.0
    }
}

fn format_em_tm_hhmm(tm: i64) -> String {
    if tm <= 0 {
        return "—".into();
    }
    let h = tm / 10000;
    let m = (tm / 100) % 100;
    format!("{h:02}:{m:02}")
}

fn em_stock_change_type_label(t: i64) -> &'static str {
    match t {
        4 => "封涨停板",
        8 => "封跌停板",
        16 => "打开涨停板",
        32 => "打开跌停板",
        64 => "有大买盘",
        128 => "有大卖盘",
        8193 => "大笔买入",
        8194 => "大笔卖出",
        8201 => "火箭发射",
        8202 => "快速反弹",
        8203 => "高台跳水",
        8204 => "加速下跌",
        8207 => "竞价上涨",
        8208 => "竞价下跌",
        8209 => "高开5日线",
        8210 => "低开5日线",
        8211 => "向上缺口",
        8212 => "向下缺口",
        8213 => "60日新高",
        8214 => "60日新低",
        8215 => "60日大幅上涨",
        8216 => "60日大幅下跌",
        _ => "盘口异动",
    }
}

/// 拉取单一异动类型。文档页：<https://quote.eastmoney.com/changes/>
/// `(tm 原始 HHMMSS, 异动类型 t, 展示项)`
async fn fetch_stock_changes_type(
    client: &reqwest::Client,
    type_param: &str,
) -> Result<Vec<(i64, i64, MarketMoveItem)>, String> {
    let url = format!(
        "http://push2ex.eastmoney.com/getAllStockChanges?type={}&pageindex=0&pagesize=120&ut=7eea3edcaed734bea9cbfc24409ed989&dpt=wzchanges",
        type_param
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let v: Value = resp.json().await.map_err(|e| e.to_string())?;
    let Some(arr) = v.pointer("/data/allstock").and_then(|x| x.as_array()) else {
        return Ok(vec![]);
    };
    let mut out = Vec::with_capacity(arr.len());
    for row in arr {
        let tm = row.get("tm").and_then(|x| x.as_i64()).unwrap_or(0);
        let c = row.get("c").and_then(|x| x.as_str()).unwrap_or("");
        let n = row.get("n").and_then(|x| x.as_str()).unwrap_or("—");
        let t = row.get("t").and_then(|x| x.as_i64()).unwrap_or(0);
        let i_s = row.get("i").and_then(|x| x.as_str()).unwrap_or("");
        if c.is_empty() {
            continue;
        }
        let code = internal_code_from_em_stock_change(c);
        let pct = parse_em_stock_change_i_pct(i_s);
        let label = em_stock_change_type_label(t);
        let text = format!("{n} · {label}");
        out.push((
            tm,
            t,
            MarketMoveItem {
                time: format_em_tm_hhmm(tm),
                text,
                stocks: vec![MarketMoveStock {
                    code,
                    name: n.to_string(),
                    change_pct: pct,
                }],
            },
        ));
    }
    Ok(out)
}

fn mock_market_moves() -> Vec<MarketMoveItem> {
    vec![
        MarketMoveItem {
            time: "13:13".into(),
            text: "算力租赁板块午后短线走低（示例）".into(),
            stocks: vec![
                MarketMoveStock {
                    code: "sz300857".into(),
                    name: "协创数据".into(),
                    change_pct: -14.89,
                },
                MarketMoveStock {
                    code: "sh603000".into(),
                    name: "大位科技".into(),
                    change_pct: -9.95,
                },
                MarketMoveStock {
                    code: "sz300166".into(),
                    name: "东方国信".into(),
                    change_pct: -12.10,
                },
            ],
        },
        MarketMoveItem {
            time: "13:02".into(),
            text: "光伏设备板块走强（示例）".into(),
            stocks: vec![
                MarketMoveStock {
                    code: "sz002865".into(),
                    name: "钧达股份".into(),
                    change_pct: 2.23,
                },
                MarketMoveStock {
                    code: "sh605117".into(),
                    name: "德业股份".into(),
                    change_pct: 9.91,
                },
            ],
        },
        MarketMoveItem {
            time: "11:20".into(),
            text: "陕西黑猫 · 火箭发射（示例）".into(),
            stocks: vec![MarketMoveStock {
                code: "sh601015".into(),
                name: "陕西黑猫".into(),
                change_pct: 3.12,
            }],
        },
    ]
}

/// 合并多类盘口异动，按时间降序去重。
pub async fn get_market_moves_impl(quote_source: &str) -> Result<Vec<MarketMoveItem>, String> {
    if quote_source == "mock" {
        return Ok(mock_market_moves());
    }
    let client = em_changes_client()?;
    let types = ["8201", "8204", "8193", "8194", "4", "8", "8203", "8202"];
    let futs: Vec<_> = types
        .iter()
        .map(|ty| fetch_stock_changes_type(&client, ty))
        .collect();
    let batches = join_all(futs).await;
    let mut flat: Vec<(i64, i64, MarketMoveItem)> = Vec::new();
    for res in batches {
        if let Ok(rows) = res {
            flat.extend(rows);
        }
    }
    flat.sort_by(|a, b| b.0.cmp(&a.0));
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<MarketMoveItem> = Vec::new();
    for (tm, kind, item) in flat {
        let code = item
            .stocks
            .first()
            .map(|s| s.code.as_str())
            .unwrap_or("");
        let key = format!("{tm}|{code}|{kind}");
        if seen.insert(key) {
            out.push(item);
        }
        if out.len() >= 80 {
            break;
        }
    }
    Ok(out)
}

pub async fn get_stock_intraday_impl(code: &str, quote_source: &str) -> Result<IntradaySeries, String> {
    let c = code.trim().to_lowercase();
    if c.is_empty() {
        return Err("代码为空".into());
    }
    if quote_source == "mock" {
        return Ok(mock_intraday_for_code(&c));
    }
    let client = em_client()?;
    Ok(fetch_intraday_eastmoney(&client, &c).await)
}

pub async fn get_stock_kline_impl(
    code: &str,
    period: &str,
    quote_source: &str,
) -> Result<KlineSeries, String> {
    let c = code.trim().to_lowercase();
    if c.is_empty() {
        return Err("代码为空".into());
    }
    if quote_source == "mock" {
        return Ok(mock_kline_for_code(&c, period));
    }
    let client = em_client()?;
    Ok(fetch_kline_eastmoney(&client, &c, period).await)
}

pub async fn get_stock_order_book_impl(code: &str, quote_source: &str) -> Result<OrderBook, String> {
    let c = code.trim().to_lowercase();
    if c.is_empty() {
        return Err("代码为空".into());
    }
    if quote_source == "mock" {
        return Ok(mock_order_book_for_code(&c));
    }
    let client = em_client()?;
    if quote_source == "tencent" {
        return Ok(fetch_order_book_tencent_gtimg(&client, &c).await);
    }
    Ok(fetch_order_book_eastmoney(&client, &c).await)
}
