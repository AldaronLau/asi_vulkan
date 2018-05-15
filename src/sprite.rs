// "asi_vulkan" crate - Licensed under the MIT LICENSE
//  * Copyright (c) 2017-2018  Jeron A. Lau <jeron.lau@plopgrizzly.com>

use null;
use mem;
use ami::Child;

use types::*;
use FogUniform;
use TransformUniform;
use Style;
use memory::{ Buffer, BufferBuilderType, Memory };
use Vk;
use Vulkan;
use VkObject;
use VkType;
use Image;

/// A render-able instance.
pub struct Sprite {
	// TODO: pub's?
	pub uniform_memory: Buffer,
	pub desc_set: Child<Vulkan, VkObject>,
	pub pipeline: VkPipeline,
	pub pipeline_layout: VkPipelineLayout,
}

impl Sprite {
	/// Create a new sprite.
	pub unsafe fn new<T>(vulkan: &mut Vk, pipeline: &Style,
		buffer_data: T,
		camera_memory: &Memory<TransformUniform>,
		effect_memory: Option<&Memory<FogUniform>>,
		texture: Option<&Image>, tex_count: bool)
		 -> Self where T: Clone
	{
	//	let connection = vulkan.0.data();

		let mut desc_pool = mem::uninitialized();
		let mut desc_set = mem::uninitialized();

		// Descriptor Pool
		(vulkan.0.data().new_descpool)(
			vulkan.0.data().device,
			// TODO: based on new_pipeline()
			&VkDescriptorPoolCreateInfo {
				s_type: VkStructureType::DescriptorPoolCreateInfo,
				next: null(),
				flags: 0,
				max_sets: 1,
				pool_size_count: if tex_count { 4 } else { 3 },
				pool_sizes: if tex_count {
					[VkDescriptorPoolSize { descriptor_type: 
						VkDescriptorType::UniformBuffer,
						descriptor_count: 1,
					},
					VkDescriptorPoolSize { descriptor_type: 
						VkDescriptorType::UniformBuffer,
						descriptor_count: 1,
					},
					VkDescriptorPoolSize { descriptor_type: 
						VkDescriptorType::UniformBuffer,
						descriptor_count: 1,
					},
					VkDescriptorPoolSize { descriptor_type: 
						VkDescriptorType::CombinedImageSampler,
						descriptor_count: 1,
					}].as_ptr()
				} else {
					[VkDescriptorPoolSize { descriptor_type: 
						VkDescriptorType::UniformBuffer,
						descriptor_count: 1,
					}, VkDescriptorPoolSize { descriptor_type: 
						VkDescriptorType::UniformBuffer,
						descriptor_count: 1,
					}, VkDescriptorPoolSize { descriptor_type: 
						VkDescriptorType::UniformBuffer,
						descriptor_count: 1,
					}].as_ptr()
				},
			},
			null(),
			&mut desc_pool
		).unwrap();

		(vulkan.0.data().new_descsets)(
			vulkan.0.data().device,
			&VkDescriptorSetAllocateInfo {
				s_type: VkStructureType::DescriptorSetAllocateInfo,
				next: null(),
				descriptor_pool: desc_pool,
				descriptor_set_count: 1,
				set_layouts: &pipeline.style().2/*descsetlayout*/
			},
			&mut desc_set
		).unwrap();

		// Allocate memory for uniform buffer.
		let uniform_memory = Buffer::new(vulkan, &[buffer_data],
			BufferBuilderType::Uniform);

		let device = vulkan.0.data().device;

		txuniform(vulkan, device, desc_set, tex_count, texture,
			&uniform_memory, camera_memory, effect_memory);

		Sprite {
			uniform_memory: uniform_memory,
			desc_set: Child::new(&vulkan.0, VkObject::new(
				VkType::Sprite, desc_set, desc_pool, 0)),
			pipeline: pipeline.style().0/*pipeline*/,
			pipeline_layout: pipeline.style().1/*pipeline_layout*/,
		}
	}

	pub/* TODO: (crate)*/ fn handles(&self) -> (u64, u64) {
		self.desc_set.data().image()
	}
}

unsafe fn txuniform(vulkan: &mut Vk, device: VkDevice,
	desc_set: VkDescriptorSet, hastex: bool, texture: Option<&Image>,
	matrix_memory: &Buffer,
	camera_memory: &Memory<TransformUniform>,
	effect_memory: Option<&Memory<FogUniform>>)
{
	let mut writer = DescriptorSetWriter::new()
		.uniform(desc_set, matrix_memory)
		.uniform(desc_set, &camera_memory.buffer);

	if let Some(memory) = effect_memory {
		writer = writer.uniform(desc_set, &memory.buffer);
	}

	if hastex {
		writer = writer.sampler(desc_set, vulkan.0.data().sampler,
			texture.unwrap().view());
	}

	writer.update_descriptor_sets(vulkan, device);
}

struct DescriptorSetWriter {
	sets: [Set; 255],
	nwrites: u8,
}

impl DescriptorSetWriter {
	/// Create a new DescriptorSetWriter.
	#[inline(always)]
	pub fn new() -> Self {
		Self {
			sets: unsafe { mem::uninitialized() },
			nwrites: 0,
		}
	}

	/// Write a uniform buffer to the descriptor set.
	#[inline(always)]
	pub fn uniform(mut self, desc_set: VkDescriptorSet, memory: &Buffer)
		-> Self
	{
		self.sets[self.nwrites as usize] = Set::Uniform(desc_set,
			memory.buffer());

		self.nwrites += 1;

		self
	}

	/// Write an image sampler to the descriptor set.
	#[inline(always)]
	pub fn sampler(mut self, desc_set: VkDescriptorSet,
		tex_sampler: VkSampler, tex_view: VkImageView) -> Self
	{
		self.sets[self.nwrites as usize] = Set::Sampler(desc_set, tex_sampler, tex_view);

		self.nwrites += 1;

		self
	}

	/// Update the descriptor sets.
	#[inline(always)]
	pub fn update_descriptor_sets(&self, connection: &mut Vk,
		device: VkDevice) -> ()
	{
		let connection = connection.0.data();

		let mut buffer_infos: [VkDescriptorBufferInfo; 255] = unsafe {
			mem::uninitialized()
		};
		let mut image_infos: [VkDescriptorImageInfo; 255] = unsafe {
			mem::uninitialized()
		};
		let mut writes: [VkWriteDescriptorSet; 255] = unsafe {
			mem::uninitialized()
		};

		for i in 0..self.nwrites {
			match self.sets[i as usize] {
				Set::Sampler(desc_set, tex_sampler, tex_view) => {
					image_infos[i as usize] = VkDescriptorImageInfo {
						sampler: tex_sampler,
						image_view: tex_view,
						image_layout: VkImageLayout::General,
					};
					writes[i as usize] = VkWriteDescriptorSet {
						s_type: VkStructureType::WriteDescriptorSet,
						next: null(),
						dst_set: desc_set,
						dst_binding: i as u32,
						descriptor_count: 1, //tex_count,
						descriptor_type: VkDescriptorType::CombinedImageSampler,
						image_info: &image_infos[i as usize],
						buffer_info: null(),
						dst_array_element: 0,
						texel_buffer_view: null(),
					};
				}
				Set::Uniform(desc_set, buffer) => {				
					buffer_infos[i as usize] = VkDescriptorBufferInfo {
						buffer: buffer,
						offset: 0,
						range: !0,
					};
					writes[i as usize] = VkWriteDescriptorSet {
						s_type: VkStructureType::WriteDescriptorSet,
						next: null(),
						dst_set: desc_set,
						dst_binding: i as u32,
						descriptor_count: 1,
						descriptor_type: VkDescriptorType::UniformBuffer,
						buffer_info: &buffer_infos[i as usize],
						dst_array_element: 0,
						texel_buffer_view: null(),
						image_info: null(),
					};
				}
			}
		}

		unsafe {
			(connection.update_descsets)(
				device,
				self.nwrites as u32,
				writes.as_ptr(),
				0,
				null(),
			);
		}
	}
}

enum Set {
	Uniform(VkDescriptorSet, VkBuffer),
	Sampler(VkDescriptorSet, VkSampler, VkImageView),
}

#[inline(always)] pub(crate) fn destroy(desc: (u64, u64), c: &mut Vulkan) {
	// Run Drop Function
	unsafe {
		(c.drop_descpool)(c.device, desc.1, null());
	}
}
