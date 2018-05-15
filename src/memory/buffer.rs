// Aldaron's System Interface / Vulkan
// Copyright (c) 2017-2018 Jeron Aldaron Lau <jeron.lau@plopgrizzly.com>
// Licensed under the MIT LICENSE
//
// src/memory/buffer.rs

use null;
use std::{ mem, ptr };
use libc::c_void;
use ami::Child;

use VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT;
use VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
use Vulkan;
use Vk;
use VkObject;
use VkType;
use types::*;

pub enum BufferBuilderType {
	Uniform,
	Vertex,
}

/// A buffer in GPU memory.
pub struct Buffer(pub(crate)Child<Vulkan, VkObject>);

impl Buffer {
	/// Create a new buffer on the GPU.
	#[inline(always)]
	pub fn new<T: Clone>(vulkan: &mut Vk, data: &[T], bbt: BufferBuilderType)
		-> Buffer
	{
		let mut buffer = unsafe { mem::uninitialized() };
		let mut memory = unsafe { mem::uninitialized() };
		let mut mem_reqs = unsafe { mem::uninitialized() };
		unsafe {
			(vulkan.0.data().new_buffer)(
				vulkan.0.data().device,
				&VkBufferCreateInfo {
					s_type: VkStructureType::BufferCreateInfo,
					next: ptr::null(),
					flags: 0,
					size: (mem::size_of::<T>() * data.len())
						as u64,
					usage: match bbt {
					  BufferBuilderType::Uniform =>
					    VkBufferUsage::UniformBufferBit,
					  BufferBuilderType::Vertex =>
					    VkBufferUsage::VertexIndexBufferBit,
					},
					sharing_mode: VkSharingMode::Exclusive,
					queue_family_index_count: 0,
					queue_family_indices: ptr::null(),
				},
				ptr::null(),
				&mut buffer
			).unwrap();
		}
		// memory requirements
		unsafe {
			(vulkan.0.data().get_bufmemreq)(
				vulkan.0.data().device,
				buffer,
				&mut mem_reqs
			);
		}
		// memory
		unsafe {
			(vulkan.0.data().mem_allocate)(
				vulkan.0.data().device,
				&VkMemoryAllocateInfo {
					s_type: VkStructureType::MemoryAllocateInfo,
					next: ptr::null(),
					allocation_size: mem_reqs.size,
					memory_type_index: ::get_memory_type(
						vulkan,
						mem_reqs.memory_type_bits,
						VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT |
						VK_MEMORY_PROPERTY_HOST_COHERENT_BIT),
				},
				ptr::null(),
				&mut memory
			).unwrap();
			(vulkan.0.data().bind_buffer_mem)(
				vulkan.0.data().device,
				buffer,
				memory,
				0
			).unwrap();
		}

		let buffer = Buffer(Child::new(&vulkan.0, VkObject::new(
			VkType::Buffer, buffer, memory, 0)));

		buffer.update(data, vulkan);

		buffer
	}

	pub fn memory(&self) -> u64 {
		self.0.data().image().1
	}

	pub fn buffer(&self) -> u64 {
		self.0.data().image().0
	}

	/// Update the contents of the memory.
	#[inline(always)] pub fn update<T: Clone>(&self, data: &[T],
		vulkan: &mut Vk)
	{
		let c = vulkan.0.data();

		let mut mapped: *mut T = unsafe { mem::uninitialized() };

		unsafe {
			(c.mapmem)(c.device, self.memory(), 0, !0, 0,
				&mut mapped as *mut *mut _ as *mut *mut c_void)
				.unwrap();
		}

		if mapped.is_null() {
			panic!("Couldn't Map Buffer Memory?  Unknown cause.");
		}

		unsafe {
			for i in 0..data.len() {
				*mapped.offset(i as isize) = data[i].clone();
			}
			(c.unmap)(c.device, self.memory());
		}
	}
}

#[inline(always)] pub(crate) fn destroy(buffer: (u64, u64), c: &mut Vulkan){
	// Run Drop Function
	unsafe {
		(c.drop_buffer)(c.device, buffer.0, null());
		(c.drop_memory)(c.device, buffer.1, null());
	}
}
