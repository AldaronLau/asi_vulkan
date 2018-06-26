// "asi_vulkan" - Aldaron's System Interface - Vulkan
//
// Copyright Jeron A. Lau 2018.
// Distributed under the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

use { std, std::{ mem, ptr::{ null, null_mut }, ffi::CString } };
use c_void;

use types::*;

use std::{ rc::Rc, cell::RefCell };

// Windows
#[cfg(target_os = "windows")]
const DL: &'static str = "vulkan-1.dll";

// Unix (Except MacOS)
#[cfg(all(unix, not(target_os = "macos")))]
const DL: &'static str = "libvulkan.so.1";

// MacOS
#[cfg(target_os = "macos")]
const DL: &'static str = "libMoltenVK.dylib";

#[inline(always)]
pub(crate) unsafe fn vk_sym<T>(vk: VkInstance, lib: &VulkanApi, name: &[u8])
	-> Result<T, String>
{
	let fn_ptr = (lib.vkGetInstanceProcAddr)(vk,
		&name[0] as *const _ as *const i8);

	if fn_ptr.is_null() {
		Err(format!("couldn't load symbol {}!",
			std::str::from_utf8(name).unwrap()))
	} else {
		Ok(mem::transmute_copy::<*mut c_void, T>(&fn_ptr))
	}
}

unsafe fn vkd_sym<T>(device: VkDevice, vkdsym: unsafe extern "system" fn(
	VkDevice, *const i8) -> *mut c_void, name: &[u8]) -> T
{
	let fn_ptr = vkdsym(device, &name[0] as *const _ as *const i8);

	if fn_ptr.is_null() {
		panic!("couldn't load symbol {}!", std::str::from_utf8(name)
			.unwrap());
	}

	mem::transmute_copy::<*mut c_void, T>(&fn_ptr)
}

pub(crate) unsafe fn sym<T>(vk: &VulkanContext, name: &[u8])
	-> Result<T, String>
{
	vk_sym(vk.vk, &vk.api, name)
}

pub(crate) unsafe fn dsym<T>(vk: &VulkanContext, name: &[u8]) -> T {
	vkd_sym(vk.device, vk.vkdsym, name)
}

unsafe fn create_instance(vk_create_instance: unsafe extern "system" fn(
	*const VkInstanceCreateInfo, *mut c_void, *mut VkInstance) -> VkResult)
	-> VkInstance
{
	// This variables must be defined separately so it stays in scope.
	let validation = CString::new("VK_LAYER_LUNARG_standard_validation")
		.unwrap();
	let dump = CString::new("VK_LAYER_LUNARG_api_dump").unwrap();
	let s1 = CString::new("VK_KHR_surface").unwrap();
	let s2 = CString::new(
		if cfg!(target_os = "linux") {
			"VK_KHR_xcb_surface"
		} else if cfg!(target_os = "android") {
			"VK_KHR_android_surface"
		} else if cfg!(target_os = "windows") {
			"VK_KHR_win32_surface"
		} else {
			panic!("No suitable surface for this platform.")
		}
	).unwrap();
	let s3 = CString::new("VK_EXT_debug_report").unwrap();
	let extnames = [s1.as_ptr(), s2.as_ptr(), s3.as_ptr()];
	let layernames = [validation.as_ptr(), dump.as_ptr()];

	let mut instance = mem::uninitialized();
	
	vk_create_instance(
		&VkInstanceCreateInfo {
			s_type: VkStructureType::InstanceCreateInfo,
			p_next: null_mut(),
			flags: 0,
			p_application_info: &VkApplicationInfo {
				s_type: VkStructureType::ApplicationInfo,
				p_next: null_mut(),
				p_application_name: null(),
				application_version: 3,
				p_engine_name: null(),
				engine_version: 3,
				api_version: 4194304, // 1.0.0
			},
			enabled_layer_count: {
				if cfg!(feature = "checks") { 1/*2*/ } else { 0 }
			},
			pp_enabled_layer_names: {
				if cfg!(feature = "checks") {
					layernames.as_ptr()
				} else {
					null()
				}
			},
			enabled_extension_count: {
				if cfg!(feature = "checks") { 3 } else { 2 }
			},
			pp_enabled_extension_names: extnames.as_ptr(),
		}, null_mut(), &mut instance
	).unwrap();

	if cfg!(feature = "checks") {
		println!("< Checks Enabled");
	}

	instance
}

dl_api!(VulkanApi, DL,
	fn vkGetInstanceProcAddr(VkInstance, *const i8) -> *mut c_void
);

/// The Vulkan context.
#[derive(Clone)] pub struct Vulkan(Rc<RefCell<VulkanContext>>);

/// The Vulkan context.
pub struct VulkanContext {
	pub(crate) vk: VkInstance,
	pub(crate) surface: VkSurfaceKHR,
	pub(crate) gpu: VkPhysicalDevice,
	pub(crate) pqi: u32,
	pub(crate) sampled: bool,
	pub(crate) device: VkDevice,
	pub(crate) command_buffer: VkCommandBuffer,
	pub(crate) command_pool: u64,
	pub(crate) sampler: VkSampler,
	pub(crate) api: VulkanApi,
	pub(crate) vkdsym: unsafe extern "system" fn(VkDevice, *const i8) -> *mut c_void,
	pub(crate) mapmem: unsafe extern "system" fn(VkDevice, VkDeviceMemory,
		VkDeviceSize, VkDeviceSize, VkFlags, *mut *mut c_void)
		-> VkResult,
	pub(crate) draw: unsafe extern "system" fn(VkCommandBuffer, u32, u32, u32, u32)
		-> (),
	pub(crate) unmap: unsafe extern "system" fn(VkDevice, VkDeviceMemory) -> (),
	pub(crate) new_swapchain: unsafe extern "system" fn(VkDevice,
		*const VkSwapchainCreateInfoKHR, *const c_void,
		*mut VkSwapchainKHR) -> VkResult,
	pub(crate) get_swapcount: unsafe extern "system" fn(VkDevice, VkSwapchainKHR,
		*mut u32, *mut VkImage) -> VkResult,
	pub(crate) create_fence: unsafe extern "system" fn(VkDevice,
		*const VkFenceCreateInfo, *const c_void, *mut VkFence)
		-> VkResult,
	pub(crate) begin_cmdbuff: unsafe extern "system" fn(VkCommandBuffer,
		*const VkCommandBufferBeginInfo) -> VkResult,
	pub(crate) pipeline_barrier: unsafe extern "system" fn(VkCommandBuffer,
		VkPipelineStage, VkPipelineStage, VkFlags, u32,
		*const VkMemoryBarrier, u32, *const VkBufferMemoryBarrier, u32,
		*const VkImageMemoryBarrier) -> (),
	pub(crate) end_cmdbuff: unsafe extern "system" fn(VkCommandBuffer) -> VkResult,
	pub(crate) queue_submit: unsafe extern "system" fn(VkQueue, u32,
		*const VkSubmitInfo, VkFence) -> VkResult,
	pub(crate) wait_fence: unsafe extern "system" fn(VkDevice, u32, *const VkFence,
		VkBool32, u64) -> VkResult,
	pub(crate) reset_fence: unsafe extern "system" fn(VkDevice, u32, *const VkFence)
		-> VkResult,
	pub(crate) reset_cmdbuff: unsafe extern "system" fn(VkCommandBuffer, VkFlags),
	pub(crate) create_imgview: unsafe extern "system" fn(VkDevice,
		*const VkImageViewCreateInfo, *const c_void, *mut VkImageView)
		-> VkResult,
	pub(crate) get_memprops: unsafe extern "system" fn(VkPhysicalDevice,
		*mut VkPhysicalDeviceMemoryProperties) -> (),
	pub(crate) create_image: unsafe extern "system" fn(VkDevice,
		*const VkImageCreateInfo, *const c_void, *mut VkImage)
		-> VkResult,
	pub(crate) get_imgmemreq: unsafe extern "system" fn(VkDevice, VkImage,
		*mut VkMemoryRequirements) -> (),
	pub(crate) mem_allocate: unsafe extern "system" fn(VkDevice,
		*const VkMemoryAllocateInfo, *const c_void, *mut VkDeviceMemory)
		-> VkResult,
	pub(crate) bind_imgmem: unsafe extern "system" fn(VkDevice, VkImage,
		VkDeviceMemory, VkDeviceSize) -> VkResult,
	pub(crate) new_renderpass: unsafe extern "system" fn(VkDevice,
		*const VkRenderPassCreateInfo, *const c_void, *mut VkRenderPass)
		-> VkResult,
	pub(crate) create_framebuffer: unsafe extern "system" fn(VkDevice,
		*const VkFramebufferCreateInfo, *const c_void, *mut VkFramebuffer)
		-> VkResult,
	pub(crate) drop_framebuffer: unsafe extern "system" fn(VkDevice, VkFramebuffer,
		*const c_void) -> (),
	pub(crate) drop_imgview: unsafe extern "system" fn(VkDevice, VkImageView,
		*const c_void) -> (),
	pub(crate) drop_renderpass: unsafe extern "system" fn(VkDevice, VkRenderPass,
		*const c_void) -> (),
	pub(crate) drop_image: unsafe extern "system" fn(VkDevice, VkImage, *const c_void)
		-> (),
	pub(crate) drop_buffer: unsafe extern "system" fn(VkDevice, VkBuffer,
		*const c_void) -> (),
	pub(crate) drop_memory: unsafe extern "system" fn(VkDevice,
		VkDeviceMemory, *const c_void) -> (),
	pub(crate) drop_swapchain: unsafe extern "system" fn(VkDevice, VkSwapchainKHR,
		*const c_void) -> (),
	pub(crate) update_descsets: unsafe extern "system" fn(VkDevice, u32,
		*const VkWriteDescriptorSet, u32, *const c_void) -> (),
	pub(crate) drop_descpool: unsafe extern "system" fn(VkDevice, VkDescriptorPool,
		*const c_void) -> (),
	pub(crate) bind_buffer_mem: unsafe extern "system" fn(VkDevice, VkBuffer,
		VkDeviceMemory, VkDeviceSize) -> VkResult,
	pub(crate) get_bufmemreq: unsafe extern "system" fn(VkDevice, VkBuffer,
		*mut VkMemoryRequirements) -> (),
	pub(crate) new_buffer: unsafe extern "system" fn(VkDevice,
		*const VkBufferCreateInfo, *const c_void, *mut VkBuffer)
		-> VkResult,
	pub(crate) new_descpool: unsafe extern "system" fn(VkDevice,
		*const VkDescriptorPoolCreateInfo, *const c_void,
		*mut VkDescriptorPool) -> VkResult,
	pub(crate) new_descsets: unsafe extern "system" fn(VkDevice,
		*const VkDescriptorSetAllocateInfo, *mut VkDescriptorSet)
		-> VkResult,
	pub(crate) new_shademod: unsafe extern "system" fn(VkDevice,
		*const VkShaderModuleCreateInfo, *const c_void,
		*mut VkShaderModule) -> VkResult,
	pub(crate) drop_shademod: unsafe extern "system" fn(VkDevice, VkShaderModule,
		*const c_void) -> (),
	pub(crate) new_pipeline: unsafe extern "system" fn(VkDevice, VkPipelineCache, u32,
		*const VkGraphicsPipelineCreateInfo, *const c_void,
		*mut VkPipeline) -> VkResult,
	pub(crate) drop_pipeline: unsafe extern "system" fn(VkDevice,
		VkPipeline, *const c_void) -> (),
	pub(crate) new_pipeline_layout: unsafe extern "system" fn(VkDevice,
		*const VkPipelineLayoutCreateInfo, *const c_void,
		*mut VkPipelineLayout) -> VkResult,
	pub(crate) drop_pipeline_layout: unsafe extern "system" fn(VkDevice,
		VkPipelineLayout, *const c_void) -> (),
	pub(crate) new_descset_layout: unsafe extern "system" fn(VkDevice,
		*const VkDescriptorSetLayoutCreateInfo, *const c_void,
		*mut VkDescriptorSetLayout) -> VkResult,
	pub(crate) drop_descset_layout: unsafe extern "system" fn(VkDevice,
		VkDescriptorSetLayout, *const c_void) -> (),
	pub(crate) bind_vb: unsafe extern "system" fn(VkCommandBuffer, u32, u32,
		*const VkBuffer, *const VkDeviceSize) -> (),
	pub(crate) bind_pipeline: unsafe extern "system" fn(VkCommandBuffer,
		VkPipelineBindPoint, VkPipeline) -> (),
	pub(crate) bind_descsets: unsafe extern "system" fn(VkCommandBuffer,
		VkPipelineBindPoint, VkPipelineLayout, u32, u32,
		*const VkDescriptorSet, u32, *const u32) -> (),
	pub(crate) new_semaphore: unsafe extern "system" fn(VkDevice,
		*const VkSemaphoreCreateInfo, *const c_void, *mut VkSemaphore)
		-> VkResult,
	pub(crate) drop_semaphore: unsafe extern "system" fn(VkDevice, VkSemaphore,
		*const c_void) -> (),
	pub(crate) get_next_image: unsafe extern "system" fn(VkDevice, VkSwapchainKHR, u64,
		VkSemaphore, VkFence, *mut u32) -> VkResult,
	pub(crate) copy_image: unsafe extern "system" fn(VkCommandBuffer, VkImage,
		VkImageLayout, VkImage, VkImageLayout, u32, *const VkImageCopy)
		-> (),
	pub(crate) gpu_props: unsafe extern "system" fn(VkPhysicalDevice, VkFormat,
		*mut VkFormatProperties) -> (),
	pub(crate) subres_layout: unsafe extern "system" fn(VkDevice, VkImage,
		*const VkImageSubresource, *mut VkSubresourceLayout) -> (),
	pub(crate) new_sampler: unsafe extern "system" fn(VkDevice,
		*const VkSamplerCreateInfo, *const c_void, *mut VkSampler)
		-> VkResult,
	pub(crate) get_surface_capabilities: unsafe extern "system" fn(VkPhysicalDevice,
		VkSurfaceKHR, *mut VkSurfaceCapabilitiesKHR) -> VkResult,
	pub(crate) begin_render: unsafe extern "system" fn(VkCommandBuffer,
		*const VkRenderPassBeginInfo, VkSubpassContents) -> (),
	pub(crate) set_viewport: unsafe extern "system" fn(VkCommandBuffer, u32, u32,
		*const VkViewport) -> (),
	pub(crate) set_scissor: unsafe extern "system" fn(VkCommandBuffer, u32, u32,
		*const VkRect2D) -> (),
	pub(crate) end_render_pass: unsafe extern "system" fn(VkCommandBuffer) -> (),
	pub(crate) destroy_fence: unsafe extern "system" fn(VkDevice, VkFence, *const c_void)
		-> (),
	pub(crate) queue_present: unsafe extern "system" fn(VkQueue, *const VkPresentInfo) -> VkResult,
	pub(crate) wait_idle: unsafe extern "system" fn(VkDevice) -> VkResult,
}

impl Vulkan {
	pub fn new() -> Result<Vulkan, String> { unsafe {
		// Load the Vulkan library
		// TODO: use ? syntax
		let api = VulkanApi::new()?;

		let vk = create_instance(
			vk_sym(mem::zeroed(), &api, b"vkCreateInstance\0")?
		);

		Ok(Vulkan(Rc::new(RefCell::new(VulkanContext {
			vk,
			// Late inits.
			surface: ::std::mem::uninitialized(),
			gpu: ::std::mem::uninitialized(),
			pqi: ::std::mem::uninitialized(),
			sampled: ::std::mem::uninitialized(),
			device: ::std::mem::uninitialized(),
			command_buffer: ::std::mem::uninitialized(),
			command_pool: ::std::mem::uninitialized(),
			sampler: ::std::mem::uninitialized(),
			vkdsym: vk_sym(vk, &api, b"vkGetDeviceProcAddr\0")?,
			mapmem: vk_sym(vk, &api, b"vkMapMemory\0")?,
			draw: vk_sym(vk, &api, b"vkCmdDraw\0")?,
			unmap: vk_sym(vk, &api, b"vkUnmapMemory\0")?,
			new_swapchain: vk_sym(vk, &api, b"vkCreateSwapchainKHR\0")?,
			get_swapcount: vk_sym(vk, &api, b"vkGetSwapchainImagesKHR\0")?,
			create_fence: vk_sym(vk, &api, b"vkCreateFence\0")?,
			begin_cmdbuff: vk_sym(vk, &api, b"vkBeginCommandBuffer\0")?,
			pipeline_barrier: vk_sym(vk, &api, b"vkCmdPipelineBarrier\0")?,
			end_cmdbuff: vk_sym(vk, &api, b"vkEndCommandBuffer\0")?,
			queue_submit: vk_sym(vk, &api, b"vkQueueSubmit\0")?,
			wait_fence: vk_sym(vk, &api, b"vkWaitForFences\0")?,
			reset_fence: vk_sym(vk, &api, b"vkResetFences\0")?,
			reset_cmdbuff: vk_sym(vk, &api, b"vkResetCommandBuffer\0")?,
			create_imgview: vk_sym(vk, &api, b"vkCreateImageView\0")?,
			get_memprops: vk_sym(vk, &api,
				b"vkGetPhysicalDeviceMemoryProperties\0")?,
			create_image: vk_sym(vk, &api, b"vkCreateImage\0")?,
			get_imgmemreq: vk_sym(vk, &api,
				b"vkGetImageMemoryRequirements\0")?,
			mem_allocate: vk_sym(vk, &api, b"vkAllocateMemory\0")?,
			bind_imgmem: vk_sym(vk, &api, b"vkBindImageMemory\0")?,
			new_renderpass: vk_sym(vk, &api, b"vkCreateRenderPass\0")?,
			create_framebuffer: vk_sym(vk, &api, b"vkCreateFramebuffer\0")?,
			drop_framebuffer: vk_sym(vk, &api, b"vkDestroyFramebuffer\0")?,
			drop_imgview: vk_sym(vk, &api, b"vkDestroyImageView\0")?,
			drop_renderpass: vk_sym(vk, &api, b"vkDestroyRenderPass\0")?,
			drop_image: vk_sym(vk, &api, b"vkDestroyImage\0")?,
			drop_buffer: vk_sym(vk, &api, b"vkDestroyBuffer\0")?,
			drop_memory: vk_sym(vk, &api, b"vkFreeMemory\0\0")?,
			drop_swapchain: vk_sym(vk, &api, b"vkDestroySwapchainKHR\0")?,
			update_descsets: vk_sym(vk, &api, b"vkUpdateDescriptorSets\0")?,
			drop_descpool: vk_sym(vk, &api, b"vkDestroyDescriptorPool\0")?,
			bind_buffer_mem: vk_sym(vk, &api, b"vkBindBufferMemory\0")?,
			get_bufmemreq: vk_sym(vk, &api,
				b"vkGetBufferMemoryRequirements\0")?,
			new_buffer: vk_sym(vk, &api, b"vkCreateBuffer\0")?,
			new_descpool: vk_sym(vk, &api, b"vkCreateDescriptorPool\0")?,
			new_descsets: vk_sym(vk, &api, b"vkAllocateDescriptorSets\0")?,
			new_shademod: vk_sym(vk, &api, b"vkCreateShaderModule\0")?,
			drop_shademod: vk_sym(vk, &api, b"vkDestroyShaderModule\0")?,
			new_pipeline: vk_sym(vk, &api, b"vkCreateGraphicsPipelines\0")?,
			drop_pipeline: vk_sym(vk, &api, b"vkDestroyPipeline\0")?,
			new_pipeline_layout:
				vk_sym(vk, &api, b"vkCreatePipelineLayout\0")?,
			drop_pipeline_layout: vk_sym(vk, &api,
				b"vkDestroyPipelineLayout\0")?,
			new_descset_layout:
				vk_sym(vk, &api, b"vkCreateDescriptorSetLayout\0")?,
			drop_descset_layout: vk_sym(vk, &api,
				b"vkDestroyDescriptorSetLayout\0")?,
			bind_vb: vk_sym(vk, &api, b"vkCmdBindVertexBuffers\0")?,
			bind_pipeline: vk_sym(vk, &api, b"vkCmdBindPipeline\0")?,
			bind_descsets: vk_sym(vk, &api, b"vkCmdBindDescriptorSets\0")?,
			new_semaphore: vk_sym(vk, &api, b"vkCreateSemaphore\0")?,
			drop_semaphore: vk_sym(vk, &api, b"vkDestroySemaphore\0")?,
			get_next_image: vk_sym(vk, &api, b"vkAcquireNextImageKHR\0")?,
			copy_image: vk_sym(vk, &api, b"vkCmdCopyImage\0")?,
			gpu_props: vk_sym(vk, &api,
				b"vkGetPhysicalDeviceFormatProperties\0")?,
			subres_layout:
				vk_sym(vk, &api, b"vkGetImageSubresourceLayout\0")?,
			new_sampler: vk_sym(vk, &api, b"vkCreateSampler\0")?,
			get_surface_capabilities: vk_sym(vk, &api,
				b"vkGetPhysicalDeviceSurfaceCapabilitiesKHR\0")?,
			begin_render: vk_sym(vk, &api, b"vkCmdBeginRenderPass\0")?,
			set_viewport: vk_sym(vk, &api, b"vkCmdSetViewport\0")?,
			set_scissor: vk_sym(vk, &api, b"vkCmdSetScissor\0")?,
			end_render_pass: vk_sym(vk, &api, b"vkCmdEndRenderPass\0")?,
			destroy_fence: vk_sym(vk, &api, b"vkDestroyFence\0")?,
			queue_present: vk_sym(vk, &api, b"vkQueuePresentKHR\0")?,
			wait_idle: vk_sym(vk, &api, b"vkDeviceWaitIdle\0")?,
			api,
		}))))
	} }

	pub(crate) fn get(&self) -> std::cell::Ref<VulkanContext> {
		self.0.borrow()
	}

	pub(crate) fn get_mut(&self) -> std::cell::RefMut<VulkanContext> {
		self.0.borrow_mut()
	}

	/// Whether or not images are sampled.
	pub fn sampled(&self) -> bool {
		self.get().sampled
	}
}

impl Drop for VulkanContext {
	fn drop(&mut self) -> () {
		// Load Function (Sampler)
		type VkDestroySampler = unsafe extern "system" fn(
			VkDevice, VkSampler, *const c_void) -> ();
		let destroy: VkDestroySampler = unsafe {
			sym(self, b"vkDestroySampler\0").unwrap()
		};

		// Run Function (Sampler)
		unsafe { destroy(self.device, self.sampler, null()) }

		// Load Function (Command Buffer & Command Pool)
		type VkDestroyCommandPool = unsafe extern "system" fn(
			VkDevice, u64, *const c_void) -> ();
		let destroy: VkDestroyCommandPool = unsafe {
			sym(self, b"vkDestroyCommandPool\0").unwrap()
		};

		// Run Function (Command Buffer & Command Pool)
		unsafe { destroy(self.device, self.command_pool, null()) }

		// Load Function (Surface)
		type VkDestroySurface = unsafe extern "system" fn(
			instance: VkInstance, surface: VkSurfaceKHR,
			pAllocator: *mut c_void) -> ();
		let destroy: VkDestroySurface = unsafe {
			sym(self, b"vkDestroySurfaceKHR\0").unwrap()
		};

		// Run Function (Surface)
		unsafe { destroy(self.vk, self.surface, null_mut()) }

		// Load Function
		type VkDestroyDevice = unsafe extern "system" fn(VkDevice,
			*const c_void) -> ();
		let destroy: VkDestroyDevice = unsafe {
			sym(self, b"vkDestroyDevice\0").unwrap()
		};

		// Run Function
		unsafe { destroy(self.device, null()) }

		// Load Function
		type VkDestroyInstance = unsafe extern "system" fn(
			instance: VkInstance, pAllocator: *mut c_void) -> ();
		let function_name = b"vkDestroyInstance\0";
		let destroy: VkDestroyInstance = unsafe {
			sym(self, function_name).unwrap()
		};

		// Run Function
		unsafe { destroy(self.vk, null_mut()) }
	}
}
