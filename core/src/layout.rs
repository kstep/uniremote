use serde::Deserialize;

use crate::id::{EventHanlder, LayoutId};

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename = "layout")]
pub struct Layout {
    #[serde(default, rename = "@orientation")]
    pub orientation: Orientation,
    #[serde(default, rename = "@scroll")]
    pub scroll: Scroll,
    #[serde(default, rename = "@onlaunch")]
    pub onlaunch: Option<EventHanlder>,
    #[serde(default, rename = "@onvolumedown")]
    pub onvolumedown: Option<EventHanlder>,
    #[serde(default, rename = "@onvolumeup")]
    pub onvolumeup: Option<EventHanlder>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "grid")]
pub struct Grid {
    #[serde(default, rename = "$value")]
    pub children: Vec<Widget>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "row")]
pub struct Row {
    #[serde(default, rename = "$value")]
    pub children: Vec<Widget>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scroll {
    #[default]
    Vertical,
    Horizontal,
    None,
    Both,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Visible,
    Invisible,
    Gone,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scale {
    #[default]
    Icon,
    Fill,
    Fit,
    Native,
}

#[derive(Debug, Clone, Deserialize)]
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
    #[serde(default, rename = "@ontap")]
    pub ontap: Option<EventHanlder>,
    #[serde(default, rename = "@onhold")]
    pub onhold: Option<EventHanlder>,
    #[serde(default, rename = "@onup")]
    pub onup: Option<EventHanlder>,
    #[serde(default, rename = "@ondown")]
    pub ondown: Option<EventHanlder>,

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "button")]
pub struct Button {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@text")]
    pub text: Option<String>,
    #[serde(default, rename = "@ontap")]
    pub ontap: Option<EventHanlder>,
    #[serde(default, rename = "@onhold")]
    pub onhold: Option<EventHanlder>,
    #[serde(default, rename = "@onup")]
    pub onup: Option<EventHanlder>,
    #[serde(default, rename = "@ondown")]
    pub ondown: Option<EventHanlder>,
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

#[derive(Debug, Clone, Deserialize)]
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
    #[serde(default, rename = "@onchange")]
    pub onchange: Option<EventHanlder>,
    #[serde(default, rename = "@ondone")]
    pub ondone: Option<EventHanlder>,
    #[serde(default, rename = "@ondown")]
    pub ondown: Option<EventHanlder>,
    #[serde(default, rename = "@onup")]
    pub onup: Option<EventHanlder>,

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

#[derive(Debug, Clone, Deserialize)]
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
    #[serde(default, rename = "@onchange")]
    pub onchange: Option<EventHanlder>,
    #[serde(default, rename = "@ondone")]
    pub ondone: Option<EventHanlder>,

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

#[derive(Debug, Clone, Deserialize)]
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
    #[serde(default, rename = "@onchange")]
    pub onchange: Option<EventHanlder>,
    #[serde(default, rename = "@ontap")]
    pub ontap: Option<EventHanlder>,
    #[serde(default, rename = "@onhold")]
    pub onhold: Option<EventHanlder>,
    #[serde(default, rename = "@onup")]
    pub onup: Option<EventHanlder>,
    #[serde(default, rename = "@ondown")]
    pub ondown: Option<EventHanlder>,

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "tabs")]
pub struct Tabs {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@index")]
    pub index: usize,
    #[serde(default, rename = "@onchange")]
    pub onchange: Option<EventHanlder>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "tab")]
pub struct Tab {
    #[serde(default, rename = "@id")]
    pub id: Option<LayoutId>,
    #[serde(default, rename = "@visibility")]
    pub visibility: Visibility,
    #[serde(default, rename = "@text")]
    pub text: Option<String>,
    #[serde(default)]
    pub grid: Option<Grid>,
    #[serde(default)]
    pub list: Option<List>,
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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
    #[serde(default, rename = "@ontap")]
    pub ontap: Option<EventHanlder>,
    #[serde(default, rename = "@onhold")]
    pub onhold: Option<EventHanlder>,
    #[serde(default, rename = "@ondown")]
    pub ondown: Option<EventHanlder>,
    #[serde(default, rename = "@onup")]
    pub onup: Option<EventHanlder>,
    #[serde(default, rename = "@ondoubletap")]
    pub ondoubletap: Option<EventHanlder>,
    #[serde(default, rename = "@ontouchsize")]
    pub ontouchsize: Option<EventHanlder>,
    #[serde(default, rename = "@ontouchstart")]
    pub ontouchstart: Option<EventHanlder>,
    #[serde(default, rename = "@ontouchend")]
    pub ontouchend: Option<EventHanlder>,
    #[serde(default, rename = "@ontouchdelta")]
    pub ontouchdelta: Option<EventHanlder>,
    #[serde(default, rename = "@onmultitap")]
    pub onmultitap: Option<EventHanlder>,

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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone)]
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
