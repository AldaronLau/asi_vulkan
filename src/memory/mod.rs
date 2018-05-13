// Aldaron's System Interface / Vulkan
// Copyright (c) 2017-2018 Jeron Aldaron Lau <jeron.lau@plopgrizzly.com>
// Licensed under the MIT LICENSE
//
// src/memory/mod.rs

use std::{ mem, ptr };
use libc::c_void;

use VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT;
use VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
use vulkan;
use Vk;
use types::*;

// TODO: pub?
pub mod buffer;

pub struct Memory<T> where T: Clone {
	pub data: T,
	pub memory: VkDeviceMemory,
	pub buffer: buffer::Buffer,
	#[allow(unused)] // TODO
	device: VkDevice,
	#[allow(unused)] // TODO
	dropfn: unsafe extern "system" fn(VkDevice, VkDeviceMemory,
		*const c_void) -> ()
}

impl<T> Memory<T> where T: Clone {
	/// Allocate memory in a GPU buffer.
	#[inline(always)]
	pub fn new(vulkan: &mut Vk, data: T) -> Memory<T> {
//		let c = vulkan.0.data();

		let buffer = buffer::Buffer::new(vulkan.0.data(),
			mem::size_of::<T>(),
			buffer::BufferBuilderType::Uniform);
		let mut memory = unsafe { mem::uninitialized() };
		let mem_reqs = buffer.get_reqs(vulkan.0.data());
		unsafe {
			(vulkan.0.data().mem_allocate)(
				vulkan.0.data().device,
				&VkMemoryAllocateInfo {
					s_type: VkStructureType::MemoryAllocateInfo,
					next: ptr::null(),
					allocation_size: mem_reqs.size,
					memory_type_index: super::get_memory_type(
						vulkan,
						mem_reqs.memory_type_bits,
						VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT |
						VK_MEMORY_PROPERTY_HOST_COHERENT_BIT),
				},
				ptr::null(),
				&mut memory
			).unwrap();
		}
		let dropfn = unsafe {
			vulkan::dsym(vulkan.0.data(), b"vkFreeMemory\0")
		};

		unsafe {
			(vulkan.0.data().bind_buffer_mem)(
				vulkan.0.data().device,
				buffer.buffer,
				memory,
				0
			).unwrap();
		}

		let device = vulkan.0.data().device;

		let memory = Memory { data, memory, buffer, device, dropfn };

		memory.update(vulkan);

		memory
	}

	/// Update the contents of the memory.
	#[inline(always)]
	pub fn update(&self, vulkan: &mut Vk) {
		let c = vulkan.0.data();

		let mut mapped: *mut T = unsafe { mem::uninitialized() }; // void *

		unsafe {
			(c.mapmem)(self.device, self.memory, 0, !0, 0,
				&mut mapped as *mut *mut _ as *mut *mut c_void)
				.unwrap();
		}

		if mapped.is_null() {
			panic!("Couldn't Map Buffer Memory?  Unknown cause.");
		}

		let write = self.data.clone();

		unsafe {
			*mapped = write;
		}

//		let mut map = mapped as *mut _ as *mut T;

//		unsafe {
//			*map = self.data.clone();
//		}

/*		extern "C" {
			fn memcpy(dest: *mut Void, src: *const Void, n: usize)
				-> ::ami::MemAddr<Void>;
		}*/

		unsafe {
	//		memcpy(mapped, cast!(&self.data), size);
	//		ptr::copy_nonoverlapping(data.as_ptr(), mapped, data.len());

			(c.unmap)(self.device, self.memory);

//			asi_vulkan::unmap_memory(c, self.device, self.memory);
		}
	}

	#[inline(always)]
	/// Update the contents of image memory.
	pub fn update_pitched() {
		
	}
}

impl<T> Drop for Memory<T> where T: Clone {
	#[inline(always)]
	fn drop(&mut self) {
		// TODO: Drop at correct time as for no segfault.
/*		unsafe {
			(self.dropfn)(self.device, self.memory, ptr::null());
		}
		self.buffer.drop(self.device);*/
	}
}
