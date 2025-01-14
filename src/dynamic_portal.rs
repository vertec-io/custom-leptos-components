use leptos::{children, prelude::*};
use leptos::{children::TypedChildrenFn, mount, IntoView};
use leptos_dom::helpers::document;
use leptos_macro::component;
use reactive_graph::{effect::Effect, graph::untrack, owner::Owner};
pub use send_wrapper::SendWrapper;

/// Renders components somewhere else in the DOM.
///
/// Useful for inserting modals and tooltips outside of a cropping layout.
/// If no mount point is given, the portal is inserted in `document.body`;
/// If wrap_children is true, children are wrapped in a `<div>` unless  `is_svg` is `true` in which case it's wrappend in a `<g>`.
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
                    leptos::logging::log!("Running dynamic portal in the mount target");
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
                    leptos::logging::log!("No mount target provided, checking for existing hidden container");
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

#[component]
pub fn PersistentPortal<V>(
    /// Target element where the children will be appended
    #[prop(into)]
    mount: ArcReadSignal<Option<web_sys::Element>>,
    /// When using SVG this has to be set to `true`. Defaults to `false`.
    #[prop(optional)]
    is_svg: bool,
    /// The children to teleport into the `mount` element
    children: TypedChildrenFn<V>,
) -> impl IntoView
where
    V: IntoView + 'static,
{
    use wasm_bindgen::JsCast;

    // Generate the children once and hold a reference to them
    let children = children.into_inner();
    let mut created_once = false;

    // Effect to handle mounting and moving children dynamically
    Effect::new(move |_| {
        // Determine the tag type for the hidden container (div for HTML, g for SVG)
        let tag = if is_svg { "g" } else { "div" };

        // Find or create the hidden fallback container
        let hidden_container = document()
            .get_element_by_id("dynamic_portal_container")
            .unwrap_or_else(|| {
                leptos::logging::log!("Creating new hidden container for PersistentPortal.");
                let container = document()
                    .create_element(tag)
                    .expect("Failed to create hidden container");
                container.set_id("dynamic_portal_container");
                container
                    .set_attribute("style", "visibility: hidden; height: 0; width: 0;")
                    .expect("Failed to set container style");
                document()
                    .body()
                    .expect("Document body not found")
                    .append_child(&container)
                    .expect("Failed to append container to body");
                container
            });

        // Get the current mount target
        let current_target = mount.get();

        // Find the first child in the hidden container (e.g., the `<canvas>`)
        let persistent_child = hidden_container.first_child();

        if let Some(target) = current_target {
            // If there's a mount target, ensure the child is moved there
            leptos::logging::log!("Moving PersistentPortal children to mount target.");

            // If the canvas is already in the target, do nothing
            if target.contains(persistent_child.as_ref()) {
                leptos::logging::log!("Canvas already in the target, no action needed.");
                return;
            }

            // Move the canvas to the target
            if let Some(child) = persistent_child {
                target
                    .append_child(&child)
                    .expect("Failed to append child to mount target");
            } else {
                leptos::logging::log!("No persistent child found in hidden container.");
            }
        } else {
            // If there's no mount target, ensure the child is in the hidden container
            leptos::logging::log!("Moving PersistentPortal children back to hidden container.");

            // If the canvas is already in the hidden container, do nothing
            if hidden_container.contains(persistent_child.as_ref()) {
                leptos::logging::log!("Canvas already in the hidden container, no action needed.");
                return;
            }else{
                leptos::logging::log!("The mount point will be gone by now, it should have already gotten moved to the hidden container outside of this effect");
                // Move the canvas back to the hidden container

                if !hidden_container.has_child_nodes() {
                    let children = children.clone();

                    if !created_once {
                        
                        leptos::logging::log!("This should create the children nodes for the first time and persist afterwards\n\n{:?}", &hidden_container);
                        let handle = SendWrapper::new((mount::mount_to(hidden_container.clone().unchecked_into(), {
                                let children = children.clone();
                                move || children().into_view()
                            }),
                            hidden_container.clone())
                        );

                        Owner::on_cleanup(move || {
                            let (handle, _hiddencontainer) = handle.take();
                            handle.forget();
                        });
                        created_once = true;
                    }

                }
            }

        }
    });
}
