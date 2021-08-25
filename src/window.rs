/*
 * Blue Engine copyright © Elham Aryanpur
 *
 * The license is same as the one on the root.
*/

use crate::definitions::{Engine, Object, Renderer, WindowDescriptor};
use winit::{
    event::{Event, WindowEvent, *},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use winit_input_helper::WinitInputHelper;

impl Engine {
    /// Creates a new window in current thread.
    #[allow(unreachable_code)]
    pub fn new(settings: WindowDescriptor) -> anyhow::Result<Self> {
        env_logger::init();
        // Dimentions of the window, as width and height
        // and then are set as a logical size that the window can accept
        let dimention = winit::dpi::LogicalSize {
            width: settings.width,   // Which sets the width of the window
            height: settings.height, // And sets the height of the window
        };

        // Here the size is finally made according to the dimentions we set earlier
        let size = winit::dpi::Size::Logical(dimention);

        // And we will create a new window and set all the options we stored
        let new_window = WindowBuilder::new()
            .with_inner_size(size) // sets the width and height of window
            .with_title(String::from(settings.title)) // sets title of the window
            .with_decorations(settings.decorations) // sets if the window should have borders
            .with_resizable(settings.resizable); // sets the window to be resizable

        // will create the main event loop of the window.
        // and will contain all the callbacks and button press
        // also will allow graphics API
        let event_loop = EventLoop::new();
        // bind the loop to window
        let window = new_window.build(&event_loop).unwrap();

        // The renderer init on current window
        let renderer = futures::executor::block_on(Renderer::new(&window));

        Ok(Self {
            window,
            event_loop,
            renderer,
            objects: Vec::new(),
        })
    }

    #[allow(unreachable_code)]
    pub fn update_loop<F>(self, mut update_function: F) -> anyhow::Result<()>
    where
        F: 'static + FnMut(&mut Renderer, &Window, &mut Vec<Object>, &WinitInputHelper),
    {
        let Self {
            event_loop,
            mut renderer,
            window,
            mut objects,
        } = self;

        // Run the callback of before renderer start
        //logic(&mut renderer, WindowCallbackEvents::Before, &window);
        // and get input events to handle them later
        let mut input = winit_input_helper::WinitInputHelper::new();
        // The main loop
        event_loop.run(move |event, _, control_flow| {
            input.update(&event);
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_renderer) => {
                            renderer.resize(*physical_renderer);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_renderer is &&mut so we have to dereference it twice
                            renderer.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }

                Event::MainEventsCleared => {
                    update_function(&mut renderer, &window, &mut objects, &input);
                    objects.iter_mut().for_each(|i| {
                        if i.changed {
                            i.update(&mut renderer, window.inner_size())
                                .expect("Couldn't update objects");
                        }
                    });

                    match renderer.render() {
                        Ok(_) => {}
                        // Recreate the swap_chain if lost
                        Err(wgpu::SwapChainError::Lost) => renderer.resize(renderer.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => (),
            }
        });
        //logic(&mut renderer, WindowCallbackEvents::After, &window);

        Ok(())
    }
}
