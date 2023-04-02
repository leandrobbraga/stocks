pub struct SummaryData {
    pub name: String,
    pub quantity: u32,
    pub current_price: f64,
    pub current_value: f64,
    pub change: f64,
    pub change_percentage: f64,
    pub average_price: f64,
    pub profit: f64,
    pub profit_percentage: f64,
    pub last_value: f64,
    pub original_cost: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProfitSummaryData {
    pub profit: f64,
    pub sold_amount: f64,
    pub tax: f64,
}

pub fn render_summary(mut data: Vec<SummaryData>) {
    data.sort_by(|a, b| a.name.cmp(&b.name));

    let titles = format!(
        "\x1b[1m{:<6}  {:^8}  {:^13}  {:^13}  {:^13}  {:^13}  {:^13}  {:^13}  {:^11}\x1b[0m",
        "Name",
        "Quantity",
        "Current Price",
        "Current Value",
        "Change (Day)",
        "% Change (Day)",
        "Average Price",
        "Profit",
        "% Profit",
    );

    let contents: Vec<String> = data.iter().map(format_summary_row).collect();

    println!("{}", titles);
    contents.into_iter().for_each(|s| println!("{}", s));
    println!("{}", format_summary_totals(&data))
}

fn format_summary_row(data: &SummaryData) -> String {
    format!(
        "{:<6}  {:>8}  R$ {:>10.2}  R$ {:>10.2}  {}R$ {:>10.2}\x1b[0m  {}{:>12.2}%\x1b[0m  R$ {:>10.2}  {}R$ {:>10.2}\x1b[0m  {}{:>10.2}%\x1b[0m",
        data.name,
        data.quantity,
        data.current_price,
        data.current_value,
        get_color(data.change),
        data.change,
        get_color(data.change),
        data.change_percentage,
        data.average_price,
        get_color(data.profit),
        data.profit,
        get_color(data.profit),
        data.profit_percentage,
    )
}

fn format_summary_totals(data: &[SummaryData]) -> String {
    let current_value: f64 = data.iter().map(|data| data.current_value).sum();
    let original_cost: f64 = data.iter().map(|data| data.original_cost).sum();
    let last_value: f64 = data.iter().map(|data| data.last_value).sum();
    let change: f64 = data.iter().map(|data| data.change).sum();
    let profit: f64 = data.iter().map(|data| data.profit).sum();

    format!(
        "\x1b[1m{:<6}  {:>8}  {:>13}  R$ {:>10.2}\x1b[0m  {}R$ {:>10.2}\x1b[0m  {}{:>12.2}%\x1b[0m  {:>13}  {}R$ {:>10.2}\x1b[0m  {}{:>10.2}%\x1b[0m",
        "Total",
        "",
        "",
        current_value,
        get_color(change),
        change,
        get_color(change),
        (change / last_value) * 100.0,
        "",
        get_color(profit),
        profit,
        get_color(profit),
        (profit / original_cost) * 100.0,
    )
}

fn get_color(value: f64) -> &'static str {
    match value.partial_cmp(&0.0).unwrap() {
        std::cmp::Ordering::Less => "\x1b[31m",
        std::cmp::Ordering::Equal => "\x1b[0m",
        std::cmp::Ordering::Greater => "\x1b[32m",
    }
}

pub fn render_profit_by_month(data: [ProfitSummaryData; 12]) {
    let titles = format!(
        "\x1b[1m{:<6}  {:^13}  {:^13}  {:^8}\x1b[0m",
        "Month", "Sold Amount", "Profit", "Tax",
    );

    let contents: Vec<String> = data
        .iter()
        .enumerate()
        .map(|(i, data)| format_profit_summary_row(i as u32, data))
        .collect();

    println!("{}", titles);
    contents.into_iter().for_each(|s| println!("{}", s));
    println!("{}", format_profit_summary_totals(&data))
}

fn format_profit_summary_row(month: u32, data: &ProfitSummaryData) -> String {
    format!(
        "{:<6}  R$ {:>10.2}  {}{:>10.2}\x1b[0m  {:>10.2}",
        month,
        data.sold_amount,
        get_color(data.profit),
        data.profit,
        data.tax,
    )
}

fn format_profit_summary_totals(data: &[ProfitSummaryData]) -> String {
    let profit_total: f64 = data.iter().map(|data| data.profit).sum();
    let sold_amount_total: f64 = data.iter().map(|data| data.sold_amount).sum();
    let tax_total: f64 = data.iter().map(|data| data.tax).sum();

    format!(
        "{:<6}  R$ {:>10.2}  {}{:>10.2}\x1b[0m  {:>10.2}",
        "Total",
        sold_amount_total,
        get_color(profit_total),
        profit_total,
        tax_total,
    )
}
