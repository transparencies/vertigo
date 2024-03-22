//! Methods for debugging or testing vertigo components by recreating HTML-like string from dom commands

use std::collections::HashMap;

use crate::driver_module::api::CallbackId;
use crate::driver_module::StaticString;
use crate::{get_driver, DomId, DriverDomCommand};

/// Make driver start gathering DOM commands into separate log
pub fn log_start() {
    get_driver().inner.dom.log_start()
}

/// Stop gathering logs, return vector of commands and erase the log
pub fn log_take() -> Vec<DriverDomCommand> {
    get_driver().inner.dom.log_take()
}

/// Fragment of DOM created from DOM commands, debuggable
#[derive(Clone, Debug)]
pub struct DomDebugFragment {
    pub map: HashMap<DomId, DomDebugNode>,
    pub css: HashMap<String, String>,
    pub root_node: Option<DomId>,
}

/// Part of `DomDebugFragment` representing single node in DOM
#[derive(Clone, Debug, Default)]
pub struct DomDebugNode {
    pub id: DomId,
    pub parent_id: DomId,
    pub name: StaticString,
    pub attrs: HashMap<StaticString, String>,
    pub callbacks: HashMap<String, CallbackId>,
    pub children: Vec<DomId>,
    pub text: Option<String>,
}

impl DomDebugFragment {
    /// Creates debug fragment directly from driver log. Log should be started by `sta
    pub fn from_log() -> Self {
        Self::from_cmds(log_take())
    }

    /// Creates debug fragment from vector of commands generated by vertigo
    pub fn from_cmds(cmds: Vec<DriverDomCommand>) -> Self {
        let mut map = HashMap::<DomId, DomDebugNode>::new();
        let mut css = HashMap::<String, String>::new();

        for cmd in cmds {
            match cmd {
                DriverDomCommand::CreateNode { id, name } => {
                    map.insert(id, DomDebugNode::from_name(id, name));
                }
                DriverDomCommand::CreateText { id, value } => {
                    map.insert(id, DomDebugNode::from_text(id, value));
                }
                DriverDomCommand::UpdateText { id, value } => {
                    map.entry(id).and_modify(|node| node.text = Some(value));
                }
                DriverDomCommand::SetAttr { id, name, value } => {
                    if let Some(node) = map.get_mut(&id) {
                        if name == "class".into() {
                            if let Some(new_styles) = css.get(&format!(".{value}")) {
                                let mut styles = String::new();
                                if let Some(old_styles) = node.attrs.get(&("style".into())) {
                                    styles.push_str(old_styles);
                                }
                                styles.push_str(new_styles);
                                node.attrs.insert("style".into(), styles);
                            } else {
                                node.attrs.insert(name, value);
                            }
                        } else {
                            node.attrs.insert(name, value);
                        }
                    }
                }
                DriverDomCommand::RemoveAttr { id, name } => {
                    if let Some(node) = map.get_mut(&id) {
                        if name == "class".into() {
                            // TODO
                        } else {
                            node.attrs.remove(&name);
                        }
                    }
                }
                DriverDomCommand::RemoveNode { id }
                | DriverDomCommand::RemoveText { id }
                | DriverDomCommand::RemoveComment { id } => {
                    // Delete node from it's parent children list
                    if let Some(parent_id) = map.get(&id).map(|node| node.parent_id) {
                        map.entry(parent_id).and_modify(|parent| {
                            parent.children.retain(|child_id| *child_id != id)
                        });
                    }
                    map.remove(&id);
                }
                DriverDomCommand::InsertBefore {
                    parent,
                    child,
                    ref_id,
                } => {
                    // Change child's parent
                    let child_parent_pair = if let Some(child) = map.get_mut(&child) {
                        let old_parent = child.parent_id;
                        child.parent_id = parent;
                        Some((old_parent, child.clone()))
                    } else {
                        None
                    };

                    if let Some((old_parent, child)) = child_parent_pair {
                        // Remove child from parent
                        if let Some(old_parent) = map.get_mut(&old_parent) {
                            old_parent.children.retain(|id| *id != child.id);
                        }

                        // Add child to new parent
                        if let Some(parent) = map.get_mut(&parent) {
                            // Insert before child indicated by ref_id
                            if let Some(ref_id) = ref_id {
                                if let Some(index) = parent
                                    .children
                                    .iter()
                                    .position(|elem_id| *elem_id == ref_id)
                                {
                                    parent.children.insert(index, child.id)
                                } else {
                                    // or at the end if child not found
                                    parent.children.push(child.id);
                                }
                            } else {
                                parent.children.push(child.id);
                            }
                        }
                    }
                }
                DriverDomCommand::InsertCss { selector, value } => {
                    println!("InsertCss {selector} {value}");
                    css.insert(selector, value);
                }
                DriverDomCommand::CreateComment { id, value } => {
                    map.insert(id, DomDebugNode::from_text(id, format!("<!-- {value} -->")));
                }
                DriverDomCommand::CallbackAdd {
                    id,
                    event_name,
                    callback_id,
                } => {
                    map.entry(id).and_modify(|node| {
                        node.callbacks.insert(event_name, callback_id);
                    });
                }
                DriverDomCommand::CallbackRemove {
                    id,
                    event_name,
                    callback_id: _callback_id,
                } => {
                    map.entry(id).and_modify(|node| {
                        node.callbacks.remove(&event_name);
                    });
                }
            }
        }

        // Try to return real root node
        let root_node = if let Some(root_node) = map
            .iter()
            .find(|(_, child)| child.parent_id == DomId::root_id())
            .map(|(id, _)| id)
            .cloned()
        {
            Some(root_node)
        } else {
            // Fallback to parent without node, as this is usually the case for dom fragment not mounted to anything
            map.iter()
                .find(|(_, child)| child.parent_id == DomId::from_u64(0))
                .map(|(id, _)| id)
                .cloned()
        };

        Self {
            map,
            css,
            root_node,
        }
    }

    /// Construct a pseudo-html string from DomDebugFragment.
    ///
    /// May render only part of the fragment if nodes are not connected to one root element.
    pub fn to_pseudo_html(&self) -> String {
        self.root_node
            .map(|rn| self.render(&rn))
            .unwrap_or_default()
    }

    fn render(&self, node_id: &DomId) -> String {
        if let Some(node) = self.map.get(node_id) {
            if node.name.is_empty() {
                node.text.clone().unwrap_or_default()
            } else {
                let children = node
                    .children
                    .iter()
                    .map(|c| self.render(c))
                    .collect::<Vec<_>>()
                    .join("");
                let attrs = node
                    .attrs
                    .iter()
                    .map(|(k, v)| format!(" {k}='{v}'"))
                    .collect::<Vec<_>>()
                    .join("");
                let callbacks = node
                    .callbacks
                    .iter()
                    .map(|(k, v)| format!(" {k}={}", v.as_u64()))
                    .collect::<Vec<_>>()
                    .join("");
                format!(
                    "<{}{attrs}{callbacks}>{children}</{}>",
                    node.name.as_str(),
                    node.name.as_str()
                )
            }
        } else {
            String::default()
        }
    }
}

impl DomDebugNode {
    pub fn from_name(id: DomId, name: StaticString) -> Self {
        Self {
            id,
            parent_id: DomId::from_u64(0),
            name,
            ..Default::default()
        }
    }

    pub fn from_text(id: DomId, text: String) -> Self {
        Self {
            id,
            parent_id: DomId::from_u64(0),
            text: Some(text),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{log_start, DomDebugFragment};
    use crate::{self as vertigo, css, dom, driver_module::api::CallbackId};

    use std::sync::Mutex;
    static SEMAPHORE: Mutex<()> = Mutex::new(());

    #[test]
    fn pseudo_html_list() {
        let _lock = SEMAPHORE.lock().unwrap();
        CallbackId::reset();

        log_start();
        let _el = dom! {
            <div>
                <ol>
                    <li>"item1"</li>
                    <li>"item2"</li>
                    <li>"item3"</li>
                </ol>
            </div>
        };
        let html = DomDebugFragment::from_log().to_pseudo_html();
        assert_eq!(
            html,
            "<div><ol><li>item1</li><li>item2</li><li>item3</li></ol></div>"
        );
    }

    #[test]
    fn pseudo_html_css() {
        let _lock = SEMAPHORE.lock().unwrap();
        CallbackId::reset();

        let green = css!("color: green;");
        log_start();
        let _el = dom! {
            <div css={green}>"something"</div>
        };
        let html = DomDebugFragment::from_log().to_pseudo_html();
        assert_eq!(html, "<div style='color: green'>something</div>");
    }

    #[test]
    fn pseudo_html_callback() {
        let _lock = SEMAPHORE.lock().unwrap();
        CallbackId::reset();

        let callback = || ();
        log_start();
        let _el = dom! {
            <div on_click={callback}>"something"</div>
        };
        let html = DomDebugFragment::from_log().to_pseudo_html();
        assert_eq!(html, "<div click=1>something</div>");
    }
}
