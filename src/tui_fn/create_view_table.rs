use cursive::{align::HAlign, views::Button};
use cursive_table_view::{TableView, TableViewItem};
use rand::Rng;
use std::{cmp::Ordering, rc::Rc};
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ViewColumn {
    Count,
    Content,
}

#[derive(Clone, Debug)]
pub struct ViewStruct {
    count: usize,
    content: String,
}

impl TableViewItem<ViewColumn> for ViewStruct {
    fn to_column(&self, column: ViewColumn) -> String {
        match column {
            ViewColumn::Count => format!("{}", self.count),
            ViewColumn::Content => self.content.clone(),
        }
    }

    fn cmp(&self, other: &Self, column: ViewColumn) -> Ordering
    where
        Self: Sized,
    {
        Ordering::Equal
    }
}

pub fn create_view_table() -> TableView<ViewStruct, ViewColumn> {
    let mut items = Vec::new();

    for i in 0..50 {
        items.push(ViewStruct {
            count: i,
            content: i.to_string(),
        });
    }

    TableView::<ViewStruct, ViewColumn>::new()
        .column(ViewColumn::Count, "Count", |c| c.width_percent(5))
        .column(ViewColumn::Content, "Count", |c| c.align(HAlign::Left))
        .items(items)
}
