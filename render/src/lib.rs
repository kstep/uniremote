use uniremote_core::{
    Layout,
    id::LayoutId,
    layout::{
        Button, Grid, Image, Item, Label, List, Row, Slider, Tab, Tabs, Text, Theme, Toggle, Touch,
        Widget,
    },
};

pub use crate::buffer::Buffer;

mod buffer;
mod layout;

pub trait RenderHtml {
    fn render(&self, output: &mut Buffer);
}

macro_rules! render_handlers {
    ($output:ident, $widget:ident, $($name:ident),+) => {
        $(if let Some(handler) = &$widget.$name {
            $output.push_str("data-");
            $output.push_str(stringify!($name));
            $output.push_str("=\"");
            $output.push_html(handler);
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

impl RenderHtml for Layout {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<div class=\"layout\" ");
        render_style!(output, self);
        render_handlers!(output, self, onlaunch, onvolumedown, onvolumeup);
        output.push_char('>');

        for child in &self.children {
            child.render(output);
        }

        output.push_str("</div>");
    }
}

impl RenderHtml for Widget {
    fn render(&self, output: &mut Buffer) {
        match self {
            Widget::Button(button) => button.render(output),
            Widget::Grid(grid) => grid.render(output),
            Widget::Row(row) => row.render(output),
            Widget::Image(image) => image.render(output),
            Widget::Label(label) => label.render(output),
            Widget::Slider(slider) => slider.render(output),
            Widget::Text(text) => text.render(output),
            Widget::Toggle(toggle) => toggle.render(output),
            Widget::Touch(touch) => touch.render(output),
            Widget::List(list) => list.render(output),
            Widget::Tabs(tabs) => tabs.render(output),
            Widget::Space => render_space(output),
        }
    }
}

impl RenderHtml for Button {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<button ");
        render_id(output, &self.id);
        render_style!(output, self);
        render_handlers!(output, self, ontap, onhold, ondown, onup);
        output.push_char('>');
        if let Some(icon) = &self.icon {
            render_icon(output, icon);
        }
        if let Some(text) = &self.text {
            output.push_html(text);
        }
        output.push_str("</button>");
    }
}

impl RenderHtml for Grid {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<div class=\"grid\" ");
        render_id(output, &self.id);
        output.push_char('>');
        for widget in &self.children {
            widget.render(output);
        }
        output.push_str("</div>");
    }
}

impl RenderHtml for Row {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<div class=\"row\" ");
        render_id(output, &self.id);
        output.push_char('>');
        for widget in &self.children {
            widget.render(output);
        }
        output.push_str("</div>");
    }
}

impl RenderHtml for Label {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<div class=\"label\" ");
        render_id(output, &self.id);
        render_style!(output, self);
        render_handlers!(output, self, ontap, onhold, ondown, onup);
        output.push_char('>');
        if let Some(icon) = &self.icon {
            render_icon(output, icon);
        }
        if let Some(image) = &self.image {
            render_external_image(output, image);
        }
        if let Some(text) = &self.text {
            output.push_html(text);
        }
        output.push_str("</div>");
    }
}

impl RenderHtml for Image {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<img class=\"image\" ");
        render_id(output, &self.id);
        if let Some(src) = &self.image {
            output.push_str("src=\"");
            output.push_html(src);
            output.push_str("\" ");
        }
        output.push_str("alt=\"\" />");
    }
}

impl RenderHtml for Slider {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<div class=\"slider\" ");
        render_id(output, &self.id);
        output.push_char('>');
        if let Some(text) = &self.text {
            output.push_str("<label>");
            output.push_html(text);
            output.push_str("</label>");
        }
        output.push_str("<input type=\"range\" ");
        render_style!(output, self);
        render_handlers!(output, self, onchange, ondone, ondown, onup);
        output.push_str("value=\"");
        output.push_html(&self.progress.to_string());
        output.push_str("\" max=\"");
        output.push_html(&self.progressmax.to_string());
        output.push_str("\" />");
        output.push_str("</div>");
    }
}

impl RenderHtml for Text {
    fn render(&self, output: &mut Buffer) {
        if self.multiline {
            output.push_str("<textarea class=\"text\" ");
        } else {
            output.push_str("<input type=\"text\" class=\"text\" ");
        }
        render_id(output, &self.id);
        render_style!(output, self);
        render_handlers!(output, self, onchange, ondone);
        if let Some(value) = &self.text {
            if self.multiline {
                output.push_char('>');
                output.push_html(value);
                output.push_str("</textarea>");
                return;
            } else {
                output.push_str("value=\"");
                output.push_html(value);
                output.push_str("\" ");
            }
        }
        if let Some(hint) = &self.hint {
            output.push_str("placeholder=\"");
            output.push_html(hint);
            output.push_str("\" ");
        }
        if self.multiline {
            output.push_str("></textarea>");
        } else {
            output.push_str("/>");
        }
    }
}

impl RenderHtml for Toggle {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<label class=\"toggle\" ");
        render_id(output, &self.id);
        output.push_char('>');
        output.push_str("<input type=\"checkbox\" ");
        render_style!(output, self);
        render_handlers!(output, self, onchange, ontap, onhold, ondown, onup);
        if self.checked {
            output.push_str("checked ");
        }
        output.push_str("/>");
        if let Some(icon) = &self.icon {
            render_icon(output, icon);
        }
        if let Some(image) = &self.image {
            render_external_image(output, image);
        }
        if let Some(text) = &self.text {
            output.push_str("<span>");
            output.push_html(text);
            output.push_str("</span>");
        }
        output.push_str("</label>");
    }
}

impl RenderHtml for Touch {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<div class=\"touch\" ");
        render_id(output, &self.id);
        render_style!(output, self);
        render_handlers!(
            output,
            self,
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
        output.push_char('>');
        if let Some(image) = &self.image {
            render_external_image(output, image);
        }
        if let Some(text) = &self.text {
            output.push_html(text);
        }
        output.push_str("</div>");
    }
}

impl RenderHtml for List {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<ul class=\"list\" ");
        render_id(output, &self.id);
        output.push_char('>');
        for item in &self.items {
            item.render(output);
        }
        output.push_str("</ul>");
    }
}

impl RenderHtml for Item {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<li ");
        render_id(output, &self.id);
        output.push_char('>');
        if let Some(icon) = &self.icon {
            render_icon(output, icon);
        }
        if let Some(image) = &self.image {
            render_external_image(output, image);
        }
        if let Some(text) = &self.text {
            output.push_html(text);
        }
        output.push_str("</li>");
    }
}

impl RenderHtml for Tabs {
    fn render(&self, output: &mut Buffer) {
        output.push_str("<div class=\"tabs\" ");
        render_id(output, &self.id);
        render_style!(output, self);
        render_handlers!(output, self, onchange);
        output.push_char('>');

        for (i, tab) in self.tabs.iter().enumerate() {
            render_tab(output, tab, i == self.index, &self.id);
        }
        output.push_str("</div>");
    }
}

// Helper functions remain as they're used internally
fn render_icon(output: &mut Buffer, icon: &str) {
    output.push_str("<img src=\"/assets/icons/");
    output.push_html(icon);
    output.push_str(".png\" alt=\"icon\" />");
}

fn render_external_image(output: &mut Buffer, src: &str) {
    output.push_str("<img src=\"");
    output.push_html(src);
    output.push_str("\" alt=\"\" />");
}

fn render_tab(output: &mut Buffer, tab: &Tab, is_active: bool, group_id: &Option<LayoutId>) {
    // Helper closure to generate unique tab ID
    let generate_tab_id = |output: &mut Buffer| {
        if let Some(tab_id) = &tab.id {
            output.push_html(tab_id);
        } else if let Some(group_id) = group_id {
            output.push_html(group_id);
            if let Some(text) = &tab.text {
                output.push_char('-');
                output.push_html(text);
            }
        }
    };

    output.push_str("<div class=\"tab\" ");
    render_id(output, &tab.id);
    output.push_str("><input type=\"radio\" id=\"tab-");

    // Generate unique id for the radio button
    generate_tab_id(output);

    output.push_str("\" name=\"tab-group");

    if let Some(group_id) = group_id {
        output.push_char('-');
        output.push_html(group_id);
    }

    output.push_str("\" class=\"tab-input\" ");

    if is_active {
        output.push_str("checked ");
    }

    output.push_str("/>");

    if let Some(text) = &tab.text {
        output.push_str("<label class=\"tab-header\" for=\"tab-");
        // Use same id generation logic
        generate_tab_id(output);
        output.push_str("\">");
        output.push_html(text);
        output.push_str("</label>");
    }

    output.push_str("<div class=\"tab-panel\">");
    for widget in &tab.children {
        widget.render(output);
    }
    output.push_str("</div></div>");
}

fn render_space(output: &mut Buffer) {
    output.push_str("<div class=\"space\"></div>");
}

fn render_style(
    output: &mut Buffer,
    color: &Option<String>,
    lightcolor: &Option<String>,
    darkcolor: &Option<String>,
    dark: &Option<Theme>,
    light: &Option<Theme>,
) {
    output.push_str("style=\"");
    if let Some(color) = color {
        output.push_str("--default-color:");
        output.push_html(color);
        output.push_char(';');
    }

    if let Some(color) = lightcolor {
        output.push_str("--light-color:");
        output.push_html(color);
        output.push_char(';');
    }

    if let Some(color) = darkcolor {
        output.push_str("--dark-color:");
        output.push_html(color);
        output.push_char(';');
    }

    if let Some(dark) = dark {
        render_theme(output, "dark", dark);
    }

    if let Some(light) = light {
        render_theme(output, "light", light);
    }

    output.push_str("\" ");
}

fn render_theme(output: &mut Buffer, name: &str, theme: &Theme) {
    if let Some(color) = &theme.color {
        output.push_str("--theme-");
        output.push_html(name);
        output.push_str("-default-color:");
        output.push_html(color);
        output.push_char(';');
    }
    if let Some(color) = &theme.active {
        output.push_str("--theme-");
        output.push_html(name);
        output.push_str("-active-color:");
        output.push_html(color);
        output.push_char(';');
    }
    if let Some(color) = &theme.normal {
        output.push_str("--theme-");
        output.push_html(name);
        output.push_str("-normal-color:");
        output.push_html(color);
        output.push_char(';');
    }
    if let Some(color) = &theme.focus {
        output.push_str("--theme-");
        output.push_html(name);
        output.push_str("-focus-color:");
        output.push_html(color);
        output.push_char(';');
    }
}

fn render_id(output: &mut Buffer, id: &Option<LayoutId>) {
    if let Some(id) = id {
        output.push_str("id=\"");
        output.push_html(id);
        output.push_str("\" ");
    }
}
