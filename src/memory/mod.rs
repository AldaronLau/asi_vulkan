// Aldaron's System Interface / Vulkan
// Copyright (c) 2017-2018 Jeron Aldaron Lau <jeron.lau@plopgrizzly.com>
// Licensed under the MIT LICENSE
//
// src/memory/mod.rs

use std::{ mem, ptr };
use libc::c_void;

use vulkan;
use Vk;
use types::*;

mod buffer;

pub use self::buffer::{ Buffer, BufferBuilderType };
pub(crate) use self::buffer::destroy;

pub struct Memory<T> where T: Clone {
	pub data: T,
//	pub memory: VkDeviceMemory,
	pub buffer: buffer::Buffer,
//	#[allow(unused)] // TODO
//	device: VkDevice,
//	#[allow(unused)] // TODO
//	dropfn: unsafe extern "system" fn(VkDevice, VkDeviceMemory,
//		*const c_void) -> ()
}

impl<T> Memory<T> where T: Clone {
	/// Allocate memory in a GPU buffer.
	#[inline(always)]
	pub fn new(vulkan: &mut Vk, data: T) -> Memory<T> {
//		let c = vulkan.0.data();

		let buffer = buffer::Buffer::new(vulkan,
			&[data.clone()],
			buffer::BufferBuilderType::Uniform);
/*		let mut memory = unsafe { mem::uninitialized() };
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
				buffer.buffer().0,
				memory,
				0
			).unwrap();
		}

		let device = vulkan.0.data().device;

		let memory = Memory { data, memory, buffer, device, dropfn };

		memory.update(vulkan);

		memory*/

		Memory { data: data.clone(), buffer }
	}

	/// Update the contents of the memory.
	#[inline(always)]
	pub fn update(&self, vulkan: &mut Vk) {
		self.buffer.update(&[self.data.clone()], vulkan);
	}
}
