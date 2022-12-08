use std::rc::Rc;

use crate::{
    computed::{Client, Value, DropResource, context::Context},
    struct_mut::ValueMut, get_driver, Computed,
};

struct HashSubscriptions {
    _sender: Client,
    _receiver: DropResource,
}

#[derive(Clone)]
pub struct Router<T: Clone + ToString + From<String> + PartialEq + 'static> {
    route_value: Value<T>,
    pub route: Computed<T>,
    _subscriptions: Rc<HashSubscriptions>,
}

impl<T: Clone + ToString + From<String> + PartialEq + 'static> PartialEq for Router<T> {
    fn eq(&self, other: &Self) -> bool {
        self.route_value.id() == other.route_value.id()
    }
}

/// Router based on hash part of current location.
///
/// ```rust
/// use vertigo::{dom, Computed, Value, DomElement};
/// use vertigo::router::Router;
///
/// #[derive(Clone, PartialEq, Debug)]
/// pub enum Route {
///     Page1,
///     Page2,
///     NotFound,
/// }
///
/// impl Route {
///     pub fn new(path: &str) -> Route {
///         match path {
///             "" | "/" | "/page1" => Self::Page1,
///             "page2" => Self::Page2,
///             _ => Self::NotFound,
///         }
///     }
/// }
///
/// impl ToString for Route {
///     fn to_string(&self) -> String {
///         match self {
///             Self::Page1 => "",
///             Self::Page2 => "page2",
///             Self::NotFound => "",
///         }.to_string()
///     }
/// }
///
/// impl From<String> for Route {
///     fn from(url: String) -> Self {
///         Route::new(url.as_str())
///     }
/// }
///
/// #[derive(Clone)]
/// pub struct State {
///     route: Router<Route>,
/// }
///
/// impl State {
///     pub fn component() -> DomElement {
///         let route = Router::new_hash_router();
///
///         let state = State {
///             route,
///         };
///
///         render(state)
///     }
/// }
///
/// fn render(state: State) -> DomElement {
///     dom! {
///         <div>
///             "..."
///         </div>
///     }
/// }
/// ```
impl<T: Clone + ToString + From<String> + PartialEq + 'static> Router<T> {    
    /// Create new Router which sets route value upon hash change in browser bar.
    /// If callback is provided then it is fired instead.
    pub fn new_hash_router() -> Router<T> {
        Self::new(false)
    }

    pub fn new_history_router() -> Router<T> {
        Self::new(true)
    }

    fn new(use_history_api: bool) -> Self {
        let driver = get_driver();
        let route_value: Value<T> = match use_history_api {
            false => Value::new(T::from(driver.inner.api.get_hash_location())),
            true => Value::new(T::from(driver.inner.api.get_history_location())),
        };

        let block_subscrition = Rc::new(ValueMut::new(true));

        let sender = route_value.to_computed().subscribe({
            let driver = driver.clone();
            let block_subscrition = block_subscrition.clone();
            move |route| {
                if block_subscrition.get() {
                    return;
                }

                match use_history_api {
                    false => driver.inner.api.push_hash_location(&route.to_string()),
                    true => driver.inner.api.push_history_location(&route.to_string()),
                };
            }
        });

        let callback = {
            let route = route_value.clone();
            let block_subscrition = block_subscrition.clone();

            Box::new(move |url: String| {
                block_subscrition.set(true);
                route.set_value_and_compare(T::from(url));
                block_subscrition.set(false);
            })
        };

        let receiver = match use_history_api {
            false => driver.inner.api.on_hash_change(callback),
            true => driver.inner.api.on_history_change(callback),
        };

        block_subscrition.set(false);

        let route = route_value.to_computed();

        Self {
            route_value,
            route,
            _subscriptions: Rc::new(HashSubscriptions {
                _sender: sender,
                _receiver: receiver
            })
        }
    }

    pub fn set(&self, value: T) {
        self.route_value.set_value_and_compare(value);
    }

    pub fn get(&self, context: &Context) -> T {
        self.route_value.get(context)
    }
}
