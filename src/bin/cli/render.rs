use std::error::Error;

use cli_table::{
    format::{Border, Justify, Separator},
    print_stdout, Cell, CellStruct, Color, Style, Table,
};

pub struct Data {
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

pub fn render_table(mut data: Vec<Data>) -> Result<(), Box<dyn Error>> {
    data.sort_by(|a, b| a.name.cmp(&b.name));

    let mut contents: Vec<Vec<CellStruct>> = data.iter().map(format_row).collect();
    contents.push(format_totals(&data));

    let table = contents
        .table()
        .title(vec![
            "Name".cell().bold(true).justify(Justify::Left),
            "Quantity".cell().bold(true).justify(Justify::Center),
            "Current Price".cell().bold(true).justify(Justify::Center),
            "Current Value".cell().bold(true).justify(Justify::Center),
            "Change (Day)".cell().bold(true).justify(Justify::Center),
            "% Change (Day)".cell().bold(true).justify(Justify::Center),
            "Average Price".cell().bold(true).justify(Justify::Center),
            "Profit".cell().bold(true).justify(Justify::Center),
            "% Profit".cell().bold(true).justify(Justify::Center),
        ])
        .separator(Separator::builder().build())
        .border(Border::builder().build());

    Ok(print_stdout(table)?)
}

fn format_row(data: &Data) -> Vec<CellStruct> {
    let change_color = get_color(data.change);
    let profit_color = get_color(data.profit);

    vec![
        data.name.clone().cell().justify(Justify::Left),
        data.quantity.cell().justify(Justify::Right),
        format!("R$ {:10.2}", data.current_price)
            .cell()
            .justify(Justify::Right),
        format!("R$ {:10.2}", data.current_value)
            .cell()
            .justify(Justify::Right),
        format!("R$ {:10.2}", data.change)
            .cell()
            .justify(Justify::Right)
            .foreground_color(change_color),
        format!("{:6.2}%", data.change_percentage)
            .cell()
            .justify(Justify::Right)
            .foreground_color(change_color),
        format!("R$ {:10.2}", data.average_price)
            .cell()
            .justify(Justify::Right),
        format!("R$ {:10.2}", data.profit)
            .cell()
            .justify(Justify::Right)
            .foreground_color(profit_color),
        format!("{:6.2}%", data.profit_percentage)
            .cell()
            .justify(Justify::Right)
            .foreground_color(profit_color),
    ]
}

fn format_totals(data: &[Data]) -> Vec<CellStruct> {
    let current_value: f64 = data.iter().map(|data| data.current_value).sum();
    let original_cost: f64 = data.iter().map(|data| data.original_cost).sum();
    let last_value: f64 = data.iter().map(|data| data.last_value).sum();
    let change: f64 = data.iter().map(|data| data.change).sum();
    let profit: f64 = data.iter().map(|data| data.profit).sum();

    let change_color = get_color(change);
    let profit_color = get_color(profit);

    vec![
        "Total".cell().justify(Justify::Left).bold(true),
        "".cell(),
        "".cell(),
        format!("R$ {:10.2}", current_value)
            .cell()
            .justify(Justify::Right)
            .bold(true),
        format!("R$ {change:10.2}")
            .cell()
            .justify(Justify::Right)
            .foreground_color(change_color)
            .bold(true),
        format!("{:6.2}%", (change / last_value) * 100.0)
            .cell()
            .justify(Justify::Right)
            .bold(true)
            .foreground_color(change_color),
        "".cell(),
        format!("R$ {:10.2}", profit)
            .cell()
            .justify(Justify::Right)
            .foreground_color(profit_color)
            .bold(true),
        format!("{:6.2}%", (profit / original_cost) * 100.0)
            .cell()
            .justify(Justify::Right)
            .foreground_color(profit_color)
            .bold(true),
    ]
}

fn get_color(value: f64) -> Option<Color> {
    match value.partial_cmp(&0.0).unwrap() {
        std::cmp::Ordering::Less => Some(Color::Red),
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Greater => Some(Color::Green),
    }
}
