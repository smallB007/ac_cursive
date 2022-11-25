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
fn f_read(file: &str) -> Result<String, std::io::Error> {
    let content: String = std::fs::read_to_string(file)?;
    Ok(content)
}

pub fn create_view_table(file: &str) -> TableView<ViewStruct, ViewColumn> {
    //let file = "/home/artie/Documents/Artur Czajkowski/Poetry/Takie tam jebaneczko";
    let content = match f_read(file) {
        Ok(content) => content,
        Err(e) => e.to_string(),
    };

    let mut items = Vec::new();
    ///home/artie/Documents/Artur Czajkowski/Poetry
    for (i, line) in content.lines().enumerate() {
        items.push(ViewStruct {
            count: i,
            content: String::from(line),
        });
    }

    TableView::<ViewStruct, ViewColumn>::new()
        .column(ViewColumn::Count, "Count", |c| c.width_percent(5))
        .column(ViewColumn::Content, "Count", |c| c.align(HAlign::Left))
        .items(items)
}
