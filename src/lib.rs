//! High level bindings to [libxdo](http://www.semicomplete.com/files/xdotool/docs/html/)

#![warn(missing_docs, clippy::pedantic)]
#![expect(clippy::missing_errors_doc)]

extern crate libxdo_sys as sys;

use std::error::Error;
use std::ffi::{CStr, CString, NulError};
use std::fmt;
use std::ptr::NonNull;
use std::num::TryFromIntError;

use versions::Versioning;

/// The main handle type which provides access to the various operations.
pub struct XDo {
    handle: NonNull<sys::xdo_t>,
}

unsafe impl Send for XDo {}
unsafe impl Sync for XDo {}

#[allow(missing_docs)]
pub type Window = sys::Window;

/// An error that can happen when trying to create an `XDo` instance.
#[derive(Debug)]
pub enum CreationError {
    /// The provided string parameter had an interior null byte in it.
    Nul(NulError),
    /// Libxdo failed to create an instance. No further information available.
    Ffi,
}

/// Search mode
#[derive(Debug, Default)]
pub enum SearchRequire {
    #[default]
    /// Any success will keep the window in search results
    Any,
    /// Any failure will skip the window
    All,
}

/// Search parameters
#[derive(Debug, Default)]
pub struct Search {
    /// pattern to test against a window title
    pub title: Option<String>,
    /// pattern to test against a window class
    pub window_class: Option<String>,
    /// pattern to test against a window class name
    pub window_class_name: Option<String>,
    /// pattern to test against a window name
    pub window_name: Option<String>,
    /// pattern to test against a window role
    pub window_role: Option<String>,
    /// window pid (From window atom _NET_WM_PID)
    pub pid: Option<i32>,
    /// depth of search. 1 means only toplevel windows
    pub max_depth: Option<isize>,
    /// boolean; set true to search only visible windows
    pub only_visible: bool,
    /// what screen to search, if any. If none given, search all screens 
    pub screen: Option<i32>,

    /// Should the tests be 'and' or 'or' ? If 'and', any failure will skip
    /// the window. If 'or', any success will keep the window in search results.
    pub require: SearchRequire,
    
    /// What desktop to search, if any. If none given, search all screens.
    pub desktop: Option<usize>,
    /// How many results to return? If 0, return all.
    pub limit: usize,
}

impl fmt::Display for CreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CreationError::Nul(ref err) => {
                write!(
                    f,
                    "Failed to create XDo instance: Nul byte in argument: {err}",
                )
            }
            CreationError::Ffi => write!(f, "Libxdo failed to create an instance."),
        }
    }
}

impl Error for CreationError {
    fn description(&self) -> &str {
        match *self {
            CreationError::Nul(_) => "libxdo creation error: Nul byte in argument",
            CreationError::Ffi => "libxdo creation error: Ffi error",
        }
    }
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            CreationError::Nul(ref err) => Some(err),
            CreationError::Ffi => None,
        }
    }
}

impl From<NulError> for CreationError {
    fn from(err: NulError) -> CreationError {
        CreationError::Nul(err)
    }
}

/// An error that can happen while executing an operation.
#[derive(Debug)]
pub enum OpError {
    /// The provided string parameter had an interior null byte in it.
    Nul(NulError),
    /// Integer conversion error.
    Int(TryFromIntError),
    /// Library version not supported.
    Ver(),
    /// Libxdo failed, returning an error code.
    Ffi(i32),
}

impl fmt::Display for OpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OpError::Nul(ref err) => {
                write!(f, "Xdo operation failed: Nul byte in argument: {err}")
            }
            OpError::Int(ref err) => {
                write!(f, "Xdo operation failed: Integer conversion error: {err}")
            }
            OpError::Ver() => {
                write!(f, "Xdo operation failed: Library version not supported")
            }
            OpError::Ffi(code) => write!(f, "Xdo operation failed. Error code {code}."),
        }
    }
}

impl Error for OpError {
    fn description(&self) -> &str {
        match *self {
            OpError::Nul(_) => "xdo operation failure: Nul byte in argument",
            OpError::Int(_) => "xdo operation failure: Integer conversion error",
            OpError::Ver() => "xdo operation failure: Library version not supported",
            OpError::Ffi(_) => "xdo operation failure: Ffi error",
        }
    }
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            OpError::Nul(ref err) => Some(err),
            OpError::Int(ref err) => Some(err),
            OpError::Ver() | OpError::Ffi(_) => None,
        }
    }
}

impl From<NulError> for OpError {
    fn from(err: NulError) -> Self {
        OpError::Nul(err)
    }
}

impl From<TryFromIntError> for OpError {
    fn from(err: TryFromIntError) -> OpError {
        OpError::Int(err)
    }
}

/// Result of an `XDo` operation.
pub type OpResult = Result<(), OpError>;

macro_rules! xdo (
    ($fncall: expr) => {
        unsafe {
            match $fncall {
                0 => Ok(()),
                code => Err(OpError::Ffi(code))
            }
        }
    }
);

impl XDo {
    /// Creates a new `XDo` instance.
    ///
    /// # Parameters
    ///
    /// display - An optional string display name, such as `":0"`. If `None`, uses `$DISPLAY`.
    ///
    /// # Returns
    ///
    /// Returns a new `XDo` instance, or a `CreationError` on error.
    pub fn new(display: Option<&str>) -> Result<XDo, CreationError> {
        let c_string;
        let display = match display {
            Some(display) => {
                c_string = CString::new(display)?;
                c_string.as_ptr()
            }
            None => ::std::ptr::null(),
        };
        let handle = unsafe { sys::xdo_new(display) };
        match NonNull::new(handle) {
            Some(handle) => Ok(Self { handle }),
            None => Err(CreationError::Ffi),
        }
    }
    /// Moves the mouse to the specified position.
    pub fn move_mouse(&self, x: i32, y: i32, screen: i32) -> OpResult {
        xdo!(sys::xdo_move_mouse(self.handle.as_ptr(), x, y, screen))
    }
    /// Moves the mouse relative to the current position.
    pub fn move_mouse_relative(&self, x: i32, y: i32) -> OpResult {
        xdo!(sys::xdo_move_mouse_relative(self.handle.as_ptr(), x, y))
    }
    /// Does a mouse click.
    pub fn click(&self, button: i32) -> OpResult {
        xdo!(sys::xdo_click_window(
            self.handle.as_ptr(),
            sys::CURRENTWINDOW,
            button
        ))
    }
    /// Holds a mouse button down.
    pub fn mouse_down(&self, button: i32) -> OpResult {
        xdo!(sys::xdo_mouse_down(
            self.handle.as_ptr(),
            sys::CURRENTWINDOW,
            button
        ))
    }
    /// Releases a mouse button.
    pub fn mouse_up(&self, button: i32) -> OpResult {
        xdo!(sys::xdo_mouse_up(
            self.handle.as_ptr(),
            sys::CURRENTWINDOW,
            button
        ))
    }
    /// Types the specified text.
    pub fn enter_text(&self, text: &str, delay_microsecs: u32) -> OpResult {
        let string = CString::new(text)?;
        xdo!(sys::xdo_enter_text_window(
            self.handle.as_ptr(),
            sys::CURRENTWINDOW,
            string.as_ptr(),
            delay_microsecs
        ))
    }
    /// Does the specified key sequence.
    pub fn send_keysequence(&self, window: Option<Window>, sequence: &str, delay_microsecs: u32) -> OpResult {
        let string = CString::new(sequence)?;
        xdo!(sys::xdo_send_keysequence_window(
            self.handle.as_ptr(),
            window.unwrap_or(sys::CURRENTWINDOW),
            string.as_ptr(),
            delay_microsecs
        ))
    }
    /// Releases the specified key sequence.
    pub fn send_keysequence_up(&self, sequence: &str, delay_microsecs: u32) -> OpResult {
        let string = CString::new(sequence)?;
        xdo!(sys::xdo_send_keysequence_window_up(
            self.handle.as_ptr(),
            sys::CURRENTWINDOW,
            string.as_ptr(),
            delay_microsecs
        ))
    }
    /// Presses the specified key sequence down.
    pub fn send_keysequence_down(&self, sequence: &str, delay_microsecs: u32) -> OpResult {
        let string = CString::new(sequence)?;
        xdo!(sys::xdo_send_keysequence_window_down(
            self.handle.as_ptr(),
            sys::CURRENTWINDOW,
            string.as_ptr(),
            delay_microsecs
        ))
    }
    /// Searches for windows.
    pub fn search_windows(&self, search: Search) -> Result<Vec<Window>, OpError> {
        let mut searchmask = 0;

        if search.title.is_some() { searchmask |= sys::SEARCH_TITLE; }
        if search.window_class.is_some() { searchmask |= sys::SEARCH_CLASS; }
        if search.window_class_name.is_some() { searchmask |= sys::SEARCH_CLASSNAME; }
        if search.window_name.is_some() { searchmask |= sys::SEARCH_NAME; }
        if search.window_role.is_some() { searchmask |= sys::SEARCH_ROLE; }
        if search.pid.is_some() { searchmask |= sys::SEARCH_PID; }
        if search.only_visible { searchmask |= sys::SEARCH_ONLYVISIBLE; }
        if search.screen.is_some() { searchmask |= sys::SEARCH_SCREEN; }
        if search.desktop.is_some() { searchmask |= sys::SEARCH_DESKTOP; }

        let searchmask = searchmask;
        let c_title = CString::new(search.title.unwrap_or_default())?;
        let c_winclass = CString::new(search.window_class.unwrap_or_default())?;
        let c_winclassname = CString::new(search.window_class_name.unwrap_or_default())?;
        let c_winname = CString::new(search.window_name.unwrap_or_default())?;
        let c_winrole = CString::new(search.window_role.unwrap_or_default())?;
        let require = match search.require {
            SearchRequire::All => sys::SEARCH_ALL,
            SearchRequire::Any => sys::SEARCH_ANY,
        };

        let version = Versioning::new(unsafe {
            CStr::from_ptr(sys::xdo_version()).to_str().map_err(|_| OpError::Ver())?
        }).ok_or(OpError::Ver())?;

        let c_search = if version >= Versioning::new("3.20210804.1").ok_or(OpError::Ver())? {
            sys::Union_xdo_search {
                v3_20210804_1: sys::Struct_xdo_search_3_20210804_1 {
                    title: c_title.as_ptr(),
                    winclass: c_winclass.as_ptr(),
                    winclassname: c_winclassname.as_ptr(),
                    winname: c_winname.as_ptr(),
                    winrole: c_winrole.as_ptr(),
                    pid: search.pid.unwrap_or_default(),
                    max_depth: search.max_depth.unwrap_or(-1).try_into()?,
                    only_visible: search.only_visible.into(),
                    screen: search.screen.unwrap_or_default(),
                    require,
                    searchmask,
                    desktop: search.desktop.unwrap_or_default().try_into()?,
                    limit: search.limit.try_into()?,
                }
            }
        } else if version >= Versioning::new("3.20150503.1").ok_or(OpError::Ver())? {
            if searchmask & sys::SEARCH_ROLE != 0 {
                Err(OpError::Ver())?;
            }

            sys::Union_xdo_search {
                v3_20150503_1: sys::Struct_xdo_search_3_20150503_1 {
                    title: c_title.as_ptr(),
                    winclass: c_winclass.as_ptr(),
                    winclassname: c_winclassname.as_ptr(),
                    winname: c_winname.as_ptr(),
                    pid: search.pid.unwrap_or_default(),
                    max_depth: search.max_depth.unwrap_or(-1).try_into()?,
                    only_visible: search.only_visible.into(),
                    screen: search.screen.unwrap_or_default(),
                    require,
                    searchmask,
                    desktop: search.desktop.unwrap_or_default().try_into()?,
                    limit: search.limit.try_into()?,
                }
            }
        } else {
            Err(OpError::Ver())?
        };
        let mut windowlist_ret: *mut sys::Window = std::ptr::null_mut();
        let mut nwindows_ret: sys::c_uint = 0;

        xdo!(sys::xdo_search_windows(
            self.handle.as_ptr(),
            &raw const c_search,
            &raw mut windowlist_ret,
            &raw mut nwindows_ret,
        ))?;

        Ok(unsafe {
            std::slice::from_raw_parts(windowlist_ret,
                nwindows_ret.try_into().unwrap_or(usize::MAX)) }.to_vec())
    }
}

impl Drop for XDo {
    fn drop(&mut self) {
        unsafe {
            sys::xdo_free(self.handle.as_ptr());
        }
    }
}
