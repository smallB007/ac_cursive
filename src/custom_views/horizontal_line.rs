use cursive::{theme::ColorStyle, Printer, View};

pub struct HorizontalLine {
    achar: String,
    enabled: bool,
    length: usize,
}
impl HorizontalLine {
    pub fn new(kar: &str, length: usize) -> Self {
        Self {
            achar: kar.to_owned(),
            enabled: true,
            length,
        }
    }
}
impl View for HorizontalLine {
    fn draw(&self, printer: &Printer) {
        if printer.size.x == 0 {
            return;
        }

        let style = if !(self.enabled && printer.enabled) {
            ColorStyle::secondary()
        } else if printer.focused {
            ColorStyle::highlight()
        } else {
            ColorStyle::primary()
        };

        let offset = (0, 0);
        //HAlign::Center.get_offset(self.label.width(), printer.size.x);

        printer.with_color(style, |printer| {
            printer.print_hline(offset, self.length, &self.achar);
        });
    }
}
