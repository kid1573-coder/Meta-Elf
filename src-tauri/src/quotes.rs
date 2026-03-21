use rand::Rng;
use serde::Serialize;

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
}

/// 行情数据源抽象，后续可替换为 HTTP/WebSocket 实现
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
    }
}

impl QuoteProvider for MockQuoteProvider {
    fn fetch(&self) -> Vec<QuoteRow> {
        let mut rng = rand::thread_rng();
        self.base
            .iter()
            .map(|r| {
                let jitter: f64 = rng.gen_range(-0.08..0.08);
                let change_pct = r.change_pct + jitter;
                let price = r.prev_close * (1.0 + change_pct / 100.0);
                QuoteRow {
                    price,
                    change_pct,
                    high: price * 1.006,
                    low: price * 0.997,
                    bid1: (price - 0.02).max(0.01),
                    ask1: price + 0.02,
                    daily_pl: r.daily_pl + rng.gen_range(-5.0..5.0),
                    ..r.clone()
                }
            })
            .collect()
    }
}

pub fn mock_provider() -> MockQuoteProvider {
    MockQuoteProvider::new()
}
