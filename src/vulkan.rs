// vulkan.rs -- Aldaron's System Interface / OpenGL
// Copyright (c) 2018  Jeron A. Lau <jeron.lau@plopgrizzly.com>
// Licensed under the MIT LICENSE

use ami::Parent;
use { std, std::{ mem, ptr::{ null, null_mut }, ffi::CString } };

use libc;
use libc::c_void;

use types::*;

// TODO: move to asi
#[cfg(target_os = "windows")]
extern "system" {
	fn LoadLibraryW(a: *const u16) -> *mut c_void /*HMODULE*/;
	fn GetProcAddress(b: *mut c_void/*HMODULE*/, c: *const u8)
		-> *mut c_void;
	fn FreeLibrary(a: *mut c_void/*HMODULE*/) -> i32 /*BOOL*/;
}

// TODO: move to asi ...
#[cfg(target_os = "windows")]
unsafe fn load_lib() -> *mut c_void {
//	let vulkan = if cfg!(target_pointer_width = "64") {
//		"C:\\Windows\\SysWOW64\\vulkan-1.dll";
//	} else {
//		"C:\\Windows\\System32\\vulkan-1.dll";
//	}
	let vulkan = "vulkan-1.dll\0";
	let vulkan16 : Vec<u16> = vulkan.encode_utf16().collect();
	let handle = LoadLibraryW(vulkan16.as_ptr());
	
	if handle.is_null() {
		panic!("failed to load vulkan-1.dll")
	} else {
		handle
	}
}

#[cfg(not(target_os = "windows"))]
unsafe fn load_lib() -> *mut c_void {
	let vulkan = b"libvulkan.so.1\0";

	libc::dlopen(&vulkan[0] as *const _ as *const i8, 1)
}

#[cfg(target_os = "windows")]
unsafe fn dl_sym<T>(lib: *mut c_void, name: &[u8]) -> T {
	let fn_ptr = GetProcAddress(lib, &name[0]);

	mem::transmute_copy::<*mut c_void, T>(&fn_ptr)
}

#[cfg(not(target_os = "windows"))]
unsafe fn dl_sym<T>(lib: *mut c_void, name: &[u8]) -> T {
	let fn_ptr = libc::dlsym(lib, &name[0] as *const _ as *const i8);

	mem::transmute_copy::<*mut c_void, T>(&fn_ptr)
}

#[inline(always)]
pub(crate) unsafe fn vk_sym<T>(vk: VkInstance, vksym: unsafe extern "system" fn(
	VkInstance, *const i8) -> *mut c_void, name: &[u8]) -> T
{
	let fn_ptr = vksym(vk, &name[0] as *const _ as *const i8);

	if fn_ptr.is_null() {
		panic!("couldn't load symbol {}!", std::str::from_utf8(name)
			.unwrap());
	}

	mem::transmute_copy::<*mut c_void, T>(&fn_ptr)
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

pub(crate) unsafe fn sym<T>(connection: &Vulkan, name: &[u8]) -> T {
	vk_sym(connection.vk, connection.vksym, name)
}

pub(crate) unsafe fn dsym<T>(connection: &Vulkan, name: &[u8]) -> T {
	vkd_sym(connection.device, connection.vkdsym, name)
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

/// The Vulkan context.
pub struct Vulkan {
	pub(crate) vk: VkInstance,
	pub(crate) surface: VkSurfaceKHR,
	pub(crate) gpu: VkPhysicalDevice,
	pub(crate) pqi: u32,
	pub(crate) sampled: bool,
	pub(crate) device: VkDevice,
	pub(crate) command_buffer: VkCommandBuffer,
	pub(crate) command_pool: u64,
	pub(crate) sampler: VkSampler,
	pub(crate) lib: *mut c_void,
	pub(crate) vksym: unsafe extern "system" fn(VkInstance, *const i8) -> *mut c_void,
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
	pub(crate) drop_descsets: unsafe extern "system" fn(VkDevice, VkDescriptorPool,
		u32, *const VkDescriptorSet) -> VkResult,
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
	pub unsafe fn new() -> Option<Vulkan> {
		let lib = load_lib();

		if lib.is_null() {
			return None; // Vulkan doesn't exist.
		}

		let vksym = dl_sym(lib, b"vkGetInstanceProcAddr\0");
	
		let vk = create_instance(
			vk_sym(mem::zeroed(), vksym, b"vkCreateInstance\0")
		);

		Some(Vulkan {
			vk, lib, vksym,
			// Late inits.
			surface: ::std::mem::uninitialized(),
			gpu: ::std::mem::uninitialized(),
			pqi: ::std::mem::uninitialized(),
			sampled: ::std::mem::uninitialized(),
			device: ::std::mem::uninitialized(),
			command_buffer: ::std::mem::uninitialized(),
			command_pool: ::std::mem::uninitialized(),
			sampler: ::std::mem::uninitialized(),
			vkdsym: vk_sym(vk, vksym, b"vkGetDeviceProcAddr\0"),
			mapmem: vk_sym(vk, vksym, b"vkMapMemory\0"),
			draw: vk_sym(vk, vksym, b"vkCmdDraw\0"),
			unmap: vk_sym(vk, vksym, b"vkUnmapMemory\0"),
			new_swapchain: vk_sym(vk, vksym, b"vkCreateSwapchainKHR\0"),
			get_swapcount: vk_sym(vk, vksym, b"vkGetSwapchainImagesKHR\0"),
			create_fence: vk_sym(vk, vksym, b"vkCreateFence\0"),
			begin_cmdbuff: vk_sym(vk, vksym, b"vkBeginCommandBuffer\0"),
			pipeline_barrier: vk_sym(vk, vksym, b"vkCmdPipelineBarrier\0"),
			end_cmdbuff: vk_sym(vk, vksym, b"vkEndCommandBuffer\0"),
			queue_submit: vk_sym(vk, vksym, b"vkQueueSubmit\0"),
			wait_fence: vk_sym(vk, vksym, b"vkWaitForFences\0"),
			reset_fence: vk_sym(vk, vksym, b"vkResetFences\0"),
			reset_cmdbuff: vk_sym(vk, vksym, b"vkResetCommandBuffer\0"),
			create_imgview: vk_sym(vk, vksym, b"vkCreateImageView\0"),
			get_memprops: vk_sym(vk, vksym,
				b"vkGetPhysicalDeviceMemoryProperties\0"),
			create_image: vk_sym(vk, vksym, b"vkCreateImage\0"),
			get_imgmemreq: vk_sym(vk, vksym,
				b"vkGetImageMemoryRequirements\0"),
			mem_allocate: vk_sym(vk, vksym, b"vkAllocateMemory\0"),
			bind_imgmem: vk_sym(vk, vksym, b"vkBindImageMemory\0"),
			new_renderpass: vk_sym(vk, vksym, b"vkCreateRenderPass\0"),
			create_framebuffer: vk_sym(vk, vksym, b"vkCreateFramebuffer\0"),
			drop_framebuffer: vk_sym(vk, vksym, b"vkDestroyFramebuffer\0"),
			drop_imgview: vk_sym(vk, vksym, b"vkDestroyImageView\0"),
			drop_renderpass: vk_sym(vk, vksym, b"vkDestroyRenderPass\0"),
			drop_image: vk_sym(vk, vksym, b"vkDestroyImage\0"),
			drop_buffer: vk_sym(vk, vksym, b"vkDestroyBuffer\0"),
			drop_memory: vk_sym(vk, vksym, b"vkFreeMemory\0\0"),
			drop_swapchain: vk_sym(vk, vksym, b"vkDestroySwapchainKHR\0"),
			update_descsets: vk_sym(vk, vksym, b"vkUpdateDescriptorSets\0"),
			drop_descsets: vk_sym(vk, vksym, b"vkFreeDescriptorSets\0"),
			drop_descpool: vk_sym(vk, vksym, b"vkDestroyDescriptorPool\0"),
			bind_buffer_mem: vk_sym(vk, vksym, b"vkBindBufferMemory\0"),
			get_bufmemreq: vk_sym(vk, vksym,
				b"vkGetBufferMemoryRequirements\0"),
			new_buffer: vk_sym(vk, vksym, b"vkCreateBuffer\0"),
			new_descpool: vk_sym(vk, vksym, b"vkCreateDescriptorPool\0"),
			new_descsets: vk_sym(vk, vksym, b"vkAllocateDescriptorSets\0"),
			new_shademod: vk_sym(vk, vksym, b"vkCreateShaderModule\0"),
			drop_shademod: vk_sym(vk, vksym, b"vkDestroyShaderModule\0"),
			new_pipeline: vk_sym(vk, vksym, b"vkCreateGraphicsPipelines\0"),
			drop_pipeline: vk_sym(vk, vksym, b"vkDestroyPipeline\0"),
			new_pipeline_layout:
				vk_sym(vk, vksym, b"vkCreatePipelineLayout\0"),
			drop_pipeline_layout: vk_sym(vk, vksym,
				b"vkDestroyPipelineLayout\0"),
			new_descset_layout:
				vk_sym(vk, vksym, b"vkCreateDescriptorSetLayout\0"),
			drop_descset_layout: vk_sym(vk, vksym,
				b"vkDestroyDescriptorSetLayout\0"),
			bind_vb: vk_sym(vk, vksym, b"vkCmdBindVertexBuffers\0"),
			bind_pipeline: vk_sym(vk, vksym, b"vkCmdBindPipeline\0"),
			bind_descsets: vk_sym(vk, vksym, b"vkCmdBindDescriptorSets\0"),
			new_semaphore: vk_sym(vk, vksym, b"vkCreateSemaphore\0"),
			drop_semaphore: vk_sym(vk, vksym, b"vkDestroySemaphore\0"),
			get_next_image: vk_sym(vk, vksym, b"vkAcquireNextImageKHR\0"),
			copy_image: vk_sym(vk, vksym, b"vkCmdCopyImage\0"),
			gpu_props: vk_sym(vk, vksym,
				b"vkGetPhysicalDeviceFormatProperties\0"),
			subres_layout:
				vk_sym(vk, vksym, b"vkGetImageSubresourceLayout\0"),
			new_sampler: vk_sym(vk, vksym, b"vkCreateSampler\0"),
			get_surface_capabilities: vk_sym(vk, vksym,
				b"vkGetPhysicalDeviceSurfaceCapabilitiesKHR\0"),
			begin_render: vk_sym(vk, vksym, b"vkCmdBeginRenderPass\0"),
			set_viewport: vk_sym(vk, vksym, b"vkCmdSetViewport\0"),
			set_scissor: vk_sym(vk, vksym, b"vkCmdSetScissor\0"),
			end_render_pass: vk_sym(vk, vksym, b"vkCmdEndRenderPass\0"),
			destroy_fence: vk_sym(vk, vksym, b"vkDestroyFence\0"),
			queue_present: vk_sym(vk, vksym, b"vkQueuePresentKHR\0"),
			wait_idle: vk_sym(vk, vksym, b"vkDeviceWaitIdle\0"),
		})
	}
}

pub enum VkType {
	Image,
	Sprite,
	Style,
	Buffer,
	Fence,
}

pub struct VkObject {
	vk_type: VkType,
	value_a: u64,
	value_b: u64,
	value_c: u64,
}

impl VkObject {
	pub fn new(vk_type: VkType, value_a: u64, value_b: u64, value_c: u64)
		-> Self
	{
		VkObject { vk_type, value_a, value_b, value_c }
	}

	pub fn fence(&self) -> u64 {
		self.value_a
	}

	pub fn image(&self) -> (u64, u64) { (self.value_a, self.value_b) }

	pub fn style(&self) -> (u64, u64, u64) {
		(self.value_a, self.value_b, self.value_c)
	}
}

impl ::ami::PseudoDrop for VkObject {
	type T = Vulkan;

	fn pdrop(&mut self, vulkan: &mut Vulkan) -> () {
		use VkType::*;

		match self.vk_type {
			Image => ::image::destroy(self.style(), vulkan),
			Sprite => ::sprite::destroy(self.image(), vulkan),
			Style => ::style::destroy(self.style(), vulkan),
			Buffer => ::memory::destroy(self.image(), vulkan),
			Fence => ::fence::destroy(self.fence(), vulkan),
		}
	}
}

/// The Vulkan Instance Handle
pub struct Vk(pub(crate) Parent<Vulkan, VkObject>);

impl Vk {
	/// Load the Vulkan library, returns None on failure.
	pub fn new() -> Option<Self> {
		unsafe { Some(Vk(Parent::new(Vulkan::new()?))) }
	}

	/// Get a pointer.
	pub fn as_ptr(&mut self) -> *mut Vulkan {
		self.0.data()
	}

	/// Whether or not images are sampled.
	pub fn sampled(&self) -> bool {
		self.0.data().sampled
	}
}

impl Drop for Vulkan {
	fn drop(&mut self) -> () {
		// Load Function (Sampler)
		type VkDestroySampler = unsafe extern "system" fn(
			VkDevice, VkSampler, *const c_void) -> ();
		let destroy: VkDestroySampler = unsafe {
			sym(self, b"vkDestroySampler\0")
		};

		// Run Function (Sampler)
		unsafe { destroy(self.device, self.sampler, null()) }

		// Load Function (Command Buffer & Command Pool)
		type VkDestroyCommandPool = unsafe extern "system" fn(
			VkDevice, u64, *const c_void) -> ();
		let destroy: VkDestroyCommandPool = unsafe {
			sym(self, b"vkDestroyCommandPool\0")
		};

		// Run Function (Command Buffer & Command Pool)
		unsafe { destroy(self.device, self.command_pool, null()) }

		// Load Function (Surface)
		type VkDestroySurface = unsafe extern "system" fn(
			instance: VkInstance, surface: VkSurfaceKHR,
			pAllocator: *mut c_void) -> ();
		let destroy: VkDestroySurface = unsafe {
			sym(self, b"vkDestroySurfaceKHR\0")
		};

		// Run Function (Surface)
		unsafe { destroy(self.vk, self.surface, null_mut()) }

		// Load Function
		type VkDestroyDevice = unsafe extern "system" fn(VkDevice,
			*const c_void) -> ();
		let destroy: VkDestroyDevice = unsafe {
			sym(self, b"vkDestroyDevice\0")
		};

		// Run Function
		unsafe { destroy(self.device, null()) }

		// Load Function
		type VkDestroyInstance = unsafe extern "system" fn(
			instance: VkInstance, pAllocator: *mut c_void) -> ();
		let function_name = b"vkDestroyInstance\0";
		let destroy: VkDestroyInstance = unsafe {
			sym(self, function_name)
		};

		// Run Function
		unsafe { destroy(self.vk, null_mut()) }
		// TODO: Drop lib in asi
		println!("FIXME: Drop dl {:?}", self.lib);
	}
}
