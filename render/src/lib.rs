use uniremote_core::{
    Layout,
    layout::{
        Button, Grid, Image, Item, Label, List, Row, Slider, Tab, Tabs, Text, Theme, Toggle, Touch,
        Widget,
    },
};

macro_rules! render_handlers {
    ($output:ident, $widget:ident, $($name:ident),+) => {
        $(if let Some(handler) = &$widget.$name {
            $output.push_str("data-");
            $output.push_str(stringify!($name));
            $output.push_str("=\"");
            $output.push_str(handler);
            $output.push_str("\" ");
        })+
    };
}

macro_rules! render_style {
    ($output:ident, $widget:ident) => {
        render_style(
            $output,
            &$widget.color,
            &$widget.lightcolor,
            &$widget.darkcolor,
            &$widget.dark,
            &$widget.light,
        );
    };
}

pub fn render_layout(output: &mut String, layout: &Layout) {
    output.push_str("<div class=\"layout\" ");
    render_style!(output, layout);
    render_handlers!(output, layout, onlaunch, onvolumedown, onvolumeup);
    output.push('>');

    for child in &layout.children {
        render_widget(output, child);
    }

    output.push_str("</div>");
}

fn render_widget(output: &mut String, widget: &Widget) {
    match widget {
        Widget::Button(button) => render_button(output, button),
        Widget::Grid(grid) => render_grid(output, grid),
        Widget::Row(row) => render_row(output, row),
        Widget::Image(image) => render_image(output, image),
        Widget::Label(label) => render_label(output, label),
        Widget::Slider(slider) => render_slider(output, slider),
        Widget::Text(text) => render_text(output, text),
        Widget::Toggle(toggle) => render_toggle(output, toggle),
        Widget::Touch(touch) => render_touch(output, touch),
        Widget::List(list) => render_list(output, list),
        Widget::Tabs(tabs) => render_tabs(output, tabs),
        Widget::Space => render_space(output),
    }
}

fn render_button(output: &mut String, button: &Button) {
    output.push_str("<button ");

    render_style!(output, button);
    render_handlers!(output, button, ontap, onhold, ondown, onup);

    output.push('>');
    if let Some(icon) = &button.icon {
        render_icon(output, icon);
    }
    if let Some(text) = &button.text {
        output.push_str(text);
    }
    output.push_str("</button>");
}

fn render_grid(output: &mut String, grid: &Grid) {
    output.push_str("<div class=\"grid\">");
    for widget in &grid.children {
        render_widget(output, widget);
    }
    output.push_str("</div>");
}

fn render_row(output: &mut String, row: &Row) {
    output.push_str("<div class=\"row\">");
    for widget in &row.children {
        render_widget(output, widget);
    }
    output.push_str("</div>");
}

fn render_icon(output: &mut String, icon: &str) {
    output.push_str("<img src=\"/assets/icons/");
    output.push_str(icon);
    output.push_str(".png\" alt=\"icon\" />");
}

fn render_external_image(output: &mut String, src: &str) {
    output.push_str("<img src=\"");
    output.push_str(src);
    output.push_str("\" alt=\"\" />");
}

fn render_label(output: &mut String, label: &Label) {
    output.push_str("<div class=\"label\" ");
    render_style!(output, label);
    render_handlers!(output, label, ontap, onhold, ondown, onup);
    output.push('>');
    if let Some(icon) = &label.icon {
        render_icon(output, icon);
    }
    if let Some(image) = &label.image {
        render_external_image(output, image);
    }
    if let Some(text) = &label.text {
        output.push_str(text);
    }
    output.push_str("</div>");
}

fn render_image(output: &mut String, image: &Image) {
    output.push_str("<img class=\"image\" ");
    if let Some(src) = &image.image {
        output.push_str("src=\"");
        output.push_str(src);
        output.push_str("\" ");
    }
    output.push_str("alt=\"\" />");
}

fn render_slider(output: &mut String, slider: &Slider) {
    output.push_str("<div class=\"slider\">");
    if let Some(text) = &slider.text {
        output.push_str("<label>");
        output.push_str(text);
        output.push_str("</label>");
    }
    output.push_str("<input type=\"range\" ");

    render_style!(output, slider);
    render_handlers!(output, slider, onchange, ondone, ondown, onup);

    output.push_str("value=\"");
    output.push_str(&slider.progress.to_string());
    output.push_str("\" max=\"");
    output.push_str(&slider.progressmax.to_string());
    output.push_str("\" />");
    output.push_str("</div>");
}

fn render_text(output: &mut String, text: &Text) {
    if text.multiline {
        output.push_str("<textarea class=\"text\" ");
    } else {
        output.push_str("<input type=\"text\" class=\"text\" ");
    }
    render_style!(output, text);
    render_handlers!(output, text, onchange, ondone);
    if let Some(value) = &text.text {
        if text.multiline {
            output.push('>');
            output.push_str(value);
            output.push_str("</textarea>");
            return;
        } else {
            output.push_str("value=\"");
            output.push_str(value);
            output.push_str("\" ");
        }
    }
    if let Some(hint) = &text.hint {
        output.push_str("placeholder=\"");
        output.push_str(hint);
        output.push_str("\" ");
    }
    if text.multiline {
        output.push_str("></textarea>");
    } else {
        output.push_str("/>");
    }
}

fn render_toggle(output: &mut String, toggle: &Toggle) {
    output.push_str("<label class=\"toggle\">");
    output.push_str("<input type=\"checkbox\" ");
    render_style!(output, toggle);
    render_handlers!(output, toggle, onchange, ontap, onhold, ondown, onup);
    if toggle.checked {
        output.push_str("checked ");
    }
    output.push_str("/>");
    if let Some(icon) = &toggle.icon {
        render_icon(output, icon);
    }
    if let Some(image) = &toggle.image {
        render_external_image(output, image);
    }
    if let Some(text) = &toggle.text {
        output.push_str("<span>");
        output.push_str(text);
        output.push_str("</span>");
    }
    output.push_str("</label>");
}

fn render_touch(output: &mut String, touch: &Touch) {
    output.push_str("<div class=\"touch\" ");
    render_style!(output, touch);
    render_handlers!(
        output,
        touch,
        ontap,
        onhold,
        ondown,
        onup,
        ondoubletap,
        ontouchsize,
        ontouchstart,
        ontouchend,
        ontouchdelta,
        onmultitap
    );
    output.push('>');
    if let Some(image) = &touch.image {
        render_external_image(output, image);
    }
    if let Some(text) = &touch.text {
        output.push_str(text);
    }
    output.push_str("</div>");
}

fn render_list(output: &mut String, list: &List) {
    output.push_str("<ul class=\"list\">");
    for item in &list.items {
        render_item(output, item);
    }
    output.push_str("</ul>");
}

fn render_item(output: &mut String, item: &Item) {
    output.push_str("<li>");
    if let Some(icon) = &item.icon {
        render_icon(output, icon);
    }
    if let Some(image) = &item.image {
        render_external_image(output, image);
    }
    if let Some(text) = &item.text {
        output.push_str(text);
    }
    output.push_str("</li>");
}

fn render_tabs(output: &mut String, tabs: &Tabs) {
    output.push_str("<div class=\"tabs\" ");
    render_style!(output, tabs);
    render_handlers!(output, tabs, onchange);
    output.push('>');

    for (i, tab) in tabs.tabs.iter().enumerate() {
        render_tab(output, tab, i == tabs.index);
    }
    output.push_str("</div>");
}

fn render_tab(output: &mut String, tab: &Tab, is_active: bool) {
    output.push_str("<div class=\"tab\"><input type=\"radio\" name=\"tabs\" ");
    if is_active {
        output.push_str("checked ");
    }
    output.push_str("/>");

    if let Some(text) = &tab.text {
        output.push_str("<label class=\"tab-header\">");
        output.push_str(text);
        output.push_str("</label>");
    }

    output.push_str("<div class=\"tab-panel\">");
    for widget in &tab.children {
        render_widget(output, widget);
    }
    output.push_str("</div></div>");
}

fn render_space(output: &mut String) {
    output.push_str("<div class=\"space\"></div>");
}

fn render_style(
    output: &mut String,
    color: &Option<String>,
    lightcolor: &Option<String>,
    darkcolor: &Option<String>,
    dark: &Option<Theme>,
    light: &Option<Theme>,
) {
    output.push_str("style=\"");
    if let Some(color) = color {
        output.push_str("--default-color:");
        output.push_str(color);
        output.push_str(";");
    }

    if let Some(color) = lightcolor {
        output.push_str("--light-color:");
        output.push_str(color);
        output.push_str(";");
    }

    if let Some(color) = darkcolor {
        output.push_str("--dark-color:");
        output.push_str(color);
        output.push_str(";");
    }

    if let Some(dark) = dark {
        render_theme(output, "dark", dark);
    }

    if let Some(light) = light {
        render_theme(output, "light", light);
    }

    output.push_str("\"");
}

fn render_theme(output: &mut String, name: &str, theme: &Theme) {
    if let Some(color) = &theme.color {
        output.push_str("--theme-");
        output.push_str(name);
        output.push_str("-default-color:");
        output.push_str(color);
        output.push_str(";");
    }
    if let Some(color) = &theme.active {
        output.push_str("--theme-");
        output.push_str(name);
        output.push_str("-active-color:");
        output.push_str(color);
        output.push_str(";");
    }
    if let Some(color) = &theme.normal {
        output.push_str("--theme-");
        output.push_str(name);
        output.push_str("-normal-color:");
        output.push_str(color);
        output.push_str(";");
    }
    if let Some(color) = &theme.focus {
        output.push_str("--theme-");
        output.push_str(name);
        output.push_str("-focus-color:");
        output.push_str(color);
        output.push_str(";");
    }
}
