// "asi_vulkan" - Aldaron's System Interface - Vulkan
//
// Copyright Jeron A. Lau 2018.
// Distributed under the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

extern crate ami;
#[macro_use]
extern crate dl_api;

// Modules
pub mod types;
pub mod memory;

mod image;
mod surface;
mod vulkan;
mod sprite;
mod style;
mod fence;

//
use std::{ mem, u64 };
use std::ptr::{ null, null_mut };
use std::os::raw::c_void;

// Export Types
pub use self::memory::{ Memory, Buffer, BufferBuilderType };
pub use self::image::Image;
pub use self::surface::{ new_surface_xcb, new_surface_windows };
pub use self::vulkan::Vk;
pub use self::sprite::Sprite;
pub use self::style::Style;
pub use self::fence::Fence;

//
use self::types::*;
use self::vulkan::{ Vulkan, VkObject, VkType };

const VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT: VkFlags = 0x00000002;
const VK_MEMORY_PROPERTY_HOST_COHERENT_BIT: VkFlags = 0x00000004;

const VK_SAMPLE_COUNT: VkSampleCount = VkSampleCount::Sc8;

// TODO
#[derive(Clone)] #[repr(C)] pub struct TransformUniform {
	pub mat4: [f32; 16],
}

// TODO
#[derive(Clone)] #[repr(C)] pub struct FogUniform {
	pub fogc: [f32; 4],
	pub fogr: [f32; 2],
}

pub unsafe fn get_gpu(connection: &mut Vk) -> Result<(), &'static str> {
	let connection = connection.0.data();

	#[repr(C)]
	struct VkQueueFamilyProperties {
		queue_flags: u32,
		queue_count: u32,
		timestamp_valid_bits: u32,
		min_image_transfer_granularity: VkExtent3D,
	}

	// Load Function
	type ListGpus = unsafe extern "system" fn(VkInstance, *mut u32,
		*mut VkPhysicalDevice) -> VkResult;
	let vk_list_gpus: ListGpus = vulkan::sym(connection,
		b"vkEnumeratePhysicalDevices\0");

	// Set Data
	let mut num_gpus = 0;

	// Run Function
	vk_list_gpus(connection.vk, &mut num_gpus, null_mut()).unwrap();

	// Set Data
	let mut gpus = vec![mem::uninitialized(); num_gpus as usize];

	// Run function
	vk_list_gpus(connection.vk, &mut num_gpus, gpus.as_mut_ptr()).unwrap();

	// Load functions
	type GetGpuQueueFamProps = unsafe extern "system" fn(VkPhysicalDevice,
		*mut u32, *mut VkQueueFamilyProperties) -> ();
	type GetGpuSurfaceSupport = unsafe extern "system" fn(VkPhysicalDevice,
		u32, VkSurfaceKHR, *mut u32) -> VkResult;

	let vk_get_props: GetGpuQueueFamProps = vulkan::sym(connection,
		b"vkGetPhysicalDeviceQueueFamilyProperties\0");
	let vk_get_support: GetGpuSurfaceSupport = vulkan::sym(connection,
		b"vkGetPhysicalDeviceSurfaceSupportKHR\0");

	// Process Data
	for i in 0..(num_gpus as usize) {
		let mut num_queue_families = 0;

		vk_get_props(gpus[i], &mut num_queue_families, null_mut());

		let queue_families_size = num_queue_families as usize;

		let mut properties = Vec::with_capacity(queue_families_size);

		properties.set_len(queue_families_size);

		vk_get_props(gpus[i], &mut num_queue_families,
			properties.as_mut_ptr());

		for j in 0..queue_families_size {
			let k = j as u32;
			let mut supports_present = 0;

			vk_get_support(gpus[i], k, connection.surface,
				&mut supports_present).unwrap();

			if supports_present != 0 &&
				(properties[j].queue_flags & 0x00000001) != 0
			{
				// 
				let mut props = mem::uninitialized();

				(connection.gpu_props)(gpus[i],
					VkFormat::R8g8b8a8Unorm, &mut props);

				connection.gpu = gpus[i];
				connection.pqi = k;
				connection.sampled=props.linear_tiling_features
					& 0x00000001 /* sampled image */ != 0;
				return Ok(());
			}
		}
	}

	Err("Couldn't Create Gpu.")
}

pub unsafe fn create_device(connection: &mut Vk) {
	let connection = connection.0.data();

	#[derive(Debug)] #[repr(C)]
	struct VkDeviceQueueCreateInfo {
		s_type: VkStructureType,
		p_next: *mut c_void,
		flags: u32,
		queue_family_index: u32,
		queue_count: u32,
		p_queue_priorities: *const f32,
	}

	#[derive(Debug)] #[repr(C)]
	struct VkDeviceCreateInfo {
		s_type: VkStructureType,
		p_next: *mut c_void,
		flags: u32,
		queue_create_info_count: u32,
		p_queue_create_infos: *const VkDeviceQueueCreateInfo,
		enabled_layer_count: u32,
		enabled_layer_names: *const *const u8,
		enabled_extension_count: u32,
		enabled_extension_names: *const *const u8,
		enabled_features: *mut c_void,
	}

	// Load function
	type VkCreateDevice = extern "system" fn(
		physicalDevice: VkPhysicalDevice,
		pCreateInfo: *const VkDeviceCreateInfo,
		pAllocator: *mut c_void,
		pDevice: *mut VkDevice) -> VkResult;
	let vk_create_device: VkCreateDevice = vulkan::sym(connection,
		b"vkCreateDevice\0");

	let ext = b"VK_KHR_swapchain\0";

	vk_create_device(connection.gpu, &VkDeviceCreateInfo {
		s_type: VkStructureType::DeviceCreateInfo,
		p_next: null_mut(),
		flags: 0,
		queue_create_info_count: 1,
		p_queue_create_infos: [VkDeviceQueueCreateInfo {
			s_type: VkStructureType::DeviceQueueCreateInfo,
			p_next: null_mut(),
			flags: 0,
			queue_family_index: connection.pqi,
			queue_count: 1,
			p_queue_priorities: &1.0,
		}].as_ptr(),
		enabled_layer_count: 0,
		enabled_layer_names: null(),
		enabled_extension_count: 1,
		enabled_extension_names: [ext.as_ptr()].as_ptr(),
		enabled_features: null_mut(),
	}, null_mut(), &mut connection.device).unwrap();
}

pub unsafe fn create_queue(connection: &mut Vk) -> VkQueue {
	let connection = connection.0.data();

	// Load function
	type VkGetDeviceQueue = extern "system" fn(device: VkDevice,
		queueFamilyIndex: u32, queueIndex: u32, pQueue: *mut VkQueue)
		-> ();
	let vk_get_device_queue: VkGetDeviceQueue = vulkan::dsym(connection,
		b"vkGetDeviceQueue\0");

	// Set Data
	let mut queue = mem::uninitialized();

	// Run Function
	vk_get_device_queue(connection.device, connection.pqi, 0, &mut queue);

	// Return
	queue
}

pub unsafe fn queue_present(connection: &mut Vk, queue: VkQueue,
	semaphore: VkSemaphore, swapchain: VkSwapchainKHR, next: u32)
{
	let connection = connection.0.data();

	let present_info = VkPresentInfo {
		s_type: VkStructureType::PresentInfo,
		next: null(),
		wait_semaphore_count: 1,
		wait_semaphores: &semaphore,
		swapchain_count: 1,
		swapchains: &swapchain,
		image_indices: &next,
		results: null_mut(),
	};

	(connection.queue_present)(queue, &present_info).unwrap()
}

pub unsafe fn wait_idle(connection: &mut Vk) {
	let connection = connection.0.data();

	(connection.wait_idle)(connection.device).unwrap();
}

pub unsafe fn create_command_buffer(connection: &mut Vk) {
	let connection = connection.0.data();

	#[repr(C)]
	enum VkCommandBufferLevel {
		Primary = 0,
	}

	#[repr(C)]
	struct VkCommandPoolCreateInfo {
		s_type: VkStructureType,
		p_next: *mut c_void,
		flags: u32,
		queue_family_index: u32,
	}

	#[repr(C)]
	struct VkCommandBufferAllocateInfo {
		s_type: VkStructureType,
		p_next: *mut c_void,
		command_pool: u64,
		level: VkCommandBufferLevel,
		command_buffer_count: u32,
	}

	// Load function
	type VkCreateCommandPool = extern "system" fn(device: VkDevice,
		pCreateInfo: *const VkCommandPoolCreateInfo,
		pAllocator: *mut c_void, pCommandPool: *mut u64) -> VkResult;
	let vk_create_command_pool: VkCreateCommandPool = vulkan::dsym(
		connection, b"vkCreateCommandPool\0");

	// Set Data
	let mut command_pool = 0;
	let mut command_buffer = mem::uninitialized();

	let create_info = VkCommandPoolCreateInfo {
		s_type: VkStructureType::CommandPoolCreateInfo,
		p_next: null_mut(),
		flags: 0x00000002, // Reset Command Buffer
		queue_family_index: connection.pqi,
	};

	// Run Function
	vk_create_command_pool(connection.device, &create_info, null_mut(),
		&mut command_pool).unwrap();

	// Load Function
	type VkAllocateCommandBuffers = extern "system" fn(device: VkDevice,
		ai: *const VkCommandBufferAllocateInfo,
		cmd_buffs: *mut VkCommandBuffer) -> VkResult;
	let vk_allocate_command_buffers: VkAllocateCommandBuffers =
		vulkan::dsym(connection, b"vkAllocateCommandBuffers\0");

	// Set Data
	let allocate_info = VkCommandBufferAllocateInfo {
		s_type: VkStructureType::CommandBufferAllocateInfo,
		p_next: null_mut(),
		command_pool: command_pool,
		level: VkCommandBufferLevel::Primary,
		command_buffer_count: 1,
	};

	// Run Function
	vk_allocate_command_buffers(connection.device, &allocate_info,
		&mut command_buffer).unwrap();

	// Return
	connection.command_buffer = command_buffer;
	connection.command_pool = command_pool;
}

pub unsafe fn new_sampler(connection: &mut Vk) {
	let connection = connection.0.data();

	(connection.new_sampler)(
		connection.device,
		&VkSamplerCreateInfo {
			s_type: VkStructureType::SamplerCreateInfo,
			next: null(),
			flags: 0,
			mag_filter: VkFilter::Linear,
			min_filter: VkFilter::Linear,
			mipmap_mode: VkSamplerMipmapMode::Linear,
			address_mode_u: VkSamplerAddressMode::Repeat,
			address_mode_v: VkSamplerAddressMode::Repeat,
			address_mode_w: VkSamplerAddressMode::Repeat,
			mip_lod_bias: 0.0,
			anisotropy_enable: 0,
			max_anisotropy: 1.0,
			compare_enable: 0,
			compare_op: VkCompareOp::Never,
			min_lod: 0.0,
			max_lod: 0.0,
			border_color: VkBorderColor::FloatOpaqueWhite,
			unnormalized_coordinates: 0,
		},
		null(),
		&mut connection.sampler
	).unwrap();
}

pub unsafe fn subres_layout(connection: &mut Vk, image: &Image)
	-> VkSubresourceLayout
{
	let connection = connection.0.data();

	let mut layout = mem::uninitialized();

	(connection.subres_layout)(
		connection.device,
		image.image().0,
		&VkImageSubresource {
			aspect_mask: VkImageAspectFlags::Color,
			mip_level: 0,
			array_layer: 0,
		},
		&mut layout
	);

	layout
}

pub unsafe fn map_memory<T>(connection: &mut Vk, vb_memory: VkDeviceMemory,
	size: u64) -> *mut T
		where T: Clone
{
	let connection = connection.0.data();

	let mut mapped = mem::uninitialized();

	(connection.mapmem)(connection.device, vb_memory, 0, size, 0,
		&mut mapped as *mut *mut _ as *mut *mut c_void).unwrap();

	mapped
}

pub unsafe fn unmap_memory(connection: &mut Vk, vb_memory: VkDeviceMemory) {
	let connection = connection.0.data();

	(connection.unmap)(connection.device, vb_memory);
}

pub unsafe fn get_memory_type(connection: &mut Vk, mut type_bits: u32,
	reqs_mask: VkFlags) -> u32
{
	let connection = connection.0.data();

	let mut props = mem::uninitialized();
	// TODO; only needs to happen once
	(connection.get_memprops)(connection.gpu, &mut props);

	for i in 0..(props.memory_type_count as usize) {
		// Memory type req's matches vkGetImageMemoryRequirements()?
		if (type_bits & 1) == 1
			&& (props.memory_types[i].property_flags & reqs_mask) ==
				reqs_mask
		{
			return i as u32;
		}
		// Check next bit from vkGetImageMemoryRequirements().
		type_bits >>= 1;
	}

	// Nothing works ... fallback to 0 and hope nothing bad happens.
	panic!(concat!(env!("CARGO_PKG_NAME"),
		"Couldn't find suitable memory type."))
}

pub unsafe fn cmd_bind_descsets(connection: &mut Vk,
	pipeline_layout: VkPipelineLayout, desc_set: VkDescriptorSet)
{
	let connection = connection.0.data();

	(connection.bind_descsets)(
		connection.command_buffer,
		VkPipelineBindPoint::Graphics,
		pipeline_layout,
		0,
		1,
		[desc_set].as_ptr(),
		0,
		null(),
	);
}

pub unsafe fn cmd_bind_pipeline(connection: &mut Vk, pipeline: VkPipeline) {
	let connection = connection.0.data();

	(connection.bind_pipeline)(
		connection.command_buffer,
		VkPipelineBindPoint::Graphics,
		pipeline
	);
}

#[inline(always)] pub unsafe fn cmd_bind_vb(connection: &mut Vk,
	vertex_buffers: &[VkBuffer])
{
	let connection = connection.0.data();

	let offsets1 : [u64; 1] = [0];
	let offsets2 : [u64; 2] = [0, 0];
	let offsets3 : [u64; 3] = [0, 0, 0];

	let length = vertex_buffers.len();

	(connection.bind_vb)(
		connection.command_buffer,
		0,
		length as u32,
		vertex_buffers.as_ptr(),
		match length {
			1 => offsets1.as_ptr(),
			2 => offsets2.as_ptr(),
			3 => offsets3.as_ptr(), 
			_ => panic!("Wrong number of vertex buffers (Not 1-3)"),
		},
	);
}

pub unsafe fn cmd_draw(connection: &mut Vk, nvertices: u32, ninstances: u32,
	firstvertex: u32, firstinstance: u32)
{
	let connection = connection.0.data();

	assert!(nvertices > 2);
	(connection.draw)(connection.command_buffer, nvertices, ninstances,
		firstvertex, firstinstance);
}

pub unsafe fn new_semaphore(connection: &mut Vk) -> VkSemaphore {
	let connection = connection.0.data();

	let mut semaphore = mem::uninitialized();

	(connection.new_semaphore)(
		connection.device,
		&VkSemaphoreCreateInfo {
			s_type: VkStructureType::SemaphoreCreateInfo,
			next: null(),
			flags: 0,
		},
		null(),
		&mut semaphore,
	).unwrap();

	semaphore
}

pub unsafe fn drop_semaphore(connection: &mut Vk, semaphore: VkSemaphore) {
	let connection = connection.0.data();

	(connection.drop_semaphore)(
		connection.device,
		semaphore,
		null(),
	);
}

pub unsafe fn draw_begin(connection: &mut Vk, render_pass: VkRenderPass,
	image: VkImage, frame_buffer: VkFramebuffer, width: u32,
	height: u32, r: f32, g: f32, b: f32)
{
	let connection = connection.0.data();

	let begin_info = VkCommandBufferBeginInfo {
		s_type: VkStructureType::CommandBufferBeginInfo,
		p_next: null(),
		flags: VkCommandBufferUsage::OneTimeSubmitBit,
		p_inheritance_info: null(),
	};

	(connection.begin_cmdbuff)(connection.command_buffer, &begin_info)
		.unwrap();

	let layout_transition_barrier = VkImageMemoryBarrier {
		s_type: VkStructureType::ImageMemoryBarrier,
		p_next: null(),
		src_access_mask: VkAccess::MemoryReadBit,
		dst_access_mask: VkAccess::ColorAttachmentReadWrite,
		old_layout: VkImageLayout::PresentSrc,
		new_layout: VkImageLayout::ColorAttachmentOptimal,
		src_queue_family_index: !0,
		dst_queue_family_index: !0,
		image: image,
		subresource_range: VkImageSubresourceRange {
			aspect_mask: VkImageAspectFlags::Color,
			base_mip_level: 0,
			level_count: 1,
			base_array_layer: 0,
			layer_count: 1,
		},
	};

	(connection.pipeline_barrier)(
		connection.command_buffer,
		VkPipelineStage::TopOfPipe, 
		VkPipelineStage::TopOfPipeAndColorAttachmentOutput, 
		0, 0, null(), 0, null(), 1, &layout_transition_barrier);

	// activate render pass:
	let clear_value = [
		VkClearValue { color: VkClearColorValue { float32: [r, g, b, 1.0] } },
		VkClearValue { depth_stencil: VkClearDepthStencilValue { depth: 1.0, stencil: 0 } },
	];

	let render_pass_begin_info = VkRenderPassBeginInfo {
		s_type: VkStructureType::RenderPassBeginInfo,
		p_next: null(),
		render_pass: render_pass,
		framebuffer: frame_buffer,
		render_area: VkRect2D {
			offset: VkOffset2D { x: 0, y: 0 },
			extent: VkExtent2D { width, height },
		},
		clear_value_count: clear_value.len() as u32,
		p_clear_values: clear_value.as_ptr(),
	};
	(connection.begin_render)(
		connection.command_buffer,
		&render_pass_begin_info,
		VkSubpassContents::Inline
	);

	// take care of dynamic state:
	let viewport = VkViewport {
		x: 0.0, y: 0.0,
		width: width as f32,
		height: height as f32,
		min_depth: 0.0,
		max_depth: 1.0,
	};

	(connection.set_viewport)(connection.command_buffer, 0, 1, &viewport);

	let scissor = VkRect2D {
		offset: VkOffset2D { x: 0, y: 0 },
		extent: VkExtent2D { width, height },
	};

	(connection.set_scissor)(connection.command_buffer, 0, 1, &scissor);
}

pub unsafe fn end_render_pass(connection: &mut Vk) {
	let connection = connection.0.data();

	(connection.end_render_pass)(connection.command_buffer);
}

pub unsafe fn pipeline_barrier(connection: &mut Vk, image: VkImage) {
	let connection = connection.0.data();

	let barrier = VkImageMemoryBarrier {
		s_type: VkStructureType::ImageMemoryBarrier,
		p_next: null(),
		src_access_mask: VkAccess::ColorAttachmentWriteBit,
		dst_access_mask: VkAccess::MemoryReadBit,
		old_layout: VkImageLayout::Undefined, // ColorAttachmentOptimal,
		new_layout: VkImageLayout::PresentSrc,
		src_queue_family_index: !0,
		dst_queue_family_index: !0,
		image: image,
		subresource_range: VkImageSubresourceRange {
			aspect_mask: VkImageAspectFlags::Color,
			base_mip_level: 0,
			level_count: 1,
			base_array_layer: 0,
			layer_count: 1,
		},
	};

	(connection.pipeline_barrier)(
		connection.command_buffer,
		VkPipelineStage::AllCommands, 
		VkPipelineStage::BottomOfPipe, 
		0, 0, null(), 0, null(), 1, &barrier);
}

pub unsafe fn get_next_image(vulkan: &mut Vk,
	presenting_complete_sem: &mut VkSemaphore, swapchain: VkSwapchainKHR)
	-> u32
{
//	let connection = vulkan.0.data();

	let mut image_id = mem::uninitialized();

	let mut result = (vulkan.0.data().get_next_image)(
		vulkan.0.data().device,
		swapchain,
		u64::MAX,
		*presenting_complete_sem,
		mem::zeroed(),
		&mut image_id,
	);

	while result == VkResult::OutOfDateKhr {
		println!("OUt OF DAte");

		drop_semaphore(vulkan, *presenting_complete_sem);
		*presenting_complete_sem = new_semaphore(vulkan);

		result = (vulkan.0.data().get_next_image)(
			vulkan.0.data().device,
			swapchain,
			u64::MAX,
			*presenting_complete_sem,
			mem::zeroed(),
			&mut image_id,
		);
	}

	if result != VkResult::Success {
		panic!("vkAcquireNextImageKHR Failed!");
	}

	image_id
}

pub unsafe fn get_color_format(connection: &mut Vk) -> VkFormat {
	let connection = connection.0.data();

	// Load Function
	type VkGetPhysicalDeviceSurfaceFormatsKHR =
		unsafe extern "system" fn(VkPhysicalDevice, VkSurfaceKHR,
			*mut u32, *mut VkSurfaceFormatKHR) -> VkResult;
	let function_name = b"vkGetPhysicalDeviceSurfaceFormatsKHR\0";
	let get_gpu_surface_formats: VkGetPhysicalDeviceSurfaceFormatsKHR
		= vulkan::sym(connection, function_name);

	// Set Data
	let mut nformats = 1;
	let mut format = mem::uninitialized();

	// Run Function
	get_gpu_surface_formats(connection.gpu, connection.surface,
		&mut nformats, &mut format).unwrap();

	// Process data
	VkFormat::B8g8r8a8Unorm
}

pub unsafe fn get_buffering(connection: &mut Vk) -> u32 {
	let connection = connection.0.data();

	// Set Data
	let mut surface_info = mem::uninitialized();

	// Run Function
	(connection.get_surface_capabilities)(connection.gpu,
		connection.surface, &mut surface_info).unwrap();

	// Process data
	let min = surface_info.min_image_count;
	let max = surface_info.max_image_count;
	let image_count;

	if min >= max {
		// Gotta use at least the minimum.
		image_count = min;
	}else{
		// If double-buffering isn't supported, use single-buffering.
		if max < 2 {
			image_count = 1;
		} else {
			image_count = 2;
		}
	}

	/*match image_count {
		1 => println!("< adi_gpu: Buffering: Single"),
		2 => println!("< adi_gpu: Buffering: Double"),
		3 => println!("< adi_gpu: Buffering: Triple"),
		_ => panic!("< adi_gpu: Image Count: {}", image_count)
	}*/

	image_count
}

pub unsafe fn get_present_mode(connection: &mut Vk) -> VkPresentModeKHR {
	let connection = connection.0.data();

	// Load Function
	type VkGetPresentModes = extern "system" fn(VkPhysicalDevice,
		VkSurfaceKHR, *mut u32, *mut VkPresentModeKHR) -> VkResult;
	let vk_get_present_modes: VkGetPresentModes = vulkan::sym(connection,
		b"vkGetPhysicalDeviceSurfacePresentModesKHR\0");

	// Set Data
	let mut npresentmodes = mem::uninitialized();

	// Run Function
	vk_get_present_modes(connection.gpu, connection.surface, &mut npresentmodes,
		null_mut()).unwrap();

	// Set Data
	let npresentmodes_usize = npresentmodes as usize;
	let mut present_modes = vec![VkPresentModeKHR::Immediate;
		npresentmodes_usize];

	// Run Function
	vk_get_present_modes(connection.gpu, connection.surface,
		&mut npresentmodes, present_modes.as_mut_ptr()).unwrap();

	// Process Data
	for i in 0..npresentmodes_usize {
		if present_modes[i] == VkPresentModeKHR::Mailbox {
			return VkPresentModeKHR::Mailbox; // optimal
		}
	}

	// Fallback
	VkPresentModeKHR::Fifo
}

#[inline(always)] pub unsafe fn copy_image(connection: &mut Vk,
	src_image: &Image, dst_image: &Image, width: u32, height: u32)
{
	let connection = connection.0.data();

	(connection.copy_image)(
		connection.command_buffer,
		src_image.image().0, VkImageLayout::TransferSrcOptimal,
		dst_image.image().0, VkImageLayout::TransferDstOptimal, 1,
		&VkImageCopy {
			src_subresource: VkImageSubresourceLayers {
				aspect_mask: VkImageAspectFlags::Color,
				mip_level: 0,
				base_array_layer: 0,
				layer_count: 1,
			},
			src_offset: VkOffset3D { x: 0, y: 0, z: 0 },
			dst_subresource: VkImageSubresourceLayers {
				aspect_mask: VkImageAspectFlags::Color,
				mip_level: 0,
				base_array_layer: 0,
				layer_count: 1,
			},
			dst_offset: VkOffset3D { x: 0, y: 0, z: 0 },
			extent: VkExtent3D { width, height, depth: 1 },
		}
	);
}

#[inline(always)] pub unsafe fn create_swapchain(
	connection: &mut Vk, swapchain: &mut VkSwapchainKHR, width: u32,
	height: u32, image_count: &mut u32, color_format: VkFormat,
	present_mode: VkPresentModeKHR, swap_images: *mut VkImage)
{
	let connection = connection.0.data();

	let surface = connection.surface;

	(connection.get_surface_capabilities)(connection.gpu, connection.surface,
		&mut mem::uninitialized()).unwrap();

	(connection.new_swapchain)(
		connection.device,
		&VkSwapchainCreateInfoKHR {
			s_type: VkStructureType::SwapchainCreateInfo,
			p_next: null(),
			flags: 0,
			surface,
			min_image_count: *image_count,
			image_format: color_format,
			image_color_space: VkColorSpaceKHR::SrgbNonlinearKhr,
			image_extent: VkExtent2D { width, height },
			image_array_layers: 1,
			image_usage: VkImageUsage::ColorAttachmentBit,
			image_sharing_mode: VkSharingMode::Exclusive,
			pre_transform: VkSurfaceTransformFlagBitsKHR::Identity,
			composite_alpha: VkCompositeAlphaFlagBitsKHR::Opaque,
			present_mode: present_mode,
			clipped: 1/*do the clipping rendering optimization*/,
			old_swapchain: mem::zeroed(), // vulkan->swapchain,
			queue_family_index_count: 0,
			p_queue_family_indices: null(),
		},
		null(),
		swapchain
	).unwrap();

	(connection.get_swapcount)(connection.device, *swapchain, image_count,
		null_mut()).unwrap();
	(connection.get_swapcount)(connection.device, *swapchain, image_count,
		swap_images).unwrap();
}

pub unsafe fn create_img_view(connection: &mut Vk, image: VkImage,
	format: VkFormat, has_color: bool) -> VkImageView
{
	let connection = connection.0.data();

	let mut image_view = mem::uninitialized();

	let (components, aspect_mask) = if has_color {
		(
			VkComponentMapping {
				r: VkComponentSwizzle::R,
				g: VkComponentSwizzle::G,
				b: VkComponentSwizzle::B,
				a: VkComponentSwizzle::A,
			},
			VkImageAspectFlags::Color
		)
	} else {
		(
			VkComponentMapping {
				r: VkComponentSwizzle::Identity,
				g: VkComponentSwizzle::Identity,
				b: VkComponentSwizzle::Identity,
				a: VkComponentSwizzle::Identity,
			},
			VkImageAspectFlags::Depth
		)
	};

	(connection.create_imgview)(
		connection.device,
		&VkImageViewCreateInfo {
			s_type: VkStructureType::ImageViewCreateInfo,
			p_next: null(),
			flags: 0,
			view_type: VkImageViewType::SingleLayer2d,
			format: format.clone(),
			components,
			subresource_range: VkImageSubresourceRange {
				aspect_mask,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			},
			image,
		},
		null(),
		&mut image_view
	).unwrap();

	image_view
}

pub unsafe fn end_cmdbuff(connection: &mut Vk) {
	let connection = connection.0.data();

	(connection.end_cmdbuff)(connection.command_buffer).unwrap();
}

pub unsafe fn queue_submit(connection: &mut Vk, submit_fence: &Fence,
	pipelane_stage: VkPipelineStage, queue: VkQueue,
	semaphore: Option<VkSemaphore>)
{
	let connection = connection.0.data();

	(connection.queue_submit)(
		queue,
		1,
		&VkSubmitInfo {
			s_type: VkStructureType::SubmitInfo,
			p_next: null(),
			wait_semaphore_count: 0,
			wait_semaphores: null(),
			wait_dst_stage_mask: &pipelane_stage,
			command_buffer_count: 1,
			p_command_buffers: &connection.command_buffer,
			signal_semaphore_count: if semaphore.is_none() { 0 }
				else { 1 },
			p_signal_semaphores: if let Some(ref sem) = semaphore {
				sem
			} else {
				null()
			},
		},
		submit_fence.fence()
	).unwrap();
}

pub unsafe fn wait_fence(connection: &mut Vk, fence: &Fence) {
	let connection = connection.0.data();

	(connection.wait_fence)(connection.device, 1, &fence.fence(), 1,
		u64::MAX).unwrap();
}

#[inline(always)] pub unsafe fn create_image_view(
	vulkan: &mut Vk, color_format: &VkFormat, image_count: u32,
	swap_images: &mut [VkImage; 2], image_views: &mut [VkImageView; 2],
	present_queue: VkQueue) -> Fence
{
	let submit_fence = Fence::new(vulkan);

	for i in 0..(image_count as usize) {
		(vulkan.0.data().begin_cmdbuff)(
			vulkan.0.data().command_buffer,
			&VkCommandBufferBeginInfo {
				s_type: VkStructureType::CommandBufferBeginInfo,
				p_next: null(),
				flags: VkCommandBufferUsage::OneTimeSubmitBit,
				p_inheritance_info: null(),
			}
		).unwrap();

		(vulkan.0.data().pipeline_barrier)(
			vulkan.0.data().command_buffer,
			VkPipelineStage::TopOfPipe, 
			VkPipelineStage::TopOfPipe,
			0, 0, null(), 0, null(), 1,
			&VkImageMemoryBarrier {
				s_type: VkStructureType::ImageMemoryBarrier,
				p_next: null(),
				src_access_mask: VkAccess::NoFlags,
				dst_access_mask: VkAccess::MemoryReadBit,
				old_layout: VkImageLayout::Undefined,
				new_layout: VkImageLayout::PresentSrc,
				src_queue_family_index: !0,
				dst_queue_family_index: !0,
				image: swap_images[i],
				subresource_range: VkImageSubresourceRange {
					aspect_mask: VkImageAspectFlags::Color,
					base_mip_level: 0,
					level_count: 1,
					base_array_layer: 0,
					layer_count: 1,
				},
			}
		);

		end_cmdbuff(vulkan);
		queue_submit(vulkan, &submit_fence,
			VkPipelineStage::ColorAttachmentOutput, present_queue,
			None);
		wait_fence(vulkan, &submit_fence);

		(vulkan.0.data().reset_fence)(vulkan.0.data().device, 1,
			&submit_fence.fence()).unwrap();
		(vulkan.0.data().reset_cmdbuff)(vulkan.0.data().command_buffer, 0);

		image_views[i] = create_img_view(vulkan, swap_images[i],
			color_format.clone(), true);
	}

	submit_fence
}

#[inline(always)] pub unsafe fn create_ms_buffer(
	vulkan: &mut Vk, color_format: &VkFormat, width: u32, height: u32)
	-> Image
{
	Image::new(vulkan, width, height, color_format.clone(),
		VkImageTiling::Optimal, VkImageUsage::TransientColorAttachment,
		VkImageLayout::Undefined, 0, VK_SAMPLE_COUNT)
}

#[inline(always)] pub unsafe fn create_depth_buffer(
	vulkan: &mut Vk, submit_fence: &Fence, present_queue: VkQueue,
	width: u32, height: u32) -> Image
{
//	let connection = vulkan.0.data();

	let image = Image::new(vulkan, width, height, VkFormat::D16Unorm,
		VkImageTiling::Optimal, VkImageUsage::DepthStencilAttachmentBit,
		VkImageLayout::Undefined, 0, VK_SAMPLE_COUNT);

	// before using this depth buffer we must change it's layout:
	(vulkan.0.data().begin_cmdbuff)(
		vulkan.0.data().command_buffer,
		&VkCommandBufferBeginInfo {
			s_type: VkStructureType::CommandBufferBeginInfo,
			p_next: null(),
			flags: VkCommandBufferUsage::OneTimeSubmitBit,
			p_inheritance_info: null(),
		}
	).unwrap();

	(vulkan.0.data().pipeline_barrier)(
		vulkan.0.data().command_buffer, 
		VkPipelineStage::TopOfPipe, 
		VkPipelineStage::TopOfPipeAndEarlyFragmentTests,
		0,
		0,
		null(),
		0,
		null(),
		1,
		&VkImageMemoryBarrier {
			s_type: VkStructureType::ImageMemoryBarrier,
			p_next: null(),
			src_access_mask: VkAccess::NoFlags,
			dst_access_mask:
				VkAccess::DepthStencilAttachmentReadWrite,
			old_layout: VkImageLayout::Undefined,
			new_layout:
				VkImageLayout::DepthStencilAttachmentOptimal,
			src_queue_family_index: !0,
			dst_queue_family_index: !0,
			image: image.image().0,
			subresource_range: VkImageSubresourceRange {
				aspect_mask: VkImageAspectFlags::Depth,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			},
		}
	);

	end_cmdbuff(vulkan);
	queue_submit(vulkan, &submit_fence,
		VkPipelineStage::ColorAttachmentOutput, present_queue, None);
	wait_fence(vulkan, &submit_fence);

	(vulkan.0.data().reset_fence)(vulkan.0.data().device, 1,
		&submit_fence.fence()).unwrap();
	(vulkan.0.data().reset_cmdbuff)(vulkan.0.data().command_buffer, 0);

	image
}

#[inline(always)] pub unsafe fn create_render_pass(
	connection: &mut Vk, color_format: &VkFormat)
	-> VkRenderPass
{
	let connection = connection.0.data();
	let mut render_pass = mem::uninitialized();

	(connection.new_renderpass)(
		connection.device,
		&VkRenderPassCreateInfo {
			s_type: VkStructureType::RenderPassCreateInfo,
			p_next: null(),
			flags: 0,
			attachment_count: 3,
			attachments: [
				// Itermediary
				VkAttachmentDescription {
					flags: 0,
					format: color_format.clone(),
					samples: VK_SAMPLE_COUNT,
					load_op: VkAttachmentLoadOp::Clear,
					store_op: VkAttachmentStoreOp::DontCare,
					stencil_load_op:
						VkAttachmentLoadOp::DontCare,
					stencil_store_op:
						VkAttachmentStoreOp::DontCare,
					initial_layout:
					  VkImageLayout::Undefined,
					final_layout:
					  VkImageLayout::ColorAttachmentOptimal,
				},
				// Depth Buffer
				VkAttachmentDescription {
					flags: 0,
					format: VkFormat::D16Unorm,
					samples: VK_SAMPLE_COUNT,
					load_op: VkAttachmentLoadOp::Clear,
					store_op: VkAttachmentStoreOp::DontCare,
					stencil_load_op:
						VkAttachmentLoadOp::DontCare,
					stencil_store_op:
						VkAttachmentStoreOp::DontCare,
					initial_layout:
					 VkImageLayout::DepthStencilAttachmentOptimal,
					final_layout:
					 VkImageLayout::DepthStencilAttachmentOptimal,
				},
				// Color Buffer
				VkAttachmentDescription {
					flags: 0,
					format: color_format.clone(),
					samples: VkSampleCount::Sc1,
					load_op: VkAttachmentLoadOp::DontCare,
					store_op: VkAttachmentStoreOp::Store,
					stencil_load_op:
						VkAttachmentLoadOp::DontCare,
					stencil_store_op:
						VkAttachmentStoreOp::DontCare,
					initial_layout:
					  VkImageLayout::Undefined,
					final_layout:
					  VkImageLayout::PresentSrc,
				},
			].as_ptr(),
			subpass_count: 1,
			subpasses: &VkSubpassDescription {
				flags: 0,
				pipeline_bind_point: VkPipelineBindPoint::Graphics,
				color_attachment_count: 1,
				color_attachments: &VkAttachmentReference {
					attachment: 0,
					layout:
					  VkImageLayout::ColorAttachmentOptimal,
				},
				depth_stencil_attachment: &VkAttachmentReference
				{
					attachment: 1,
					layout:
					 VkImageLayout::DepthStencilAttachmentOptimal,
				},
				input_attachment_count: 0,
				input_attachments: null(),
				preserve_attachment_count: 0,
				preserve_attachments: null(),
				resolve_attachments: &VkAttachmentReference
				{
					attachment: 2,
					layout:
					 VkImageLayout::PresentSrc,
				},
			},
			dependency_count: 1,
			dependencies: &VkSubpassDependency {
				src_subpass: !0,
				dst_subpass: 0,
				src_stage_mask: VkPipelineStage::ColorAttachmentOutput,
				dst_stage_mask: VkPipelineStage::ColorAttachmentOutput,
				src_access_mask: VkAccess::ColorAttachmentWriteBit,
				dst_access_mask: VkAccess::ColorAttachmentReadWrite,
				dependency_flags: 0,
			},
		},
		null(),
		&mut render_pass
	).unwrap();

	render_pass
}

#[inline(always)] pub unsafe fn create_framebuffers(
	connection: &mut Vk, image_count: u32,
	render_pass: VkRenderPass, present_imgviews: &[VkImageView],
	multisample_img: &Image, depth_img: &Image,
	width: u32, height: u32, fbs: &mut[VkFramebuffer])
{
	let connection = connection.0.data();

	// create a framebuffer per swap chain imageView:
	for i in 0..(image_count as usize) {
		(connection.create_framebuffer)(
			connection.device,
			&VkFramebufferCreateInfo {
				s_type: VkStructureType::FramebufferCreateInfo,
				p_next: null(),
				flags: 0,
				attachment_count: 3,
				attachments: [
					multisample_img.image().2,
					depth_img.image().2,
					present_imgviews[i],
				].as_ptr(),
				layers: 1,
				render_pass, width, height,
			},
			null(),
			&mut fbs[i]
		).unwrap();
	}
}

#[inline(always)] pub unsafe fn destroy_swapchain(
	connection: &mut Vk, frame_buffers: &[VkFramebuffer],
	present_imgviews: &[VkImageView], render_pass: VkRenderPass,
	image_count: u32, swapchain: VkSwapchainKHR)
{
	let connection = connection.0.data();
	let device = connection.device;

	// Free framebuffers & present image views
	for i in 0..(image_count as usize) {
		(connection.drop_framebuffer)(device, frame_buffers[i],
			null());
		(connection.drop_imgview)(device, present_imgviews[i],
			null());
	}
	// Free render pass
	(connection.drop_renderpass)(device, render_pass, null());
	// Free swapchain
	(connection.drop_swapchain)(device, swapchain, null());
}

pub unsafe fn vw_camera_new(connection: &mut Vk,
	fog_color: (f32, f32, f32, f32), range: (f32, f32))
	 -> (Memory<TransformUniform>, Memory<FogUniform>)
{
	let ucamera_memory = Memory::new(connection,
		TransformUniform {
			mat4: [
				1.0, 0.0, 0.0, 0.0,
				0.0, 1.0, 0.0, 0.0,
				0.0, 0.0, 1.0, 0.0,
				0.0, 0.0, 0.0, 1.0
			],
		}
	);

	let ueffect_memory = Memory::new(connection,
		FogUniform {
			fogc: [fog_color.0, fog_color.1, fog_color.2, fog_color.3],
			fogr: [range.0, range.1],
		}
	);

	(ucamera_memory, ueffect_memory)
}

pub unsafe fn new_buffer(vulkan: &mut Vk, vertices: &[f32]) -> Buffer {
	Buffer::new(vulkan, vertices, BufferBuilderType::Vertex)
}

pub struct ShaderModule(
	VkShaderModule,
	// TODO: Don't
	VkDevice,
	unsafe extern "system" fn(VkDevice, VkShaderModule, *const c_void) -> (),
);

impl ShaderModule {
	/// Load a new shader module into memory.
	pub fn new(connection: &mut Vk, spirv_shader: &[u8]) -> ShaderModule {
		let connection = connection.0.data();

		let mut shader = unsafe { mem::uninitialized() };

		unsafe {
			(connection.new_shademod)(
				connection.device,
				&VkShaderModuleCreateInfo {
					s_type: VkStructureType::ShaderModuleCreateInfo,
					next: null(),
					flags: 0,
					code_size: spirv_shader.len(),
					code: spirv_shader.as_ptr(),
				},
				null(),
				&mut shader
			).unwrap();
		}

		ShaderModule(shader, connection.device, connection.drop_shademod)
	}
}

impl Drop for ShaderModule {
	fn drop(&mut self) -> () {
		unsafe {
			(self.2)(self.1, self.0, null());
		}
	}
}
