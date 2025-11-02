use ltrait::{Action, action::ActionWrapper};

impl<T> ActionExt for T
where
    T: Action,
    <T as Action>::Context: Sync + Send,
{
}

pub trait ActionExt: Action
where
    <Self as Action>::Context: Sync + Send,
{
    fn to_if<Cushion, F, TransF>(self, f: F, transformer: TransF) -> impl Action<Context = Cushion>
    where
        Self: Sized,
        Cushion: Sync + Send,
        F: Fn(&Cushion) -> bool + Send,
        TransF: Fn(&Cushion) -> <Self as Action>::Context + Send,
    {
        ActionIf::new(self, f, transformer)
    }
}

pub struct ActionIf<T, Ctx, F>
where
    T: Action<Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync,
{
    inner: T,

    f: F,
}

impl<Cushion, F, Inner, TransF, Ctx>
    ActionIf<ActionWrapper<Ctx, Inner, TransF, Cushion>, Cushion, F>
where
    F: Fn(&Cushion) -> bool + Send,
    Cushion: Sync + Send,
    TransF: Fn(&Cushion) -> Ctx + Send,
    Inner: Action<Context = Ctx>,
    Ctx: Sync + Send,
{
    pub fn new(inner: Inner, f: F, transformer: TransF) -> Self {
        Self {
            inner: ActionWrapper::new(inner, transformer),
            f,
            // _ctx: PhantomData,
        }
    }
}

impl<T, Ctx, F> Action for ActionIf<T, Ctx, F>
where
    T: Action<Context = Ctx>,
    F: Fn(&Ctx) -> bool + Send,
    Ctx: Sync + Send,
{
    type Context = Ctx;

    fn act(&self, ctx: &Self::Context) -> ltrait::color_eyre::eyre::Result<()> {
        if (self.f)(ctx) {
            self.inner.act(ctx)
        } else {
            Ok(())
        }
    }
}
