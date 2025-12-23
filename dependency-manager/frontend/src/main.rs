use dioxus::prelude::asset;
use frontend::ui::root::root_ui;
use manganis::Asset;

static CSS: Asset = asset!("/assets/main.css");

fn main()
{
    dioxus::launch(|| root_ui(CSS));
}
