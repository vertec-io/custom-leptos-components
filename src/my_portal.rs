use leptos_dom::IntoView;
use leptos_macro::component;
use leptos_reactive::untrack;
use cfg_if::cfg_if;
use wasm_bindgen::JsCast;
use leptos_dom::{document, Mountable};
use leptos_reactive::{create_effect, on_cleanup};
use web_sys;

#[component]
pub fn MyPortal(
    #[prop(into, optional)]
    mount: Option<web_sys::Element>,
    #[prop(optional)]
    use_shadow: bool,
    #[prop(optional)]
    is_svg: bool,
    children: leptos::ChildrenFn,
) -> impl IntoView {
    cfg_if! { if #[cfg(all(target_arch = "wasm32", any(feature = "hydrate", feature = "csr")))] {
        let mount = mount.unwrap_or_else(|| document().body().expect("body to exist").unchecked_into());

        create_effect(move |_| {
            let tag = if is_svg { "g" } else { "div" };
            let container = document()
                .create_element(tag)
                .expect("element creation to work");

            let render_root = if use_shadow {
                container
                    .attach_shadow(&web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open))
                    .map(|root| root.unchecked_into())
                    .unwrap_or(container.clone())
            } else {
                container.clone()
            };

            let children = untrack(|| children().into_view().get_mountable_node());
            let _ = render_root.append_child(&children);
            let _ = mount.append_child(&container);

            on_cleanup({
                let mount = mount.clone();
                move || {
                    let _ = mount.remove_child(&container);
                }
            })
        });
    } else {
        let _ = mount;
        let _ = use_shadow;
        let _ = is_svg;
        let _ = children;
    }}
}
