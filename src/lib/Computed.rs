use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    BoxRefCell::BoxRefCell,
    Dependencies::Dependencies,
    Client::Client,
    RefreshToken::RefreshToken,
    GraphId::GraphId,
};




pub struct ComputedInner<T: 'static> {
    deps: Dependencies,
    getValueFromParent: Box<dyn Fn() -> Rc<T> + 'static>,
    id: GraphId,
    isFreshCell: Rc<BoxRefCell<bool>>,
    valueCell: BoxRefCell<Rc<T>>,
}

impl<T> Drop for ComputedInner<T> {
    fn drop(&mut self) {
        self.deps.removeRelation(&self.id);
    }
}



pub struct Computed<T: 'static> {
    inner: Rc<ComputedInner<T>>,
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            inner: self.inner.clone()
        }
    }
}

impl<T: 'static> Computed<T> {
    pub fn new<F: Fn() -> Rc<T> + 'static>(deps: Dependencies, getValue: F) -> Computed<T> {

        let id = GraphId::new();
        let isFreshCell = Rc::new(BoxRefCell::new(true));

        let getValue = deps.wrapGetValue(getValue, id.clone());

        deps.registerRefreshToken(id.clone(), RefreshToken::newComputed(isFreshCell.clone()));

        let value = getValue();

        Computed {
            inner: Rc::new(ComputedInner {
                deps,
                getValueFromParent: getValue,
                id,
                isFreshCell: isFreshCell,
                valueCell: BoxRefCell::new(value),
            })
        }
    }

    pub fn getId(&self) -> GraphId {
        self.inner.id.clone()
    }

    pub fn getValue(&self) -> Rc<T> {
        let inner = self.inner.as_ref();
        let selfId = inner.id.clone();
        let deps = inner.deps.clone();

        deps.reportDependenceInStack(selfId);

        let shouldRecalculate = {
            self.inner.isFreshCell.changeNoParams(|state|{
                let shouldRecalculate = *state == false;
                *state = true;
                shouldRecalculate
            })
        };

        let newValue = if shouldRecalculate {
            let ComputedInner { getValueFromParent, .. } = self.inner.as_ref();
            let result = getValueFromParent();
            Some(result)
        } else {
            None
        };

        let result = inner.valueCell.change(newValue, |state, newValue| {
            if let Some(value) = newValue {
                *state = value;
            }

            (*state).clone()
        });

        result
    }

    pub fn subscribe<F: Fn(&T) + 'static>(self, call: F) -> Client {
        let client = Client::new(self.inner.deps.clone(), self.clone(), call);
        client
    }

    pub fn from2<A: Debug, B: Debug>(
        a: Computed<A>,
        b: Computed<B>,
        calculate: fn(&A, &B) -> T
    ) -> Computed<T> {
        let deps = a.inner.deps.clone();

        Computed::new(deps, move || {
            let aValue = a.getValue();
            let bValue = b.getValue();

            let result = calculate(aValue.as_ref(), bValue.as_ref());

            Rc::new(result)
        })
    }

    pub fn map<K>(self, fun: fn(&T) -> K) -> Computed<K> {
        let deps = self.inner.deps.clone();

        Computed::new(deps, move || {
            let value = self.getValue();
            let result = fun(value.as_ref());
            Rc::new(result)
        })
    }
}

