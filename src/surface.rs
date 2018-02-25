// Aldaron's System Interface / Vulkan
// Copyright (c) 2017-2018 Jeron Aldaron Lau <jeron.lau@plopgrizzly.com>
// Licensed under the MIT LICENSE
//
// src/surface.rs

// TODO: Make surface a buffer and blit onto screen with window manager.

use std::mem;
use std::ptr::null_mut;
use libc::c_void;

use super::types::*;
use super::Connection;

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
pub fn create_surface_xcb(_: &Connection, _: VkInstance, _: *mut c_void, _: u32)
	-> VkSurfaceKHR
{
	panic!("Can't create XCB surface on not Unix.");
}

#[cfg(unix)]
pub fn create_surface_xcb(c: &Connection, instance: VkInstance,
	connection: *mut c_void, window: u32) -> VkSurfaceKHR
{
	let mut surface = unsafe { mem::uninitialized() };
	let surface_create_info = SurfaceCreateInfoXcb {
		s_type: VkStructureType::SurfaceCreateInfoXcb,
		p_next: null_mut(),
		flags: 0,
		connection: connection,
		window: window,
	};
	
	let create_surface : unsafe extern "system" fn(
		instance: VkInstance,
		pCreateInfo: *const SurfaceCreateInfoXcb,
		pAllocator: *mut c_void,
		surface: *mut VkSurfaceKHR) -> VkResult
		= unsafe
	{
		super::vk_sym(instance, c.vksym, b"vkCreateXcbSurfaceKHR\0")
	};

	unsafe {
		(create_surface)(instance,
			&surface_create_info, null_mut(), &mut surface)
		.unwrap();
	};

	surface
}

#[cfg(not(target_os = "windows"))]
pub fn create_surface_windows(_: &Connection, _: VkInstance, _: *mut c_void,
	_: *mut c_void) -> VkSurfaceKHR
{
	panic!("Can't create Windows surface on not Windows.");
}

#[cfg(target_os = "windows")]
pub fn create_surface_windows(c: &Connection, instance: VkInstance,
	connection: *mut c_void, window: *mut c_void) -> VkSurfaceKHR
{
	let mut surface = unsafe { mem::uninitialized() };
	let surface_create_info = SurfaceCreateInfoWindows {
		s_type: VkStructureType::SurfaceCreateInfoWindows,
		p_next: null_mut(),
		flags: 0,
		hinstance: connection,
		hwnd: window,
	};
	
	let create_surface: unsafe extern "system" fn(
		instance: VkInstance,
		pCreateInfo: *const SurfaceCreateInfoWindows,
		pAllocator: *mut c_void,
		surface: *mut VkSurfaceKHR) -> VkResult
		= unsafe
	{
		super::vk_sym(instance, c.vksym, b"vkCreateWin32SurfaceKHR\0")
	};

	unsafe {
		(create_surface)(instance, &surface_create_info, null_mut(),
			&mut surface)
		.unwrap();
	};

	surface
}

// TODO
/* #[cfg(not(target_os = "android"))]
pub fn create_surface_android(_: VkInstance, _: *mut c_void) -> VkSurfaceKHR {
	panic!("Can't create Android surface on not Android.");
}

#[cfg(target_os = "android")]
pub fn create_surface_android(instance: VkInstance, window: *mut c_void)
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
			instance, &surface_create_info, null_mut(), &mut surface
		));
	};

	surface
}*/
