use alloc::{borrow::Cow, sync::Arc};

use yoke::{Yoke, Yokeable};

use crate::r5::{DepFile, DepInfo, ModuleDesc, ProvidedModuleDesc, RequiredModuleDesc};

pub type DepInfoCart = Arc<dyn AsRef<[u8]> + Send + Sync + 'static>;
#[allow(clippy::module_name_repetitions)]
pub type DepInfoYoke = Yoke<DepInfo<'static>, DepInfoCart>;

#[derive(Clone)]
#[repr(transparent)]
#[allow(clippy::module_name_repetitions)]
pub struct DepInfoNameYoke {
    yoke: Yoke<Cow<'static, str>, DepInfoCart>,
}
impl core::fmt::Debug for DepInfoNameYoke {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.yoke.get(), f)
    }
}
impl core::hash::Hash for DepInfoNameYoke {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: core::hash::Hasher,
    {
        self.yoke.get().hash(state);
    }
}
impl PartialEq for DepInfoNameYoke {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.yoke.get().eq(other.yoke.get())
    }
}
impl Eq for DepInfoNameYoke {}
impl PartialOrd for DepInfoNameYoke {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DepInfoNameYoke {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.yoke.get().cmp(other.yoke.get())
    }
}

pub trait DepInfoYokeExt: self::private::Sealed {
    fn provides(&self) -> impl Iterator<Item = DepInfoNameYoke>;
    fn requires(&self) -> impl Iterator<Item = DepInfoNameYoke>;
}
impl DepInfoYokeExt for DepInfoYoke {
    #[inline]
    fn provides(&self) -> impl Iterator<Item = DepInfoNameYoke> {
        self.get().provides.iter().map(|require| DepInfoNameYoke {
            yoke: Yoke::attach_to_cart(Arc::clone(self.backing_cart()), |_| unsafe {
                Yokeable::make(require.desc.logical_name())
            }),
        })
    }

    #[inline]
    fn requires(&self) -> impl Iterator<Item = DepInfoNameYoke> {
        self.get().requires.iter().map(|require| DepInfoNameYoke {
            yoke: Yoke::attach_to_cart(Arc::clone(self.backing_cart()), |_| unsafe {
                Yokeable::make(require.desc.logical_name())
            }),
        })
    }
}

#[cfg(feature = "yoke")]
unsafe impl<'a> Yokeable<'a> for DepFile<'static> {
    type Output = DepFile<'a>;

    #[inline]
    fn transform(&'a self) -> &'a Self::Output {
        self
    }

    #[inline]
    fn transform_owned(self) -> Self::Output {
        self
    }

    #[inline]
    unsafe fn make(from: Self::Output) -> Self {
        core::mem::transmute(from)
    }

    #[inline]
    fn transform_mut<F>(&'a mut self, f: F)
    where
        F: 'static + for<'b> FnOnce(&'b mut Self::Output),
    {
        let this = unsafe { core::mem::transmute::<&'a mut Self, &'a mut Self::Output>(self) };
        f(this);
    }
}

#[cfg(feature = "yoke")]
unsafe impl<'i> Yokeable<'i> for DepInfo<'static> {
    type Output = DepInfo<'i>;

    #[inline]
    fn transform(&'i self) -> &'i Self::Output {
        self
    }

    #[inline]
    fn transform_owned(self) -> Self::Output {
        self
    }

    #[inline]
    unsafe fn make(from: Self::Output) -> Self {
        core::mem::transmute(from)
    }

    #[inline]
    fn transform_mut<F>(&'i mut self, f: F)
    where
        F: 'static + for<'b> FnOnce(&'b mut Self::Output),
    {
        let this = unsafe { core::mem::transmute::<&mut Self, &mut Self::Output>(self) };
        f(this);
    }
}

#[cfg(feature = "yoke")]
unsafe impl<'i> Yokeable<'i> for ModuleDesc<'static> {
    type Output = ModuleDesc<'i>;

    #[inline]
    fn transform(&'i self) -> &'i Self::Output {
        self
    }

    #[inline]
    fn transform_owned(self) -> Self::Output {
        self
    }

    #[inline]
    unsafe fn make(from: Self::Output) -> Self {
        core::mem::transmute(from)
    }

    #[inline]
    fn transform_mut<F>(&'i mut self, f: F)
    where
        F: 'static + for<'b> FnOnce(&'b mut Self::Output),
    {
        let this = unsafe { core::mem::transmute::<&mut Self, &mut Self::Output>(self) };
        f(this);
    }
}

#[cfg(feature = "yoke")]
unsafe impl<'i> Yokeable<'i> for ProvidedModuleDesc<'static> {
    type Output = ProvidedModuleDesc<'i>;

    #[inline]
    fn transform(&'i self) -> &'i Self::Output {
        self
    }

    #[inline]
    fn transform_owned(self) -> Self::Output {
        self
    }

    #[inline]
    unsafe fn make(from: Self::Output) -> Self {
        core::mem::transmute(from)
    }

    #[inline]
    fn transform_mut<F>(&'i mut self, f: F)
    where
        F: 'static + for<'b> FnOnce(&'b mut Self::Output),
    {
        let this = unsafe { core::mem::transmute::<&mut Self, &mut Self::Output>(self) };
        f(this);
    }
}

#[cfg(feature = "yoke")]
unsafe impl<'i> Yokeable<'i> for RequiredModuleDesc<'static> {
    type Output = RequiredModuleDesc<'i>;

    #[inline]
    fn transform(&'i self) -> &'i Self::Output {
        self
    }

    #[inline]
    fn transform_owned(self) -> Self::Output {
        self
    }

    #[inline]
    unsafe fn make(from: Self::Output) -> Self {
        core::mem::transmute(from)
    }

    #[inline]
    fn transform_mut<F>(&'i mut self, f: F)
    where
        F: 'static + for<'b> FnOnce(&'b mut Self::Output),
    {
        let this = unsafe { core::mem::transmute::<&mut Self, &mut Self::Output>(self) };
        f(this);
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::DepInfoYoke {}
}
