use std::rc::Rc;
use crate::{
    driver_module::{driver::Driver, DomAccess},
    dom::{
        dom_node::DomNode,
        dom_id::DomId,
    }, get_driver, Css, Client, Computed, struct_mut::VecMut, ApiImport, DropResource, JsValue, DropFileItem,
};

use super::{types::{KeyDownEvent, DropFileEvent}, dom_node::{DomNodeFragment}};
use crate::struct_mut::VecDequeMut;

pub enum AttrValue<T: Into<String> + Clone + PartialEq + 'static> {
    String(T),
    Computed(Computed<T>)
}

impl From<&'static str> for AttrValue<&'static str> {
    fn from(value: &'static str) -> Self {
        AttrValue::String(value)
    }
}

impl From<String> for AttrValue<String> {
    fn from(value: String) -> Self {
        AttrValue::String(value)
    }
}

impl From<Computed<String>> for AttrValue<String> {
    fn from(value: Computed<String>) -> Self {
        AttrValue::Computed(value)
    }
}

pub enum CssValue {
    Css(Css),
    Computed(Computed<Css>),
}

impl From<Css> for CssValue {
    fn from(value: Css) -> Self {
        CssValue::Css(value)
    }
}

impl From<Computed<Css>> for CssValue {
    fn from(value: Computed<Css>) -> Self {
        CssValue::Computed(value)
    }
}

#[derive(Clone)]
pub struct DomElementRef {
    api: Rc<ApiImport>,
    id: DomId,
}

impl DomElementRef {
    pub fn new(api: Rc<ApiImport>, id: DomId) -> DomElementRef {
        DomElementRef {
            api,
            id,
        }
    }

    pub fn dom_access(&self) -> DomAccess {
        self.api.dom_access().element(self.id.to_u64())
    }
}

impl PartialEq for DomElementRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// A Real DOM representative - element kind
pub struct DomElement {
    driver: Driver,
    id_dom: DomId,
    child_node: VecDequeMut<DomNode>,
    subscriptions: VecMut<Client>,
    drop: VecMut<DropResource>,
}

impl DomElement {
    pub fn new(name: &'static str) -> DomElement {
        let node_id = DomId::default();

        let driver = get_driver();

        driver.inner.dom.create_node(node_id, name);

        DomElement {
            driver,
            id_dom: node_id,
            child_node: VecDequeMut::new(),
            subscriptions: VecMut::new(),
            drop: VecMut::new(),
        }
    }

    pub fn get_ref(&self) -> DomElementRef {
        DomElementRef::new(self.driver.inner.api.clone(), self.id_dom)
    }

    pub fn create_with_id(id: DomId) -> DomElement {
        let driver = get_driver();

        DomElement {
            driver,
            id_dom: id,
            child_node: VecDequeMut::new(),
            subscriptions: VecMut::new(),
            drop: VecMut::new(),
        }
    }

    fn subscribe<T: Clone + PartialEq + 'static>(&self, value: Computed<T>, call: impl Fn(T) + 'static) {
        let client = value.subscribe(call);
        self.subscriptions.push(client);
    }

    pub fn css(self, css: CssValue) -> Self {
        match css {
            CssValue::Css(css) => {
                let class_name = get_driver().get_class_name(&css);
                self.driver.inner.dom.set_attr(self.id_dom, "class", &class_name);
            },
            CssValue::Computed(css) => {
                let id_dom = self.id_dom;
                let driver = self.driver.clone();

                self.subscribe(css, move |css| {
                    let class_name = driver.get_class_name(&css);
                    driver.inner.dom.set_attr(id_dom, "class", &class_name);
                });
            }
        }
        self
    }

    pub fn attr<T: Into<String> + Clone + PartialEq + 'static>(self, name: &'static str, value: AttrValue<T>) -> Self {
        match value {
            AttrValue::String(value) => {
                let id_dom = self.id_dom;
                let value: String = value.into();
                self.driver.inner.dom.set_attr(id_dom, name, &value);
            },
            AttrValue::Computed(computed) => {
                let id_dom = self.id_dom;
                let driver = self.driver.clone();

                self.subscribe(computed, move |value| {
                    let value: String = value.into();
                    driver.inner.dom.set_attr(id_dom, name, &value);
                });

            }
        };

        self
    }

    pub fn id_dom(&self) -> DomId {
        self.id_dom
    }

    pub fn add_child(&self, child_node: impl Into<DomNodeFragment>) {
        let parent_id = self.id_dom;
        let child_node = child_node.into();

        let child_id = child_node.id();
        self.driver.inner.dom.insert_before(self.id_dom, child_id, None);

        let child_node = child_node.convert_to_node(parent_id);
        self.child_node.push(child_node);
    }

    pub fn child(self, child_node: impl Into<DomNodeFragment>) -> Self {
        self.add_child(child_node);
        self
    }

    pub fn on_click(self, on_click: impl Fn() + 'static) -> Self {
        let (callback_id, drop) = self.driver.inner.callback_store.register(move |_data| {
            on_click();
            JsValue::Undefined
        });

        let drop_event = DropResource::new({
            let callback_id = callback_id.clone();
            let driver = self.driver.clone();
            move || {
                driver.inner.dom.callback_remove(self.id_dom, "mousedown", callback_id);
                drop.off();
            }
        });

        self.driver.inner.dom.callback_add(self.id_dom, "mousedown", callback_id);
        self.drop.push(drop_event);

        self
    }

    pub fn on_mouse_enter(self, on_mouse_enter: impl Fn() + 'static) -> Self {
        let (callback_id, drop) = self.driver.inner.callback_store.register(move |_data| {
            on_mouse_enter();
            JsValue::Undefined
        });

        let drop_event = DropResource::new({
            let callback_id = callback_id.clone();
            let driver = self.driver.clone();
            move || {
                driver.inner.dom.callback_remove(self.id_dom, "mouseenter", callback_id);
                drop.off();
            }
        });

        self.driver.inner.dom.callback_add(self.id_dom, "mouseenter", callback_id);
        self.drop.push(drop_event);

        self
    }

    pub fn on_mouse_leave(self, on_mouse_leave: impl Fn() + 'static) -> Self {
        let (callback_id, drop) = self.driver.inner.callback_store.register(move |_data| {
            on_mouse_leave();
            JsValue::Undefined
        });

        let drop_event = DropResource::new({
            let callback_id = callback_id.clone();
            let driver = self.driver.clone();
            move || {
                driver.inner.dom.callback_remove(self.id_dom, "mouseleave", callback_id);
                drop.off();
            }
        });

        self.driver.inner.dom.callback_add(self.id_dom, "mouseleave", callback_id);
        self.drop.push(drop_event);

        self
    }

    pub fn on_input(self, on_input: impl Fn(String) + 'static) -> Self {
        let (callback_id, drop) = self.driver.inner.callback_store.register(move |data| {
            if let JsValue::String(text) = data {
                on_input(text);
            } else {
                log::error!("Invalid data: on_input: {data:?}");
            }

            JsValue::Undefined
        });

        let drop_event = DropResource::new({
            let callback_id = callback_id.clone();
            let driver = self.driver.clone();
            move || {
                driver.inner.dom.callback_remove(self.id_dom, "input", callback_id);
                drop.off();
            }
        });

        self.driver.inner.dom.callback_add(self.id_dom, "input", callback_id);
        self.drop.push(drop_event);

        self
    }

    pub fn on_key_down(self, on_key_down: impl Fn(KeyDownEvent) -> bool + 'static) -> Self {
        let (callback_id, drop) = self.driver.inner.callback_store.register(move |data| {
            match get_key_down_event(data) {
                Ok(event) => {
                    let prevent_default = on_key_down(event);

                    match prevent_default {
                        true => JsValue::True,
                        false => JsValue::False,
                    }
                },
                Err(error) => {
                    log::error!("export_websocket_callback_message -> params decode error -> {error}");
                    JsValue::False
                }
            }
        });

        let drop_event = DropResource::new({
            let callback_id = callback_id.clone();
            let driver = self.driver.clone();
            move || {
                driver.inner.dom.callback_remove(self.id_dom, "keydown", callback_id);
                drop.off();
            }
        });

        self.driver.inner.dom.callback_add(self.id_dom, "keydown", callback_id);
        self.drop.push(drop_event);

        self
    }

    pub fn on_dropfile(self, on_dropfile: impl Fn(DropFileEvent) + 'static) -> Self {
        let (callback_id, drop) = self.driver.inner.callback_store.register(move |data| {
            let params = data
                .convert(|mut params| {

                    let files = params.get_list("files", |mut item| {
                        let name = item.get_string("name")?;
                        let data = item.get_buffer("data")?;

                        Ok(DropFileItem::new(name, data))
                    })?;
                    params.expect_no_more()?;

                    Ok(DropFileEvent::new(files))
                });

            match params {
                Ok(params) => {
                    on_dropfile(params);
                },
                Err(error) => {
                    log::error!("on_dropfile -> params decode error -> {error}");
                }
            };

            JsValue::Undefined
        });

        let drop_event = DropResource::new({
            let callback_id = callback_id.clone();
            let driver = self.driver.clone();
            move || {
                driver.inner.dom.callback_remove(self.id_dom, "drop", callback_id);
                drop.off();
            }
        });

        self.driver.inner.dom.callback_add(self.id_dom, "drop", callback_id);
        self.drop.push(drop_event);

        self
    }

    pub fn hook_key_down(self, on_hook_key_down: impl Fn(KeyDownEvent) -> bool + 'static) -> Self {
        let (callback_id, drop) = self.driver.inner.callback_store.register(move |data| {
            match get_key_down_event(data) {
                Ok(event) => {
                    let prevent_default = on_hook_key_down(event);

                    match prevent_default {
                        true => JsValue::True,
                        false => JsValue::False,
                    }
                },
                Err(error) => {
                    log::error!("export_websocket_callback_message -> params decode error -> {error}");
                    JsValue::False
                }
            }
        });

        let drop_event = DropResource::new({
            let callback_id = callback_id.clone();
            let driver = self.driver.clone();
            move || {
                driver.inner.dom.callback_remove(self.id_dom, "hook_keydown", callback_id);
                drop.off();
            }
        });

        self.driver.inner.dom.callback_add(self.id_dom, "hook_keydown", callback_id);
        self.drop.push(drop_event);

        self
    }

}

impl Drop for DomElement {
    fn drop(&mut self) {
        self.driver.inner.dom.remove_node(self.id_dom);
    }
}


fn get_key_down_event(data: JsValue) -> Result<KeyDownEvent, String> {
    data.convert(|mut params| {
        let key = params.get_string("key")?;
        let code = params.get_string("code")?;
        let alt_key = params.get_bool("altKey")?;
        let ctrl_key = params.get_bool("ctrlKey")?;
        let shift_key = params.get_bool("shiftKey")?;
        let meta_key = params.get_bool("metaKey")?;
        params.expect_no_more()?;

        Ok((key, code, alt_key, ctrl_key, shift_key, meta_key))
    }).map(|(key, code, alt_key, ctrl_key, shift_key, meta_key)| {
        KeyDownEvent {
            key,
            code,
            alt_key,
            ctrl_key,
            shift_key,
            meta_key,
        }
    })
}