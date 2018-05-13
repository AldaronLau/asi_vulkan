// Aldaron's System Interface / Vulkan
// Copyright (c) 2017-2018 Jeron Aldaron Lau <jeron.lau@plopgrizzly.com>
// Licensed under the MIT LICENSE
//
// src/memory/buffer.rs

use std::{ mem, ptr };
use libc::c_void;

use vulkan;
use Vulkan;
use types::*;

pub enum BufferBuilderType {
	Uniform,
	Vertex,
}

/// A buffer in GPU memory.
pub struct Buffer {
	pub buffer: VkBuffer,
	dropfn: unsafe extern "system" fn(VkDevice, VkBuffer, *const c_void)
		-> (),
}

impl Buffer {
	/// Create a new buffer on the GPU.
	#[inline(always)]
	pub fn new(c: &Vulkan, nbytes: usize, bbt: BufferBuilderType) -> Buffer{
		let mut buffer = unsafe { mem::uninitialized() };
		unsafe {
			(c.new_buffer)(
				c.device,
				&VkBufferCreateInfo {
					s_type: VkStructureType::BufferCreateInfo,
					next: ptr::null(),
					flags: 0,
					size: nbytes as u64,
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
		let dropfn = unsafe {
			vulkan::dsym(c, b"vkDestroyBuffer\0")
		};
		Buffer { buffer, dropfn }
	}

	/// Get Memory Requirements.
	#[inline(always)]
	pub fn get_reqs(&self, connection: &Vulkan) -> VkMemoryRequirements {
		let mut mem_reqs = unsafe { mem::uninitialized() };
		unsafe {
			(connection.get_bufmemreq)(
				connection.device,
				self.buffer,
				&mut mem_reqs
			);
		}
		mem_reqs
	}

	/// Called by `Memory`'s drop()
	#[inline(always)]
	pub fn drop(&mut self, device: VkDevice) {
		unsafe {
			(self.dropfn)(device, self.buffer, ptr::null());
		}
	}
}
