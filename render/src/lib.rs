use uniremote_core::{Layout, layout::Widget};

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

pub fn render_layout(output: &mut String, layout: &Layout) {
    tracing::info!("rendering layout: {layout:#?}");

    output.push_str("<div class=\"layout\" ");
    render_handlers!(output, layout, onlaunch, onvolumedown, onvolumeup);
    output.push('>');

    for child in &layout.children {
        render_widget(output, child);
    }

    output.push_str("</div>");
}

fn render_widget(output: &mut String, widget: &Widget) {
    match widget {
        Widget::Button(button) => {
            output.push_str("<button ");

            render_handlers!(output, button, ontap, onhold, ondown, onup);

            output.push('>');
            if let Some(text) = &button.text {
                output.push_str(text);
            }
            output.push_str("</button>");
        }
        Widget::Grid(grid) => {
            output.push_str("<div class=\"grid\">");
            for widget in &grid.children {
                render_widget(output, widget);
            }
            output.push_str("</div>");
        }
        Widget::Row(row) => {
            output.push_str("<div class=\"row\">");
            for widget in &row.children {
                render_widget(output, widget);
            }
            output.push_str("</div>");
        }
        _ => {
            output.push_str("<div class=\"unknown\">Unsupported widget</div>");
        }
    }
}
