// Crate Dependencies ---------------------------------------------------------
// ----------------------------------------------------------------------------
extern crate cursive;
extern crate cursive_table_view;
extern crate rand;

// STD Dependencies -----------------------------------------------------------
// ----------------------------------------------------------------------------
use std::cmp::Ordering;

// External Dependencies ------------------------------------------------------
// ----------------------------------------------------------------------------
use cursive::theme::ColorStyle;
use cursive::traits::*;
use cursive::views::{Dialog, DummyView, LinearLayout, ResizedView, StackView, TextView};
use cursive::{align::HAlign, views::Button};
use cursive::{direction::Orientation, views::CircularFocus};
use rand::Rng;

// Modules --------------------------------------------------------------------
// ----------------------------------------------------------------------------
use cursive_table_view::{TableView, TableViewItem};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Name,
    Count,
    Rate,
}

#[derive(Clone, Debug)]
struct Foo {
    name: String,
    count: usize,
    rate: usize,
}

impl TableViewItem<BasicColumn> for Foo {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_string(),
            BasicColumn::Count => format!("{}", self.count),
            BasicColumn::Rate => format!("{}", self.rate),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name),
            BasicColumn::Count => self.count.cmp(&other.count),
            BasicColumn::Rate => self.rate.cmp(&other.rate),
        }
    }
}
mod tui_fn;
use tui_fn::create_menu;
fn main() {
    let mut siv = cursive::default();
    create_menu::create_menubar(&mut siv);
    let mut layout_panes = LinearLayout::new(Orientation::Horizontal);
    layout_panes.add_child(Dialog::around(create_table()).title("Left").full_screen());
    layout_panes.add_child(ResizedView::with_fixed_size((4, 0), DummyView));
    layout_panes.add_child(Dialog::around(create_table()).title("Right").full_screen());
    let layout_circular_panes = CircularFocus::new(layout_panes);
    let layout_circular_panes = layout_circular_panes.wrap_tab();

    let help_layout = LinearLayout::horizontal()
        .child(TextView::new("F1").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Help ]", |s| {}));
    let menu_layout = LinearLayout::horizontal()
        .child(TextView::new("F2").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Popup ]", |s| {}));
    let view_layout = LinearLayout::horizontal()
        .child(TextView::new("F3").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ View ]", |s| {}));
    let edit_layout = LinearLayout::horizontal()
        .child(TextView::new("F4").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Edit ]", |s| {}));
    let copy_layout = LinearLayout::horizontal()
        .child(TextView::new("F5").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Copy ]", |s| {}));
    let rn_mv_layout = LinearLayout::horizontal()
        .child(TextView::new("F6").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Rnm/Mv ]", |s| {}));
    let mkdir_layout = LinearLayout::horizontal()
        .child(TextView::new("F8").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ MkDir ]", |s| {}));
    let pulldown_layout = LinearLayout::horizontal()
        .child(TextView::new("F9").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Menu ]", |s| {}));
    let quit_layout = LinearLayout::horizontal()
        .child(TextView::new("F10").style(ColorStyle::title_primary()))
        .child(Button::new_raw("[ Quit ]", |s| s.quit()));

    let classic_buttons = LinearLayout::horizontal()
        .child(help_layout)
        .child(DummyView.full_width())
        .child(menu_layout)
        .child(DummyView.full_width())
        .child(view_layout)
        .child(DummyView.full_width())
        .child(edit_layout)
        .child(DummyView.full_width())
        .child(copy_layout)
        .child(DummyView.full_width())
        .child(rn_mv_layout)
        .child(DummyView.full_width())
        .child(mkdir_layout)
        .child(DummyView.full_width())
        .child(pulldown_layout)
        .child(DummyView.full_width())
        .child(quit_layout);

    let mut stack_buttons = StackView::new().fixed_height(1);
    stack_buttons
        .get_inner_mut()
        .add_fullscreen_layer(classic_buttons);
    //  let aligned_center = AlignedView::with_center(stack_buttons);
    let main_view = LinearLayout::vertical()
        .child(layout_circular_panes)
        .child(stack_buttons);

    siv.add_fullscreen_layer(main_view);

    siv.run();
}

fn create_table() -> TableView<Foo, BasicColumn> {
    let mut items = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..50 {
        items.push(Foo {
            name: format!("Name {}", i),
            count: rng.gen_range(0..=255),
            rate: rng.gen_range(0..=255),
        });
    }

    TableView::<Foo, BasicColumn>::new()
        .column(BasicColumn::Name, "Name", |c| c.width_percent(20))
        .column(BasicColumn::Count, "Count", |c| c.align(HAlign::Center))
        .column(BasicColumn::Rate, "Rate", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        })
        .items(items)
}
