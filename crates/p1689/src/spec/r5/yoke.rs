#[cfg(feature = "yoke")]
use yoke::Yokeable;

use crate::r5::{DepFile, DepInfo, ModuleDesc, ProvidedModuleDesc, RequiredModuleDesc};

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
