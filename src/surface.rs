// "asi_vulkan" crate - Licensed under the MIT LICENSE
//  * Copyright (c) 2017-2018  Jeron A. Lau <jeron.lau@plopgrizzly.com>

// TODO: Make surface a buffer and blit onto screen with window manager.

use std::{ mem, ptr::{ null_mut } };
use libc::c_void;

use vulkan;
use types::*;
use Vk;

#[cfg(unix)] #[repr(C)]
struct SurfaceCreateInfoXcb {
	s_type: VkStructureType,
	p_next: *mut c_void,
	flags: u32,
	connection: *mut c_void,
	window: u32,
}

#[cfg(target_os = "windows")] #[repr(C)]
struct SurfaceCreateInfoWindows {
	s_type: VkStructureType,
	p_next: *mut c_void,
	flags: u32,
	// TODO
	hinstance: *mut c_void,
	hwnd: *mut c_void,
}

#[cfg(target_os = "android")] #[repr(C)]
struct SurfaceCreateInfoAndroid {
	s_type: VkStructureType,
	p_next: *mut c_void,
	flags: u32,
	window: *mut c_void, // ANativeWindow,
}

#[cfg(not(unix))]
pub fn new_surface_xcb(_: &mut Vk, _: *mut c_void, _: u32) {
	panic!("Can't create XCB surface on not Unix.");
}

#[cfg(unix)]
pub fn new_surface_xcb(c: &mut Vk, wc: *mut c_void, w: u32) {
	let c = c.0.data();
	let mut surface = unsafe { mem::uninitialized() };
	let surface_create_info = SurfaceCreateInfoXcb {
		s_type: VkStructureType::SurfaceCreateInfoXcb,
		p_next: null_mut(),
		flags: 0,
		connection: wc,
		window: w,
	};

	let new_surface : unsafe extern "system" fn(
		instance: VkInstance,
		pCreateInfo: *const SurfaceCreateInfoXcb,
		pAllocator: *mut c_void,
		surface: *mut VkSurfaceKHR) -> VkResult
		= unsafe
	{
		vulkan::sym(c, b"vkCreateXcbSurfaceKHR\0")
	};

	unsafe {
		(new_surface)(c.vk, &surface_create_info, null_mut(),
			&mut surface)
		.unwrap();
	};

	c.surface = surface;
}

#[cfg(not(target_os = "windows"))]
pub fn new_surface_windows(_: &mut Vk, _: *mut c_void, _: *mut c_void) {
	panic!("Can't create Windows surface on not Windows.");
}

#[cfg(target_os = "windows")]
pub fn new_surface_windows(c: &mut Vk, wc: *mut c_void, w: *mut c_void) {
	let c = c.0.data();
	let mut surface = unsafe { mem::uninitialized() };
	let surface_create_info = SurfaceCreateInfoWindows {
		s_type: VkStructureType::SurfaceCreateInfoWindows,
		p_next: null_mut(),
		flags: 0,
		hinstance: wc,
		hwnd: w,
	};
	
	let new_surface: unsafe extern "system" fn(
		instance: VkInstance,
		pCreateInfo: *const SurfaceCreateInfoWindows,
		pAllocator: *mut c_void,
		surface: *mut VkSurfaceKHR) -> VkResult
		= unsafe
	{
		vulkan::sym(c, b"vkCreateWin32SurfaceKHR\0")
	};

	unsafe {
		(new_surface)(c.vk, &surface_create_info, null_mut(),
			&mut surface)
		.unwrap();
	};

	c.surface = surface;
}

// TODO
/* #[cfg(not(target_os = "android"))]
fn new_surface_android(_: VkInstance, _: *mut c_void) -> VkSurfaceKHR {
	panic!("Can't create Android surface on not Android.");
}

#[cfg(target_os = "android")]
fn new_surface_android(c: &Vulkan, window: *mut c_void)
	-> VkSurfaceKHR
{
	let mut surface = unsafe { mem::uninitialized() };
	let surface_create_info = SurfaceCreateInfoAndroid {
		s_type: VkStructureType::SurfaceCreateInfoAndroid,
		p_next: null_mut(),
		flags: 0,
		window: window,
	};

	unsafe {
		extern "system" {
			fn vkCreateAndroidSurfaceKHR(instance: VkInstance,
				pCreateInfo: *const SurfaceCreateInfoAndroid,
				pAllocator: *mut c_void,
				surface: *mut VkSurfaceKHR) -> VkResult;
		}
		check_error(ERROR, vkCreateAndroidSurfaceKHR(
			c.vk, &surface_create_info, null_mut(), &mut surface
		));
	};

	surface
}*/
