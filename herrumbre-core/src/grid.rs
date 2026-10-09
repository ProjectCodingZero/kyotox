use std::collections::VecDeque;

pub mod text;
pub mod cell;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Row{
    cells: Vec<cell::Cell>,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Grid{
    cols: usize,
    rows: usize,
    lines: VecDeque<Row>
}