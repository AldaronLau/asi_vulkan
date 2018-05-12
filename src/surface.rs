// Aldaron's System Interface / Vulkan
// Copyright (c) 2017-2018 Jeron Aldaron Lau <jeron.lau@plopgrizzly.com>
// Licensed under the MIT LICENSE
//
// src/surface.rs

// TODO: Make surface a buffer and blit onto screen with window manager.

use std::{ mem, ptr::{ null_mut } };
use libc::c_void;
use ami::{ PseudoDrop, Child };

use vulkan;
use types::*;
use Vk;
use Vulkan;

pub struct Surface(pub(crate)Child<Vulkan, VkSurfaceKHR>);

impl Surface {
	/// Create a new surface on a Windows Window
	pub fn new_windows(vulkan: &mut Vk, connection: *mut c_void,
		window: *mut c_void) -> Self
	{
		let c = vulkan.0.data();

		Surface(Child::new(&vulkan.0, create_surface_windows(c, connection, window)))
	}

	/// Create a new surface on an XCB Window
	pub fn new_xcb(vulkan: &mut Vk, connection: *mut c_void,
		window: u32) -> Self
	{
		let c = vulkan.0.data();

		Surface(Child::new(&vulkan.0, create_surface_xcb(c, connection, window)))
	}
}

impl PseudoDrop for VkSurfaceKHR {
	type T = Vulkan;

	fn pdrop(&mut self, vulkan: &mut Vulkan) -> () {
		let c = vulkan;

		// Load Drop Function
		type VkDestroySurface = unsafe extern "system" fn(
			instance: VkInstance, surface: VkSurfaceKHR,
			pAllocator: *mut c_void) -> ();
		let destroy: VkDestroySurface = unsafe {
			vulkan::vk_sym(c.vk, c.vksym, b"vkDestroySurfaceKHR\0")
		};

		// Run Drop Function
		unsafe {
			destroy(c.vk, *self, null_mut())
		};

		println!("TEST: Drop Surface");
	}
}

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
fn create_surface_xcb(_: &Vulkan, _: *mut c_void, _: u32)
	-> VkSurfaceKHR
{
	panic!("Can't create XCB surface on not Unix.");
}

#[cfg(unix)]
fn create_surface_xcb(c: &Vulkan, connection: *mut c_void, window: u32)
	-> VkSurfaceKHR
{
	let mut surface = unsafe { mem::uninitialized() };
	let surface_create_info = SurfaceCreateInfoXcb {
		s_type: VkStructureType::SurfaceCreateInfoXcb,
		p_next: null_mut(),
		flags: 0,
		connection,
		window,
	};

	let create_surface : unsafe extern "system" fn(
		instance: VkInstance,
		pCreateInfo: *const SurfaceCreateInfoXcb,
		pAllocator: *mut c_void,
		surface: *mut VkSurfaceKHR) -> VkResult
		= unsafe
	{
		vulkan::sym(c, b"vkCreateXcbSurfaceKHR\0")
	};

	unsafe {
		(create_surface)(c.vk, &surface_create_info, null_mut(),
			&mut surface)
		.unwrap();
	};

	surface
}

#[cfg(not(target_os = "windows"))]
fn create_surface_windows(_: &Vulkan, _: *mut c_void, _: *mut c_void)
	-> VkSurfaceKHR
{
	panic!("Can't create Windows surface on not Windows.");
}

#[cfg(target_os = "windows")]
fn create_surface_windows(c: &Vulkan, connection: *mut c_void,
	window: *mut c_void) -> VkSurfaceKHR
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
		vulkan::sym(c, b"vkCreateWin32SurfaceKHR\0")
	};

	unsafe {
		(create_surface)(c.vk, &surface_create_info, null_mut(),
			&mut surface)
		.unwrap();
	};

	surface
}

// TODO
/* #[cfg(not(target_os = "android"))]
fn create_surface_android(_: VkInstance, _: *mut c_void) -> VkSurfaceKHR {
	panic!("Can't create Android surface on not Android.");
}

#[cfg(target_os = "android")]
fn create_surface_android(c: &Vulkan, window: *mut c_void)
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
