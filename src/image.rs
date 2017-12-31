use std::mem;
use ami::Void;

use types::*;
use Connection;
use get_memory_type;

/// An Image
pub struct Image {
	pub image: VkImage,
	pub image_memory: VkDeviceMemory,
	device: VkDevice,
	drop_image: unsafe extern "system" fn(VkDevice, VkImage, *const Void)
		-> (),
	drop_memory: unsafe extern "system" fn(VkDevice, VkDeviceMemory,
		*const Void) -> (),
}

impl Image {
	/// Create a new image.
	#[inline(always)] pub fn new(c: &Connection, device: VkDevice,
		gpu: VkPhysicalDevice, width: u32, height: u32,
		format: VkFormat, tiling: VkImageTiling, usage: VkImageUsage,
		initial_layout: VkImageLayout, reqs_mask: VkFlags) -> Image
	{ unsafe {
		let drop_image
			= super::vkd_sym(device, c.vkdsym, b"vkDestroyImage\0");
		let drop_memory =
			super::vkd_sym(device, c.vkdsym, b"vkFreeMemory\0");

		let mut image = mem::uninitialized();
		let mut image_memory = mem::uninitialized();

		let mut memory_reqs = mem::uninitialized();

		(c.create_image)(
			device,
			&VkImageCreateInfo {
				s_type: VkStructureType::ImageCreateInfo,
				p_next: null!(),
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
				samples: VkSampleCount::Sc1,
				tiling,
				usage,
				sharing_mode: VkSharingMode::Exclusive,
				queue_family_index_count: 0,
				p_queue_family_indices: null!(),
				initial_layout,
			},
			null!(),
			&mut image
		).unwrap();

		(c.get_imgmemreq)(device, image, &mut memory_reqs);

		(c.mem_allocate)(
			device,
			&VkMemoryAllocateInfo {
				s_type: VkStructureType::MemoryAllocateInfo,
				next: null!(),
				allocation_size: memory_reqs.size,
				memory_type_index: get_memory_type(
					c,
					gpu,
					memory_reqs.memory_type_bits,
					reqs_mask
				),
			},
			null!(),
			&mut image_memory
		).unwrap();

		(c.bind_imgmem)(device, image, image_memory, 0)
			.unwrap();

		Image { image, image_memory, device, drop_image, drop_memory }
	} }
}

impl Drop for Image {
	#[inline(always)]
	fn drop(&mut self) {
		// TODO: image & image_memory are being moved, dropping the
		// Image - causing them to be invalid
		unsafe {
//			(self.drop_image)(self.device, self.image, null!());
//			(self.drop_memory)(self.device, self.image_memory,
//				null!());
		}
	}
}
