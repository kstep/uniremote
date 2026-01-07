use serde::Deserialize;

use crate::id::{ActionId, LayoutId};

#[derive(Default, Debug, Deserialize)]
#[serde(rename = "layout")]
pub struct Layout {
    #[serde(default, rename = "@orientation")]
    pub orientation: Orientation,
    #[serde(default, rename = "@scroll")]
    pub scroll: Scroll,
    #[serde(default, rename = "@onlaunch", alias = "@onLaunch")]
    pub onlaunch: Option<ActionId>,
    #[serde(default, rename = "@onvolumedown", alias = "@onVolumeDown")]
    pub onvolumedown: Option<ActionId>,
    #[serde(default, rename = "@onvolumeup", alias = "@onVolumeUp")]
    pub onvolumeup: Option<ActionId>,
    #[serde(default, rename = "$value")]
    pub children: Vec<Widget>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "grid")]
pub struct Grid {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "$value")]
    pub children: Vec<Widget>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "row")]
pub struct Row {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "$value")]
    pub children: Vec<Widget>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Widget {
    Button(Button),
    Image(Image),
    Label(Label),
    Slider(Slider),
    Text(Text),
    Toggle(Toggle),
    Touch(Touch),
    List(List),
    Grid(Grid),
    Row(Row),
    Tabs(Tabs),
    Space,
}

#[derive(Default, Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

#[derive(Default, Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Scroll {
    #[default]
    Vertical,
    Horizontal,
    None,
    Both,
}

#[derive(Default, Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Visible,
    Invisible,
    Gone,
}

#[derive(Default, Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Default, Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Scale {
    #[default]
    Icon,
    Fill,
    Fit,
    Native,
}

#[derive(Debug, Deserialize)]
pub struct Label {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@text")]
    pub text: Option<String>,
    #[serde(default, rename = "@textalign")]
    pub textalign: TextAlign,
    #[serde(default, rename = "@icon")]
    pub icon: Option<String>,
    #[serde(default, rename = "@image")]
    pub image: Option<String>,
    #[serde(default, rename = "@ontap", alias = "@onTap")]
    pub ontap: Option<ActionId>,
    #[serde(default, rename = "@onhold", alias = "@onHold")]
    pub onhold: Option<ActionId>,
    #[serde(default, rename = "@onup", alias = "@onUp")]
    pub onup: Option<ActionId>,
    #[serde(default, rename = "@ondown", alias = "@onDown")]
    pub ondown: Option<ActionId>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "button")]
pub struct Button {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@text")]
    pub text: Option<String>,
    #[serde(default, rename = "@ontap", alias = "@onTap")]
    pub ontap: Option<ActionId>,
    #[serde(default, rename = "@onhold", alias = "@onHold")]
    pub onhold: Option<ActionId>,
    #[serde(default, rename = "@onup", alias = "@onUp")]
    pub onup: Option<ActionId>,
    #[serde(default, rename = "@ondown", alias = "@onDown")]
    pub ondown: Option<ActionId>,
    #[serde(default, rename = "@textalign")]
    pub textalign: TextAlign,
    #[serde(default, rename = "@icon")]
    pub icon: Option<String>,
    #[serde(default, rename = "@image")]
    pub image: Option<String>,
    #[serde(default, rename = "@scale")]
    pub scale: Scale,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "slider")]
pub struct Slider {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@text")]
    pub text: Option<String>,
    #[serde(default, rename = "@progress")]
    pub progress: usize,
    #[serde(default = "default_progressmax", rename = "@progressmax")]
    pub progressmax: usize,
    #[serde(default, rename = "@onchange", alias = "@onChange")]
    pub onchange: Option<ActionId>,
    #[serde(default, rename = "@ondone", alias = "@onDone")]
    pub ondone: Option<ActionId>,
    #[serde(default, rename = "@ondown", alias = "@onDown")]
    pub ondown: Option<ActionId>,
    #[serde(default, rename = "@onup", alias = "@onUp")]
    pub onup: Option<ActionId>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

fn default_progressmax() -> usize {
    100
}

#[derive(Debug, Deserialize)]
#[serde(rename = "text")]
pub struct Text {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@text")]
    pub text: Option<String>,
    #[serde(default, rename = "@textalign")]
    pub textalign: TextAlign,
    #[serde(default, rename = "@hint")]
    pub hint: Option<String>,
    #[serde(default, rename = "@multiline")]
    pub multiline: bool,
    #[serde(default, rename = "@onchange", alias = "@onChange")]
    pub onchange: Option<ActionId>,
    #[serde(default, rename = "@ondone", alias = "@onDone")]
    pub ondone: Option<ActionId>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "toggle")]
pub struct Toggle {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@checked")]
    pub checked: bool,
    #[serde(default, rename = "@text")]
    pub text: Option<String>,
    #[serde(default, rename = "@textalign")]
    pub textalign: TextAlign,
    #[serde(default, rename = "@icon")]
    pub icon: Option<String>,
    #[serde(default, rename = "@image")]
    pub image: Option<String>,
    #[serde(default, rename = "@onchange", alias = "@onChange")]
    pub onchange: Option<ActionId>,
    #[serde(default, rename = "@ontap", alias = "@onTap")]
    pub ontap: Option<ActionId>,
    #[serde(default, rename = "@onhold", alias = "@onHold")]
    pub onhold: Option<ActionId>,
    #[serde(default, rename = "@onup", alias = "@onUp")]
    pub onup: Option<ActionId>,
    #[serde(default, rename = "@ondown", alias = "@onDown")]
    pub ondown: Option<ActionId>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "tabs")]
pub struct Tabs {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@index")]
    pub index: usize,
    #[serde(default, rename = "@onchange", alias = "@onChange")]
    pub onchange: Option<ActionId>,
    #[serde(default, rename = "$value")]
    pub tabs: Vec<Tab>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "tab")]
pub struct Tab {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@text")]
    pub text: Option<String>,
    #[serde(default, rename = "$value")]
    pub children: Vec<Widget>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "image")]
pub struct Image {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@image")]
    pub image: Option<String>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "touch")]
pub struct Touch {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@text")]
    pub text: Option<String>,
    #[serde(default, rename = "@image")]
    pub image: Option<String>,
    #[serde(default, rename = "@ontap", alias = "@onTap")]
    pub ontap: Option<ActionId>,
    #[serde(default, rename = "@onhold", alias = "@onHold")]
    pub onhold: Option<ActionId>,
    #[serde(default, rename = "@ondown", alias = "@onDown")]
    pub ondown: Option<ActionId>,
    #[serde(default, rename = "@onup", alias = "@onUp")]
    pub onup: Option<ActionId>,
    #[serde(default, rename = "@ondoubletap", alias = "@onDoubleTap")]
    pub ondoubletap: Option<ActionId>,
    #[serde(default, rename = "@ontouchsize", alias = "@onTouchSize")]
    pub ontouchsize: Option<ActionId>,
    #[serde(default, rename = "@ontouchstart", alias = "@onTouchStart")]
    pub ontouchstart: Option<ActionId>,
    #[serde(default, rename = "@ontouchend", alias = "@onTouchEnd")]
    pub ontouchend: Option<ActionId>,
    #[serde(default, rename = "@ontouchdelta", alias = "@onTouchDelta")]
    pub ontouchdelta: Option<ActionId>,
    #[serde(default, rename = "@onmultitap", alias = "@onMultiTap")]
    pub onmultitap: Option<ActionId>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "list")]
pub struct List {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "$value")]
    pub items: Vec<Item>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "item")]
pub struct Item {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@text", alias = "$text")]
    pub text: Option<String>,
    #[serde(default, rename = "@icon")]
    pub icon: Option<String>,
    #[serde(default, rename = "@image")]
    pub image: Option<String>,

    #[serde(default, rename = "@color")]
    pub color: Option<String>,
    #[serde(default, rename = "@lightcolor")]
    pub lightcolor: Option<String>,
    #[serde(default, rename = "@darkcolor")]
    pub darkcolor: Option<String>,
    #[serde(default, rename = "@dark")]
    pub dark: Option<Theme>,
    #[serde(default, rename = "@light")]
    pub light: Option<Theme>,
}

#[derive(Debug)]
pub struct Theme {
    pub color: Option<String>,
    pub normal: Option<String>,
    pub focus: Option<String>,
    pub active: Option<String>,
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        let mut color = None;
        let mut normal = None;
        let mut focus = None;
        let mut active = None;

        for part in s.split(';') {
            let Some((name, value)) = part.split_once(':') else {
                continue;
            };
            let target = match name.trim() {
                "color" => &mut color,
                "normal" => &mut normal,
                "focus" => &mut focus,
                "active" => &mut active,
                _ => continue,
            };

            *target = Some(value.trim().to_string());
        }

        Ok(Theme {
            color,
            normal,
            focus,
            active,
        })
    }
}
