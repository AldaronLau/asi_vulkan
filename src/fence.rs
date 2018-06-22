// "asi_vulkan" - Aldaron's System Interface - Vulkan
//
// Copyright Jeron A. Lau 2018.
// Distributed under the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

use null;
use ami::Child;
use std::mem;

use Vulkan;
use Vk;
use VkObject;
use VkType;
use types::*;

pub struct Fence(pub(crate)Child<Vulkan, VkObject>);

impl Fence {
	pub fn new(connection: &mut Vk) -> Self {
		let fence = VkObject::new(VkType::Fence,
			unsafe { create_fence(connection) }, 0, 0);

		Fence(Child::new(&connection.0, fence))
	}

	pub fn fence(&self) -> u64 {
		self.0.data().fence()
	}
}

#[inline(always)] pub(crate) fn destroy(fence: u64, c: &mut Vulkan){
	// Run Drop Function
	unsafe {
		(c.destroy_fence)(c.device, fence, null());
	}
}

unsafe fn create_fence(connection: &mut Vk) -> VkFence {
	let connection = connection.0.data();

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
