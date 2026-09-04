//! # Windows Tray Implementation
//!
//! This module implements the system tray functionality for Windows platforms.
//! It creates a message-only window to handle tray icon events and provides
//! a native Windows system tray experience.
//!
//! ## Responsibilities
//! - Creating and managing the system tray icon
//! - Handling tray icon clicks (left-click to show window, right-click for context menu)
//! - Providing a context menu with "Open TouchGrass" and "Quit TouchGrass" options
//! - Communicating with the main application via mpsc channels for thread safety
//!
//! ## Threading Model
//! This implementation runs on a dedicated thread that manages the Windows message loop
//! for the tray icon. Communication with the main application thread happens through
//! message passing channels to ensure thread safety.
//!
//! ## Windows API Usage
//! Uses the windows-sys crate for direct Windows API calls, avoiding unsafe abstractions
//! where possible while still providing access to necessary low-level functionality.
//!
//! ## Platform Specificity
//! This module is only compiled for Windows targets via #[cfg(target_os = "windows")].
//! Non-Windows platforms use a dummy implementation that does nothing.

use crate::platform::tray::ApplicationCommand;
use crate::platform::TrayService;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::ptr;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, POINT};
use windows_sys::Win32::UI::Shell::{NOTIFYICONDATAW, NIF_MESSAGE, NIF_ICON, NIF_TIP, NIM_ADD, NIM_DELETE};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadIconW, PostQuitMessage,
    RegisterClassExW, TranslateMessage, WM_CREATE, WM_DESTROY,
    WM_USER, WNDCLASSEXW,
    IDI_APPLICATION,
    AppendMenuW, CreatePopupMenu, GetCursorPos, SetForegroundWindow, TrackPopupMenu,
    DestroyWindow, DestroyMenu,
    MF_STRING, TPM_RIGHTBUTTON, CS_HREDRAW, CS_VREDRAW,
    PostMessageW, GWLP_USERDATA, SetWindowLongPtrW,
    GetWindowLongPtrW,
    LoadCursorW, IDC_ARROW,
    WM_LBUTTONDOWN, WM_LBUTTONDBLCLK, WM_RBUTTONUP, WM_COMMAND, WM_QUIT,
};
use windows_sys::Win32::Graphics::Gdi::{COLOR_WINDOW, HBRUSH};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use std::ffi::OsStr;
use std::iter;
use std::os::windows::ffi::OsStrExt;

const TRAY_MSG_CALLBACK: u32 = WM_USER + 1;
const TRAY_ICON_ID: usize = 1;

#[derive(Clone)]
struct TrayContext {
    window: HWND,
    command_tx: mpsc::Sender<ApplicationCommand>,
}

/// Windows system tray service implementation
pub struct WindowsTrayService {
    window: Mutex<Option<HWND>>,
    command_tx: mpsc::Sender<ApplicationCommand>,
    _thread: Mutex<Option<thread::JoinHandle<()>>>,
}

impl WindowsTrayService {
    pub fn new(command_tx: mpsc::Sender<ApplicationCommand>) -> Self {
        Self {
            window: Mutex::new(None),
            command_tx,
            _thread: Mutex::new(None),
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let ctx = Box::into_raw(Box::new(TrayContext {
                    window: hwnd,
                    command_tx: Self::get_command_tx(hwnd),
                }));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, ctx as isize);
                0
            }
            TRAY_MSG_CALLBACK => {
                match lparam as u32 {
                    WM_LBUTTONDOWN => {
                        // Left click - show window
                        if let Some(ctx) = Self::get_context(hwnd) {
                            let _ = ctx.command_tx.send(ApplicationCommand::ShowWindow);
                        }
                    }
                    WM_LBUTTONDBLCLK => {
                        // Left double-click - show window
                        if let Some(ctx) = Self::get_context(hwnd) {
                            let _ = ctx.command_tx.send(ApplicationCommand::ShowWindow);
                        }
                    }
                    WM_RBUTTONUP => {
                        // Right click - show context menu
                        if let Some(ctx) = Self::get_context(hwnd) {
                            Self::show_context_menu(hwnd, &ctx);
                        }
                    }
                    _ => {}
                }
                0
            }
            WM_COMMAND => {
                match (wparam & 0xFFFF) as u32 { // LOWORD equivalent
                    1001 => { // Open menu item
                        if let Some(ctx) = Self::get_context(hwnd) {
                            let _ = ctx.command_tx.send(ApplicationCommand::ShowWindow);
                        }
                    }
                    1002 => { // Quit menu item
                        if let Some(ctx) = Self::get_context(hwnd) {
                            let _ = ctx.command_tx.send(ApplicationCommand::Quit);
                        }
                    }
                    _ => {}
                }
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    fn get_context(hwnd: HWND) -> Option<TrayContext> {
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut TrayContext;
        if ptr.is_null() {
            None
        } else {
            let ctx = unsafe { &*ptr };
            Some(ctx.clone())
        }
    }

    fn get_command_tx(hwnd: HWND) -> mpsc::Sender<ApplicationCommand> {
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut TrayContext;
        if ptr.is_null() {
            // Fallback - create a new channel (not ideal but prevents crashes)
            let (tx, _) = mpsc::channel();
            tx
        } else {
            let ctx = unsafe { &*ptr };
            ctx.command_tx.clone()
        }
    }

    unsafe fn show_context_menu(hwnd: HWND, _ctx: &TrayContext) {
        // Get cursor position
        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);

        // Create popup menu
        let hmenu = CreatePopupMenu();
        AppendMenuW(hmenu, MF_STRING, 1001, to_wstring("Open TouchGrass").as_ptr());
        AppendMenuW(hmenu, MF_STRING, 1002, to_wstring("Quit TouchGrass").as_ptr());

        // Show menu and get selection
        SetForegroundWindow(hwnd);
        TrackPopupMenu(
            hmenu,
            TPM_RIGHTBUTTON,
            pt.x,
            pt.y,
            0,
            hwnd,
            ptr::null(),
        );

        // Cleanup
        DestroyMenu(hmenu);
    }

    unsafe fn create_tray_icon(hwnd: HWND) {
        let hinstance = GetModuleHandleW(ptr::null());

        // Load a default icon (you would replace this with your actual icon)
        let hicon = LoadIconW(hinstance, IDI_APPLICATION);

        let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = TRAY_ICON_ID as u32;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = TRAY_MSG_CALLBACK;
        nid.hIcon = hicon;

        // Set tooltip
        let tooltip = to_wstring("TouchGrass");
        nid.szTip.copy_from_slice(&tooltip[..tooltip.len().min(128)]);

        // Use the correct path for Shell_NotifyIconW
        windows_sys::Win32::UI::Shell::Shell_NotifyIconW(NIM_ADD, &nid);
    }

    unsafe fn remove_tray_icon(hwnd: HWND) {
        let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = TRAY_ICON_ID as u32;
        // Use the correct path for Shell_NotifyIconW
        windows_sys::Win32::UI::Shell::Shell_NotifyIconW(NIM_DELETE, &nid);
    }

    // Private helper method to run the message loop
    fn run_message_loop(command_tx: mpsc::Sender<ApplicationCommand>) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let hinstance = GetModuleHandleW(ptr::null());

            // Register window class
            let class_name = to_wstring("TouchGrassTrayWindow");
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: LoadIconW(hinstance, IDI_APPLICATION),
                hCursor: LoadCursorW(0, IDC_ARROW),
                hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: LoadIconW(hinstance, IDI_APPLICATION),
                ..std::mem::zeroed()
            };

            if RegisterClassExW(&wc) == 0 {
                return Err("Failed to register window class".into());
            }

            // Create message-only window
            let window = CreateWindowExW(
                0,                                      // dwExStyle
                class_name.as_ptr(),                    // lpClassName
                ptr::null(),                            // lpWindowName
                0,                                      // dwStyle
                0,                                      // x
                0,                                      // y
                0,                                      // nWidth
                0,                                      // nHeight
                0 as HWND,                              // hWndParent
                0,                                      // hMenu
                hinstance,                              // hInstance
                ptr::null(),                            // lpParam
            );

            if window == 0 {
                return Err("Failed to create window".into());
            }

            // Store context in window
            let context = TrayContext {
                window,
                command_tx: command_tx.clone(),
            };

            let context_ptr = Box::into_raw(Box::new(context));
            SetWindowLongPtrW(window, GWLP_USERDATA, context_ptr as isize);

            // Create tray icon
            Self::create_tray_icon(window);

            // Message loop
            let mut msg = std::mem::MaybeUninit::uninit();
            loop {
                // Process window messages
                if GetMessageW(msg.as_mut_ptr(), 0, 0, 0) > 0 {
                    TranslateMessage(msg.as_ptr());
                    DispatchMessageW(msg.as_ptr());
                } else {
                    break;
                }
            }

            // Cleanup
            Self::remove_tray_icon(window);
            DestroyWindow(window);
        }

        Ok(())
    }
}

impl TrayService for WindowsTrayService {
    fn new(command_tx: mpsc::Sender<ApplicationCommand>) -> Self {
        Self {
            window: Mutex::new(None),
            command_tx,
            _thread: Mutex::new(None),
        }
    }

    fn start(&self) {
        // Clone the sender for the thread
        let command_tx = self.command_tx.clone();

        // Spawn the message loop thread
        let thread_handle = thread::spawn(move || {
            let _ = Self::run_message_loop(command_tx);
        });

        // Store the thread handle
        *self._thread.lock().unwrap() = Some(thread_handle);
    }

    fn stop(&self) {
        // Post a quit message to the window thread to make it exit
        if let Some(window) = *self.window.lock().unwrap() {
            unsafe {
                PostMessageW(window, WM_QUIT, 0, 0);
            }
        }

        // Wait for the thread to finish
        if let Some(thread_handle) = self._thread.lock().unwrap().take() {
            let _ = thread_handle.join();
        }
    }
}

// Helper function to encode string to wide characters
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(iter::once(0))
        .collect()
}

// Dummy implementation for non-Windows platforms
#[cfg(not(target_os = "windows"))]
mod dummy {
    use super::*;
    use std::sync::{Arc, Mutex};

    pub struct DummyTrayService {
        command_tx: mpsc::Sender<ApplicationCommand>,
    }

    impl TrayService for DummyTrayService {
        fn new(command_tx: mpsc::Sender<ApplicationCommand>) -> Self {
            Self {
                command_tx,
            }
        }

        fn start(&self) {
            // No-op for non-Windows
        }

        fn stop(&self) {
            // No-op for non-Windows
        }
    }
}