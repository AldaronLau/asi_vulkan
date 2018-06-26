// "asi_vulkan" - Aldaron's System Interface - Vulkan
//
// Copyright Jeron A. Lau 2018.
// Distributed under the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

use null;
use std::mem;

use Vulkan;
use types::*;
use std::{ rc::Rc };

/// A `VkFence` (TODO)
#[derive(Clone)] pub struct Fence(Rc<FenceContext>);

struct FenceContext {
	fence: u64,
	vulkan: Vulkan,
}

impl Fence {
	pub fn new(connection: &mut Vulkan) -> Self {
		Fence(Rc::new(FenceContext {
			fence: unsafe { create_fence(connection) },
			vulkan: connection.clone()
		}))
	}

	pub fn fence(&self) -> u64 {
		self.0.fence
	}
}

unsafe fn create_fence(connection: &mut Vulkan) -> VkFence {
	let connection = connection.get();

	let mut fence = mem::uninitialized();

	(connection.create_fence)(
		connection.device,
		&VkFenceCreateInfo {
			s_type: VkStructureType::FenceCreateInfo,
			p_next: null(),
			flags: 0,
		},
		null(),
		&mut fence
	).unwrap();

	fence
}

impl Drop for FenceContext {
	fn drop(&mut self) {
		let vk = self.vulkan.get();

		unsafe {
			(vk.destroy_fence)(vk.device, self.fence, null());
		}
	}
}
