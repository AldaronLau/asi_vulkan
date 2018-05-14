// Aldaron's System Interface / Vulkan
// Copyright (c) 2017 Jeron Aldaron Lau <jeron.lau@plopgrizzly.com>
// Licensed under the MIT LICENSE
//
// src/image.rs

use std::mem;
use std::ptr::{ null };
use ami::Child;

use VkObject;
use VkType;
use Vulkan;
use Vk;
use types::*;
use get_memory_type;

/// An Image
pub struct Image(pub(crate)Child<Vulkan, VkObject>);

impl Image {
	/// Create a new image.
	#[inline(always)] pub fn new(vulkan: &mut Vk, width: u32, height: u32,
		format: VkFormat, tiling: VkImageTiling, usage: VkImageUsage,
		initial_layout: VkImageLayout, reqs_mask: VkFlags,
		samples: VkSampleCount) -> Image
	{ unsafe {
//		let c = vulkan.0.data();

//		let drop_image
//			= vulkan::dsym(vulkan.0.data(), device, b"vkDestroyImage\0");
//		let drop_memory =
//			vulkan::dsym(vulkan.0.data(), device, b"vkFreeMemory\0");

		let mut image = mem::uninitialized();
		let mut image_memory = mem::uninitialized();
		let mut memory_reqs = mem::uninitialized();

		(vulkan.0.data().create_image)(
			vulkan.0.data().device,
			&VkImageCreateInfo {
				s_type: VkStructureType::ImageCreateInfo,
				p_next: null(),
				flags: 0,
				image_type: VkImageType::Dim2d,
				format,
				extent: VkExtent3D {
					width: width,
					height: height,
					depth: 1,
				},
				mip_levels: 1,
				array_layers: 1,
				samples,
				tiling,
				usage,
				sharing_mode: VkSharingMode::Exclusive,
				queue_family_index_count: 0,
				p_queue_family_indices: null(),
				initial_layout,
			},
			null(),
			&mut image
		).unwrap();

		(vulkan.0.data().get_imgmemreq)(vulkan.0.data().device, image,
			&mut memory_reqs);

		(vulkan.0.data().mem_allocate)(
			vulkan.0.data().device,
			&VkMemoryAllocateInfo {
				s_type: VkStructureType::MemoryAllocateInfo,
				next: null(),
				allocation_size: memory_reqs.size,
				memory_type_index: get_memory_type(
					vulkan,
					memory_reqs.memory_type_bits,
					reqs_mask
				),
			},
			null(),
			&mut image_memory
		).unwrap();

		(vulkan.0.data().bind_imgmem)(vulkan.0.data().device, image,
			image_memory, 0).unwrap();

		Image(Child::new(&vulkan.0,
			VkObject::new(VkType::Image, image, image_memory, 0)))
	} }

	pub (crate) fn image(&self) -> (u64, u64) {
		self.0.data().image()
	}

	/// Get the memory handle for this image.
	pub fn memory(&self) -> u64 {
		self.image().1
	}
}

#[inline(always)] pub(crate) fn destroy(image: (u64, u64), c: &mut Vulkan) {
	// Run Drop Function
	unsafe {
		(c.drop_image)(c.device, image.0, null());
		(c.drop_memory)(c.device, image.1, null());
	}

	println!("TEST: Drop Image");
}
