use crate::avm1::{Avm1, Value};
use crate::context::UpdateContext;
pub use crate::display_object::{DisplayObject, TDisplayObject};
use gc_arena::{Collect, GcCell, MutationContext};

#[collect(no_drop)]
#[derive(Clone, Copy, Collect, Debug)]
pub struct FocusTracker<'gc>(GcCell<'gc, Option<DisplayObject<'gc>>>);

impl<'gc> FocusTracker<'gc> {
    pub fn new(gc_context: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(gc_context, None))
    }

    pub fn get(&self) -> Option<DisplayObject<'gc>> {
        *self.0.read()
    }

    pub fn set(
        &self,
        focused_element: Option<DisplayObject<'gc>>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        let old = std::mem::replace(&mut *self.0.write(context.gc_context), focused_element);

        if old.is_none() && focused_element.is_none() {
            // We didn't have anything, we still don't, no change.
            return;
        }
        if old.is_some() == focused_element.is_some()
            && old.unwrap().as_ptr() == focused_element.unwrap().as_ptr()
        {
            // We're setting it to the same object as before, no change.
            return;
        }

        if let Some(old) = old {
            old.on_focus_changed(false);
        }
        if let Some(new) = focused_element {
            new.on_focus_changed(true);
        }

        let level0 = context.levels.get(&0).copied().unwrap();
        Avm1::notify_system_listeners(
            level0,
            context.swf.version(),
            context,
            "Selection",
            "onSetFocus",
            &[
                old.map(|v| v.object()).unwrap_or(Value::Null),
                focused_element.map(|v| v.object()).unwrap_or(Value::Null),
            ],
        );
    }
}