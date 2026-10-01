//! Browser tools backed by the loadable TinyComputer module.

#[allow(clippy::module_inception)]
#[cfg(feature = "modules")]
mod browser;
#[cfg(feature = "modules")]
mod browser_open;
<<<<<<< HEAD
mod image_info;
=======
mod security;
>>>>>>> pr-6829

#[cfg(feature = "modules")]
pub use browser::BrowserTool;
#[cfg(feature = "modules")]
pub use browser_open::BrowserOpenTool;
