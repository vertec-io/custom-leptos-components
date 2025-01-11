use leptos::prelude::*;
use leptos::{children::TypedChildrenFn, mount, IntoView};
use leptos_dom::helpers::document;
use leptos_macro::component;
use reactive_graph::{effect::Effect, graph::untrack, owner::Owner};

/// Renders components somewhere else in the DOM.
///
/// Useful for inserting modals and tooltips outside of a cropping layout.
/// If no mount point is given, the portal is inserted in `document.body`;
/// If wrap_children is true, it is wrapped in a `<div>` unless  `is_svg` is `true` in which case it's wrappend in a `<g>`.
/// If no mount point is given, it will be hidden unless hide_if_none is false
/// Setting `use_shadow` to `true` places the element in a shadow root to isolate styles.
#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
pub fn DynamicPortal<V>(
    /// Target element where the children will be appended
    #[prop(into)]
    mount: ArcReadSignal<Option<web_sys::Element>>,
    /// Whether to use a shadow DOM inside `mount`. Defaults to `false`.
    #[prop(optional)]
    use_shadow: bool,
    /// When using SVG this has to be set to `true`. Defaults to `false`.
    #[prop(optional)]
    is_svg: bool,
    /// Whether to delete the container from the DOM on cleanup. Defaults to `false`.
    #[prop(optional, default=true)]
    hide_if_none: bool,
    // Whether to wrap the children in a `<div>`/`g` or directly render into the mount point. Defaults to `false`
    #[prop(optional)]
    wrap_children: bool,
    /// The children to teleport into the `mount` element
    children: TypedChildrenFn<V>,
) -> impl IntoView
where
    V: IntoView + 'static,
{
    if cfg!(target_arch = "wasm32")
        && Owner::current_shared_context()
            .map(|sc| sc.is_browser())
            .unwrap_or(true)
    {
        use send_wrapper::SendWrapper;
        use wasm_bindgen::JsCast;

        let children = children.into_inner();
        
        Effect::new(move |_| {
            let tag = if is_svg { "g" } else { "div" };
            let mount_target = mount.get();
            let mount_target_provided = mount_target.is_some();
            
           let container = match mount_target{
                Some(ref element) => {

                    if wrap_children {
                        // Create a wrapper container
                        let wrapper = create_container(tag, "dynamic_portal_wrapper", "", use_shadow);

                        // Append the existing mount target to the wrapper
                        wrapper
                            .append_child(element)
                            .expect("Failed to append mount target to wrapper");
                        wrapper
                    } else {
                        element.clone()
                    }
                }
                None => {
                    // Check if a dynamic container already exists
                    if let Some(existing_container) = document().get_element_by_id("dynamic_portal_container") {
                        // Hide the existing container
                        if hide_if_none {
                            existing_container.style("visibility:hidden; height:0; width:0;");
                        }
                        existing_container
                    } else {
                        let style = if hide_if_none { "visibility:hidden; height:0; width:0;" } else { "" };
                        let new_container = create_container(tag, "dynamic_portal_container", style, use_shadow);
                        new_container
                    }
                }
           }; 

            // Mount the children to the container
            let children = children.clone();
            let handle = SendWrapper::new((
                mount::mount_to(container.clone().unchecked_into(), {
                    move || untrack(|| children())
                }),
                container.clone(),
                mount_target_provided
            ));

            // Cleanup logic to remove the container we created when it's no longer needed
            // Don't remove the mount_target if rendering directly into the mount target.
            Owner::on_cleanup(move || {
                let (handle, container, mount_target_provided) = handle.take();
                drop(handle);

                if let Some(parent) = container.parent_node() {
                        if !mount_target_provided {
                        let _ = parent.remove_child(&container);
                    }
                }
            });
        });
    }
}


fn create_container(tag: &str, id: &str, style: &str, use_shadow: bool) -> web_sys::Element {
    // Create a new container if none exists
    let new_container = document()
        .create_element(tag)
        .expect("Failed to create element");
    
    new_container.set_id(id);
    new_container.style(style);
    
    if use_shadow {
        new_container
            .attach_shadow(&web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open))
            .expect("Failed to attach shadow root");
    }

    // Append the new container to the body
    document()
        .body()
        .expect("Document body not found")
        .append_child(&new_container)
        .expect("Failed to append container to body");

    new_container
}