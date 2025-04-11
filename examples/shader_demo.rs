use std::fs;
use log::LevelFilter;
use crate::boiler::App;

pub mod boiler {
    use log::{info, trace};
    use sdl3_sys::everything::*;
    use std::ffi::{CStr, CString, c_char, c_void};
    use std::mem::{transmute, zeroed, MaybeUninit};
	use std::ptr;
	use std::ptr::{null_mut};

    #[allow(unused)]
    pub(crate) unsafe trait SdlProperty {
        /// INVARIANT: the underlying SDL API functions must not modify the data passed to them
        unsafe fn set_sdl_prop(&self, props: SDL_PropertiesID, key: *const c_char) -> bool;
    }

    /// SAFETY: This function must only be called after SDL is initialized by the engine,
    /// and the event subsystem must have been initialized (implicit with initializing video).
    #[inline(always)]
    pub unsafe fn poll_event() -> Option<SDL_Event> {
        unsafe {
            let mut ev: SDL_Event = MaybeUninit::uninit().assume_init();
            if SDL_PollEvent(&mut ev) {
                Some(ev)
            } else {
                None
            }
        }
    }

    /// Gets the latest SDL error as a Rust string.
    /// The returned value is a borrowed string because of lifetime reasons.
    pub fn get_sdl_error() -> Option<String> {
        // SDL_GetError is internally thread safe, so this function is as well.
        let err = unsafe { SDL_GetError() };
        if err.is_null() {
            None
        } else {
            // TODO: I don't think SDL would ever return invalid UTF-8,
            // 		but this may not be the best way of handling if it does.
            let string = unsafe { CStr::from_ptr(err as *mut c_char) }
                .to_str()
                .unwrap();
            // to owned
            Some(String::from(string))
        }
    }

    /// Panics and prints the contents of SDL_GetError to the console.
    pub fn panic_sdl_error(message: &str) -> ! {
        let err = get_sdl_error();
        let err_message = if let Some(err) = &err {
            &*err
        } else {
            "No further information (missing SDL error)."
        };
        panic!("{} {}", message, err_message);
    }

    #[inline]
    #[allow(private_bounds)]
    /// SAFETY:
    /// - The "key" field must be a null-terminated UTF-8 byte slice, contained in the SDL headers.
    /// - The value, if a CStr/CString, must be null-terminated.
    /// - The value, if a pointer, must either be null or point to a valid memory address (not previously freed memory).
    pub(crate) unsafe fn set_sdl_prop(
        props: SDL_PropertiesID,
        key: *const c_char,
        val: impl SdlProperty,
    ) -> bool {
        unsafe { val.set_sdl_prop(props, key) }
    }

    unsafe impl SdlProperty for *mut c_void {
        unsafe fn set_sdl_prop(&self, props: SDL_PropertiesID, key: *const c_char) -> bool {
            unsafe { SDL_SetPointerProperty(props, key as *const _, *self) }
        }
    }

    unsafe impl SdlProperty for &str {
        unsafe fn set_sdl_prop(&self, props: SDL_PropertiesID, key: *const c_char) -> bool {
            let c_safe = (*self).to_owned() + "\0";
            unsafe { SDL_SetStringProperty(props, key as *const _, c_safe.as_ptr() as *const _) }
        }
    }

    unsafe impl SdlProperty for String {
        unsafe fn set_sdl_prop(&self, props: SDL_PropertiesID, key: *const c_char) -> bool {
            let mut c_safe = self.clone();
            c_safe.push('\0');
            unsafe { SDL_SetStringProperty(props, key as *const _, c_safe.as_ptr() as *const _) }
        }
    }

    unsafe impl SdlProperty for &CStr {
        unsafe fn set_sdl_prop(&self, props: SDL_PropertiesID, key: *const c_char) -> bool {
            unsafe { SDL_SetStringProperty(props, key as *const _, self.as_ptr()) }
        }
    }

    unsafe impl SdlProperty for CString {
        #[inline]
        unsafe fn set_sdl_prop(&self, props: SDL_PropertiesID, key: *const c_char) -> bool {
            unsafe { SdlProperty::set_sdl_prop(&self.as_c_str(), props, key) }
        }
    }

    unsafe impl SdlProperty for f32 {
        unsafe fn set_sdl_prop(&self, props: SDL_PropertiesID, key: *const c_char) -> bool {
            unsafe { SDL_SetFloatProperty(props, key as *const _, *self) }
        }
    }

    unsafe impl SdlProperty for i64 {
        unsafe fn set_sdl_prop(&self, props: SDL_PropertiesID, key: *const c_char) -> bool {
            unsafe { SDL_SetNumberProperty(props, key as *const _, *self) }
        }
    }

    unsafe impl SdlProperty for bool {
        unsafe fn set_sdl_prop(&self, props: SDL_PropertiesID, key: *const c_char) -> bool {
            unsafe { SDL_SetBooleanProperty(props, key as *const _, *self) }
        }
    }

	unsafe fn wait_for_command_buffer(gpu_device: *mut SDL_GPUDevice, command_buffer: *mut SDL_GPUCommandBuffer) {
		unsafe {
			let fence = SDL_SubmitGPUCommandBufferAndAcquireFence(command_buffer);
			if fence.is_null() {
				panic_sdl_error("GPU fence was null!");
			}
			let fences = [fence];
			// For now, blocking is fine. It makes my fans spin up less :)
			SDL_WaitForGPUFences(gpu_device, true, fences.as_ptr(), fences.len() as u32);
			SDL_ReleaseGPUFence(gpu_device, fence);
		}
	}

	pub struct App {
		is_running: bool,
		window: *mut SDL_Window,
		gpu_device: *mut SDL_GPUDevice,
		quad_vertex_buffer: *mut SDL_GPUBuffer,
		quad_index_buffer: *mut SDL_GPUBuffer,
		vertex_shader: *mut SDL_GPUShader,
		fragment_shader: *mut SDL_GPUShader,
		width: i32,
		height: i32,
		pipeline: *mut SDL_GPUGraphicsPipeline,
	}

    impl App {
        pub fn new(vertex_shader: &[u8], fragment_shader: &[u8]) -> Self {
            let app_name = c"Yuri Demo";
            unsafe {
                // region: Initialize SDL
                // called before init
                SDL_SetMainReady();
                SDL_SetLogPriorities(SDL_LogPriority::TRACE);
                SDL_SetAppMetadata(
                    app_name.as_ptr(),
                    c"1.0.0".as_ptr(),
                    c"sh.addie.yuri.demo".as_ptr(),
                );

                info!("Initializing SDL");
                if !SDL_Init(SDL_INIT_VIDEO) {
                    panic_sdl_error("Failed to initialize SDL!");
                }
                // endregion

				info!("Done initializing SDL");

                // region: Create SDL window
                let window_props = SDL_CreateProperties();

                // the render width
                let width: i32 = 640;
                // the render height
                let height: i32 = 480;

                set_sdl_prop(window_props, SDL_PROP_WINDOW_CREATE_WIDTH_NUMBER, width as i64);
                set_sdl_prop(window_props, SDL_PROP_WINDOW_CREATE_HEIGHT_NUMBER, height as i64);
                set_sdl_prop(window_props, SDL_PROP_WINDOW_CREATE_TITLE_STRING, app_name);
                set_sdl_prop(window_props, SDL_PROP_WINDOW_CREATE_RESIZABLE_BOOLEAN, true);
                set_sdl_prop(window_props, SDL_PROP_WINDOW_CREATE_FOCUSABLE_BOOLEAN, true);
                set_sdl_prop(window_props, SDL_PROP_WINDOW_CREATE_HIDDEN_BOOLEAN, false);
				// 	SDL_HINT_VIDEO_DRIVER,
				// 	SdlProperty::String("wayland\0")
				// 	SDL_HINT_VIDEO_WAYLAND_ALLOW_LIBDECOR,
				// 	SdlProperty::String("0\0")

                let window = SDL_CreateWindowWithProperties(window_props);
                if window.is_null() {
                    panic_sdl_error("Failed to create window!");
                }
                // endregion

				info!("Done creating window");

				// region: Initialize GPU subsystem
                let gpu_device = SDL_CreateGPUDevice(
                    // we only support SPIR-V currently
                    SDL_GPU_SHADERFORMAT_SPIRV,
                    cfg!(debug_assertions),
                    null_mut(),
                );
                if gpu_device.is_null() {
                    panic_sdl_error("Failed to create SDL_GPU device!");
                }
                if !SDL_ClaimWindowForGPUDevice(gpu_device, window) {
                    panic_sdl_error("Failed to claim window for GPU!");
                }

				// upload buffer to gpu
				let quad_mesh_vertices: [f32; 16] = [
					// pos, 	uv
					-1., -1., 	0.0, 0.0,
					1.0, -1., 	1.0, 0.0,
					1.0, 1.1, 	1.0, 1.0,
					-1., 1.0, 	0.0, 1.0,
				];
				let quad_mesh_indices: [u16; 6] = [
					0, 1, 2,
					2, 3, 0,
				];
				
				let command_buffer = SDL_AcquireGPUCommandBuffer(gpu_device);

				if command_buffer.is_null() {
					panic_sdl_error("Failed to acquire GPU command buffer");
				}
				let copy_pass: *mut SDL_GPUCopyPass = SDL_BeginGPUCopyPass(command_buffer);

				let make_buffer = |size: usize, usage: SDL_GPUBufferUsageFlags| -> *mut SDL_GPUBuffer {
					let create_info = SDL_GPUBufferCreateInfo {
						usage,
						size: size as u32,
						props: 0,
					};
					let buffer = SDL_CreateGPUBuffer(
						gpu_device,
						&create_info
					);

					if buffer.is_null() {
						panic_sdl_error("Failed to create GPU transfer buffer!");
					}

					buffer
				};

				let vertices_size = size_of_val(&quad_mesh_vertices);
				trace!("vertex buffer size (bytes): {vertices_size}");
				let indices_size = size_of_val(&quad_mesh_indices);
				trace!("index buffer size (bytes): {indices_size}");

				let quad_mesh_vertex_buffer = make_buffer(
					vertices_size,
					SDL_GPU_BUFFERUSAGE_VERTEX
				);

				let quad_mesh_index_buffer = make_buffer(
					indices_size,
					SDL_GPU_BUFFERUSAGE_INDEX
				);

				// raw implementation looks like this:
				// - determine size of buffer
				// - create transfer buffer
				// - map it to a pointer
				// - write data into buffer
				// - unmap pointer
				// - begin copy pass
				// - upload each
				// - end copy pass

				// cycling is weird, see:
				// https://github.com/libsdl-org/SDL/blob/fbdb6379781f2874860fe3079f452ec1efbca1ac/include/SDL3/SDL_gpu.h#L260

				let transfer_buffer = {
					let transfer_size = vertices_size + indices_size;
					trace!("transfer buffer size (bytes): {transfer_size}");
					let create_info = SDL_GPUTransferBufferCreateInfo {
						// for now, this is hardcoded.
						usage: SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD,
						size: transfer_size as u32,
						props: 0,
					};
					SDL_CreateGPUTransferBuffer(
						gpu_device,
						&create_info,
					)
				};
				if transfer_buffer.is_null() {
					panic_sdl_error("Failed to create GPU transfer buffer!");
				}

				// memcopy scary
				let scary_big_evil_buffer = SDL_MapGPUTransferBuffer(
					gpu_device,
					transfer_buffer,
					false
				);

				ptr::copy_nonoverlapping(
					quad_mesh_vertices.as_ptr() as *const u8,
					scary_big_evil_buffer as _,
					vertices_size
				);
				ptr::copy_nonoverlapping(
					quad_mesh_indices.as_ptr() as *const u8,
					scary_big_evil_buffer.byte_add(vertices_size) as *mut u8,
					indices_size
				);

				{
					let src = SDL_GPUTransferBufferLocation {
						transfer_buffer,
						offset: 0,
					};
					let dst = SDL_GPUBufferRegion {
						buffer: quad_mesh_vertex_buffer,
						offset: 0,
						size: vertices_size as u32,
					};

					SDL_UploadToGPUBuffer(
						copy_pass,
						&src,
						&dst,
						false,
					);
				}

				{
					let src = SDL_GPUTransferBufferLocation {
						transfer_buffer,
						offset: vertices_size as u32,
					};
					let dst = SDL_GPUBufferRegion {
						buffer: quad_mesh_index_buffer,
						offset: 0,
						size: indices_size as u32,
					};

					SDL_UploadToGPUBuffer(
						copy_pass,
						&src,
						&dst,
						false,
					);
				}

				SDL_EndGPUCopyPass(copy_pass);

				SDL_UnmapGPUTransferBuffer(gpu_device, transfer_buffer);
				SDL_ReleaseGPUTransferBuffer(gpu_device, transfer_buffer);

				wait_for_command_buffer(gpu_device, command_buffer);

				let vertex_shader = Self::compile_shader(
					gpu_device,
					SDL_GPUShaderStage::VERTEX,
					vertex_shader
				);

				let fragment_shader = Self::compile_shader(
					gpu_device,
					SDL_GPUShaderStage::FRAGMENT,
					fragment_shader
				);

				// endregion

                let mut this = Self {
                    is_running: true,
                    window,
                    gpu_device,
					quad_vertex_buffer: quad_mesh_vertex_buffer,
					quad_index_buffer: quad_mesh_index_buffer,
					vertex_shader,
					fragment_shader,
					width,
                    height,

					// we initialize the pipeline once we have everything else, for convenience.
					// the rebuild_pipeline function is mostly called post-initialization,
					// so I just added a check of "if we haven't created the pipeline yet,
					// don't try and delete the old one."
					pipeline: null_mut(),
				};
				this.rebuild_pipeline();
				this
            }
        }

        pub fn ready(&self) -> bool {
            self.is_running
        }

		unsafe fn render(&mut self) {
			unsafe {
				let command_buffer = SDL_AcquireGPUCommandBuffer(self.gpu_device);

				if command_buffer.is_null() {
					panic_sdl_error("Failed to acquire GPU command buffer");
				}

				let mut swapchain_texture = zeroed();
				if !SDL_WaitAndAcquireGPUSwapchainTexture(
					command_buffer,
					self.window,
					&mut swapchain_texture,
					null_mut(),
					null_mut(),
				) {
					panic_sdl_error("Failed to acquire GPU swapchain texture!");
				}

				if swapchain_texture.is_null() {
					// Swapchain is unavailable, cancel work
					SDL_CancelGPUCommandBuffer(command_buffer);
				} else {
					let render_pass;
					let mut color_target_info: SDL_GPUColorTargetInfo = zeroed();
					color_target_info.texture = swapchain_texture;
					color_target_info.clear_color = SDL_FColor {
						r: 0.0,
						g: 0.0,
						b: 0.0,
						a: 1.0,
					};
					color_target_info.load_op = SDL_GPULoadOp::CLEAR;
					color_target_info.store_op = SDL_GPUStoreOp::STORE;

					render_pass = SDL_BeginGPURenderPass(
						command_buffer,
						&color_target_info,
						1,
						null_mut()
					);
					{
						let viewport = SDL_GPUViewport {
							x: 0.0,
							y: 0.0,
							// w: 128.0,
							// h: 128.0,
							w: self.width as f32,
							h: self.height as f32,
							min_depth: -1.0,
							max_depth: 1.0,
						};
						SDL_SetGPUViewport(render_pass, &viewport);
						SDL_BindGPUGraphicsPipeline(render_pass, self.pipeline);
						let vertex_binding = SDL_GPUBufferBinding {
							buffer: self.quad_vertex_buffer,
							offset: 0
						};
						SDL_BindGPUVertexBuffers(
							render_pass,
							0,
							&vertex_binding,
							1
						);
						let index_binding = SDL_GPUBufferBinding {
							buffer: self.quad_index_buffer,
							offset: 0
						};
						SDL_BindGPUIndexBuffer(
							render_pass,
							&index_binding,
							SDL_GPU_INDEXELEMENTSIZE_16BIT
						);
						SDL_DrawGPUIndexedPrimitives(render_pass, 6, 1, 0, 0, 0);
					}
					SDL_EndGPURenderPass(render_pass);

					wait_for_command_buffer(self.gpu_device, command_buffer);
				}
			}
		}

        pub fn update(mut self) -> Option<Self> {
            if !self.is_running {
                return None;
            }

            unsafe {
                while let Some(ev) = poll_event() {
                    match transmute(ev.r#type) {
						SDL_EVENT_QUIT => {
							self.is_running = false;
						}
                        _ => {}
                    }
                }
            }

            // update
			unsafe {
				self.render();
			}

            Some(self)
        }

		unsafe fn rebuild_pipeline(&mut self) {
			unsafe {
				// Create the pipelines
				let color_target_descriptions = [SDL_GPUColorTargetDescription {
					format: SDL_GetGPUSwapchainTextureFormat(self.gpu_device, self.window),
					.. Default::default()
				}];
				let vertex_buffer_descriptions = [SDL_GPUVertexBufferDescription {
					slot: 0,
					input_rate: SDL_GPU_VERTEXINPUTRATE_VERTEX,
					instance_step_rate: 0,
					// x, y, u, v
					pitch: (size_of::<f32>() * (2 + 2)) as u32,
				}];
				let vertex_attributes = [
					SDL_GPUVertexAttribute {
						buffer_slot: 0,
						format: SDL_GPU_VERTEXELEMENTFORMAT_FLOAT2,
						location: 0,
						offset: 0,
					},
					SDL_GPUVertexAttribute {
						buffer_slot: 0,
						format: SDL_GPU_VERTEXELEMENTFORMAT_FLOAT2,
						location: 1,
						offset: (size_of::<f32>() * 2) as u32,
					}
				];
				let create_info = SDL_GPUGraphicsPipelineCreateInfo {
					target_info: SDL_GPUGraphicsPipelineTargetInfo {
						num_color_targets: color_target_descriptions.len() as u32,
						color_target_descriptions: color_target_descriptions.as_ptr(),
						.. Default::default()
					},
					vertex_input_state: SDL_GPUVertexInputState {
						num_vertex_buffers: vertex_buffer_descriptions.len() as u32,
						vertex_buffer_descriptions: vertex_buffer_descriptions.as_ptr(),
						num_vertex_attributes: vertex_attributes.len() as u32,
						vertex_attributes: vertex_attributes.as_ptr(),
					},
					primitive_type: SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
					vertex_shader: self.vertex_shader,
					fragment_shader: self.fragment_shader,
					.. Default::default()
				};
				let pipeline = SDL_CreateGPUGraphicsPipeline(
					self.gpu_device,
					&create_info
				);
				if pipeline.is_null() {
					panic_sdl_error("Failed to create fill pipeline!");
				}
				// free the old pipeline
				if !self.pipeline.is_null() {
					SDL_ReleaseGPUGraphicsPipeline(self.gpu_device, self.pipeline);
				}
				self.pipeline = pipeline;
			}
		}

		unsafe fn compile_shader(gpu_device: *mut SDL_GPUDevice, stage: SDL_GPUShaderStage, new: &[u8]) -> *mut SDL_GPUShader {
			unsafe {
				let create_info = SDL_GPUShaderCreateInfo {
					code_size: new.len() * size_of::<u8>(),
					code: new.as_ptr() as _,
					entrypoint: c"main".as_ptr(),
					format: SDL_GPU_SHADERFORMAT_SPIRV,
					stage,
					num_samplers: 0,
					num_storage_textures: 0,
					num_storage_buffers: 0,
					num_uniform_buffers: 0,
					props: 0,
				};
				let shader = SDL_CreateGPUShader(gpu_device, &create_info);
				if shader.is_null() {
					panic_sdl_error("Failed to create GPU shader!");
				}
				shader
			}
		}

		pub fn swap_vertex_shader(&mut self, vertex_shader: &[u8]) {
			unsafe {
				// SDL_WaitForGPUIdle(self.gpu_device);
				let shader = Self::compile_shader(
					self.gpu_device,
					SDL_GPUShaderStage::VERTEX,
					vertex_shader
				);

				SDL_ReleaseGPUShader(self.gpu_device, self.vertex_shader);
				self.vertex_shader = shader;
				self.rebuild_pipeline();
			}
		}

		pub fn swap_fragment_shader(&mut self, fragment_shader: &[u8]) {
			unsafe {
				// SDL_WaitForGPUIdle(self.gpu_device);
				let shader = Self::compile_shader(
					self.gpu_device,
					SDL_GPUShaderStage::FRAGMENT,
					fragment_shader
				);
				SDL_ReleaseGPUShader(self.gpu_device, self.fragment_shader);
				self.fragment_shader = shader;
				self.rebuild_pipeline();
			}
		}
    }

    impl Drop for App {
        fn drop(&mut self) {
            unsafe {
                // release GPU resources
                if !self.gpu_device.is_null() {
					SDL_ReleaseGPUGraphicsPipeline(self.gpu_device, self.pipeline);
					SDL_ReleaseGPUShader(self.gpu_device, self.vertex_shader);
					SDL_ReleaseGPUShader(self.gpu_device, self.fragment_shader);

					SDL_ReleaseGPUBuffer(self.gpu_device, self.quad_index_buffer);
					SDL_ReleaseGPUBuffer(self.gpu_device, self.quad_vertex_buffer);

                    SDL_ReleaseWindowFromGPUDevice(self.gpu_device, self.window);
                    SDL_DestroyGPUDevice(self.gpu_device);
                }

                // close window
                if !self.window.is_null() {
                    SDL_DestroyWindow(self.window);
                }

                // de-init
                SDL_Quit();
            }
        }
    }
}

fn main() {
    colog::basic_builder().filter_level(LevelFilter::Trace).init();

	let frag_source = fs::read("yuri.frag.spv").unwrap();
	let vert_source = fs::read("minimal.vert.spv").unwrap();

    let mut app = App::new(&vert_source, &frag_source);

    loop {
        // move out of app
        let res = app.update();
        if let None = res {
            return;
        }

        // move into app
        app = res.unwrap();
    }
}
