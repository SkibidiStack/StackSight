use dioxus::prelude::*;

#[component]
pub fn DataTable() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Table" }
            p { class: "muted", "Sortable and filterable table placeholder." }
        }
    }
}
