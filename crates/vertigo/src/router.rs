use std::rc::Rc;

use crate::{
    computed::{Client, Value},
    DomDriver, RoutingReceiver,
    utils::BoxRefCell,
};

#[derive(PartialEq)]
pub enum RouterType {
    Hash,
    Browser
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Loading,
    Pushing,
    Popping,
}

#[derive(PartialEq)]
pub struct Router {
    sender: Client,
    receiver: RoutingReceiver,
}

impl Router {
    /// Create new Router which sets route value upon hash/browser change in browser bar.
    /// If callback is provided then it is fired instead.
    pub fn new<T>(rtype: RouterType, driver: &DomDriver, route: Value<T>, callback: Box<dyn Fn(String)>) -> Self
    where
        T: PartialEq + ToString
    {
        let direction = Rc::new(BoxRefCell::new(Direction::Loading));

        let sender = route.to_computed().subscribe({
            let driver = driver.clone();
            let direction = direction.clone();
            move |route| {
                let dir = direction.get(|state| *state);
                match dir {
                    // First change is upon page loading, ignore it but accept further pushes
                    Direction::Loading =>
                        direction.change_no_params(|state| *state = Direction::Pushing),
                    Direction::Pushing =>
                        match rtype {
                            RouterType::Hash => driver.push_hash_location(route.to_string()),
                            RouterType::Browser => driver.push_browser_location(route.to_string()),
                        },
                    _ => ()
                }
            }
        });

        let receiver = driver.on_hash_route_change({
            Box::new(move |url: String| {
                direction.change_no_params(|state| *state = Direction::Popping);
                callback(url);
                direction.change_no_params(|state| *state = Direction::Pushing);
            })
        });

        Self {
            sender,
            receiver,
        }
    }
}
