use leptos::{ChildrenFn, MaybeSignal};
// use leptos::*;
// use leptos_dom::{Element, IntoView};
// use leptos_macro::component;
// use leptos_reactive::MaybeSignal;
// use leptos_reactive::untrack;
// use web_sys::HtmlDivElement;
use leptos_dom::IntoView;
use leptos_macro::component;
use leptos_reactive::untrack;
use cfg_if::cfg_if;
use wasm_bindgen::JsCast;
use leptos_dom::{document, Mountable};
use leptos_reactive::{create_effect, on_cleanup};
use web_sys::{self, HtmlDivElement, ShadowRootMode};
use leptos::SignalGet;

// #[allow(unused_variables)] 
/// Renders components somewhere else in the DOM.
///
/// Useful for inserting modals and tooltips outside of a cropping layout.
/// If no mount point is given, the portal is inserted in `document.body`;
/// it is wrapped in a `<div>` unless  `is_svg` is `true`, in which case it's wrapped in a `<g>`.
/// Setting `use_shadow` to `true` places the element in a shadow root to isolate styles.
#[cfg_attr(
    any(debug_assertions, feature = "ssr"),
    tracing::instrument(level = "trace", skip_all)
)]
#[component]
pub fn DynamicPortal(
    /// Target element where the children will be appended. Accepts a reactive `MaybeSignal`.
    #[prop(into, optional)]
    mount: MaybeSignal<Option<HtmlDivElement>>,
    /// Whether to use a shadow DOM inside `mount`. Defaults to `false`.
    #[prop(optional)]
    use_shadow: bool,
    /// When using SVG this has to be set to `true`. Defaults to `false`.
    #[prop(optional)]
    is_svg: bool,
    /// The children to teleport into the `mount` element
    children: ChildrenFn,
) -> impl IntoView {
    cfg_if! {
        if #[cfg(all(target_arch = "wasm32", any(feature = "hydrate", feature = "csr")))] {
            // use leptos_reactive::{create_effect, on_cleanup};
            // use web_sys::{Element, ShadowRootMode};
            // use wasm_bindgen::JsCast;
            // use leptos_dom::{document, IntoView, View};
            // Effect that updates and mounts children reactively whenever `mount` changes
            create_effect(move |_| {
                // Unwrap the `MaybeSignal` to get the current `mount` element reactively
                let mount_element = mount.get();
                // let mount_element = mount.
                

                // Proceed only if a valid `mount` element is provided
                if let Some(mount_element) = mount_element {
                    let tag = if is_svg { "g" } else { "div" };
                    let container = document()
                        .create_element(tag)
                        .expect("Element creation to succeed");

                    // Optionally attach Shadow DOM for style isolation
                    let render_root = if use_shadow {
                        container
                            .attach_shadow(&web_sys::ShadowRootInit::new(ShadowRootMode::Open))
                            .map(|root| root.unchecked_into())
                            .unwrap_or_else(|_| container.clone())
                    } else {
                        container.clone()
                    };

                    // Render children into the container
                    let children_view = untrack(|| children().into_view().get_mountable_node());
                    render_root
                        .append_child(&children_view)
                        .expect("Failed to append children");

                    // Mount the container to the target element
                    mount_element
                        .append_child(&container)
                        .expect("Failed to mount container");

                    // Cleanup: Remove the container when the component is destroyed
                    on_cleanup(move || {
                        let _ = mount_element.remove_child(&container);
                    });
                }
            });
        } else {
            // SSR Fallback: Render an empty view
            view! {}
        }
    }
}
