// cargo +stable-i686-pc-windows-gnu build --release

#![crate_type = "cdylib"]

extern crate libc;
use libc::c_char;
use std::ffi::CStr;
use std::i64;

extern crate sciter;

#[no_mangle]
pub extern "cdecl" fn foo(handle: *const c_char) -> () {
	let c_str: &CStr = unsafe { CStr::from_ptr(handle) };
	let hex_string: &str = c_str.to_str().unwrap();
	let window_handle = i64::from_str_radix(hex_string, 16).unwrap();
  let hwnd = window_handle as sciter::types::HWINDOW;
  hook_messages(hwnd);
  let mut frame = sciter::Window::attach(hwnd);
	frame.load_html(include_bytes!("../index.htm"), None);
	frame.expand(false);
}

fn hook_messages(hwnd: sciter::types::HWINDOW) {
	use sciter::types::*;

	#[link(name="user32")]
	extern "system"
	{
		fn SetWindowLongW(hwnd: HWINDOW, index: i32, new_data: WndProc) -> WndProc;
		fn CallWindowProcW(prev: WndProc, hwnd: HWINDOW, msg: UINT, wp: WPARAM, lp: LPARAM) -> LRESULT;
	}

	type WndProc = extern "system" fn (hwnd: HWINDOW, msg: UINT, wp: WPARAM, lp: LPARAM) -> LRESULT;
	type PrevProcs = std::collections::HashMap<HWINDOW, WndProc>;

	thread_local! {
		static PREV_PROC: std::cell::RefCell<PrevProcs> = Default::default();
	}

	// https://sciter.com/developers/embedding-principles/
	extern "system" fn wnd_proc(hwnd: HWINDOW, msg: UINT, wp: WPARAM, lp: LPARAM) -> LRESULT {
		// first, pass the message to Sciter.
		let mut handled = false as BOOL;
		let lr = (sciter::SciterAPI().SciterProcND)(hwnd, msg, wp, lp, &mut handled);

		// if it was handled by Sciter, we're done here.
		if handled != 0 {
			return lr;
		}

		// if not, call the original window proc.
		let mut lr: LRESULT = 0;
		PREV_PROC.with(|procs| {
			let prev_proc = *procs.borrow().get(&hwnd).expect("An unregistered WindowProc is called somehow.");
			lr = unsafe { CallWindowProcW(prev_proc, hwnd, msg, wp, lp) }
		});

		// and return its result
		lr
	}

	// Subclass the window in order to receive its messages.
	const GWLP_WNDPROC: i32 = -4;
	let prev_proc = unsafe { SetWindowLongW(hwnd, GWLP_WNDPROC, wnd_proc) };
	PREV_PROC.with(|procs| {
		procs.borrow_mut().insert(hwnd, prev_proc);
	});
}
