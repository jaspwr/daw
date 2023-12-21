use std::{cell::RefCell, rc::Rc};

use crate::global::Globals;

use super::BoundingBoxRef;

#[macro_export]
macro_rules! bind_reactives {
    (
        $element: ident  {
            $(
                [ $( $reactive_dependency: ident ),* ] => $callback: expr
            ),* $(,)?
        }
    ) => {
        {
            let element: ElementRef = $element.clone();

            $({
                $(let $reactive_dependency = $reactive_dependency.clone().get_copy();)*

                element.mutate(Box::new(move |element: &mut Element| {
                    $callback(element, $( $reactive_dependency ),*);
                }));
            })*

            $({
                $(let $reactive_dependency = $reactive_dependency.clone();)*

                let callback_ = {
                    $(let $reactive_dependency = $reactive_dependency.clone();)*

                    Rc::new(move |element: &mut Element| {
                        $(let $reactive_dependency = $reactive_dependency.clone().get_copy();)*

                        $callback(element, $( $reactive_dependency ),*);
                    })
                };

                $(
                    let callback__ = callback_.clone();
                    element.subscribe_mutation_to_reactive(
                        &$reactive_dependency,
                        Box::new(move |element: &mut Element, _: &_| {
                            callback__.clone()(element);
                        })
                    );
                )*

            })*
        }
    };
}

pub struct ElementInitDeps<'a> {
    pub gl: &'a glow::Context,
    pub globals: &'a mut Globals,
    pub needs_rerender: Rc<RefCell<bool>>,
    pub frame_bounding_box: BoundingBoxRef,
}
