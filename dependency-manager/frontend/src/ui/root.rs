use dioxus::prelude::*;

pub fn root_ui(css: Asset) -> Element
{
    rsx! {
        document::Stylesheet { href: css }

        div {
            id: "title",
            {
                rsx! {
                    h1 { id: "main_title_top", {
                        "Dependency Regsitry"
                    } }
                    div { id: "main_title_bottom", {
                        rsx! {
                            p {{
                                "Dependency Regsitry service for"
                            }}
                            a { href: "https://github.com/marci1175/fog", {" Fog"} }
                        }
                    } }
                    div {
                        id: "search_field",
                        {
                            rsx! {
                                input { id: "search_text", placeholder: "Enter text", {  } }
                                button { id: "search_icon", { "Search" } }
                            }
                        }
                    }
                }
            }
        }

        div {
            id: "latest_activity",
            {
                rsx! {
                    h2 { id: "latest_act_title", {"Latest activity"} },
                    table { id: "latest_act_tbl" }
                }
            }
        }

        div {
            id: "bottom_menu",
            {
                rsx! {
                    a { id: "documentation", href: "https://marci1175.github.io/fog/book/", { "Official language book" } }
                    p { id: "made_with_hate", "Made with ♥️" }
                }
            }
        }
    }
}
