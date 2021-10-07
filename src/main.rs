extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod mesh;
mod scene_graph;
use scene_graph::SceneNode;
mod toolbox;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}




// == // Modify and complete the function below for the first task
unsafe fn setup_vao(vek: &Vec<f32>, ind: &Vec<u32>, col: &Vec<f32>, n_vec: &Vec<f32>) -> u32 {

    /* Sets up a Vertex Array Object containing triangles
       Returns the integer ID of the created VAO

       #Args
       * 'vek'   - Vector of 3D vertex coordinates
       * 'ind'   - Vector of indices
       * 'col'   - Vector of rgba values
       * 'n_vec' - Vector of normal vectors
    */ 

    let mut vao: gl::types::GLuint=0;
    let mut vbo: gl::types::GLuint=0;
    let mut ibuffer: gl::types::GLuint=0;
    let mut color_vbo: gl::types::GLuint=0;
    let mut n_vec_vbo: gl::types::GLuint=0;
    //let mut uni: gl::types::GLuint=0; //used for task 3
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);
    gl::GenBuffers(1,&mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER,vbo);
    gl::BufferData(gl::ARRAY_BUFFER,byte_size_of_array(vek),pointer_to_array(vek),gl::STATIC_DRAW);
    gl::VertexAttribPointer(0,3,gl::FLOAT,gl::FALSE,0,ptr::null());
    gl::EnableVertexAttribArray(0);
    gl::GenBuffers(1,&mut ibuffer);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,ibuffer);
    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,byte_size_of_array(ind),pointer_to_array(ind),gl::STATIC_DRAW);

    //color
    gl::GenBuffers(1, &mut color_vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, color_vbo);
    //gl::BufferData(gl::ARRAY_BUFFER, byte_size_of_array(col), pointer_to_array(col), gl::STATIC_DRAW);
    gl::BufferData(gl::ARRAY_BUFFER, (mem::size_of::<f32>()*col.len()) as isize, pointer_to_array(col), gl::STATIC_DRAW);
    gl::VertexAttribPointer(1,4,gl::FLOAT,gl::FALSE,16,ptr::null());
    gl::EnableVertexAttribArray(1);

    //normal vectors
    gl::GenBuffers(1, &mut n_vec_vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, n_vec_vbo);
    gl::BufferData(gl::ARRAY_BUFFER, (mem::size_of::<f32>()*n_vec.len()) as isize, pointer_to_array(n_vec), gl::STATIC_DRAW);
    gl::VertexAttribPointer(5,3,gl::FLOAT,gl::FALSE,0,ptr::null());
    gl::EnableVertexAttribArray(5);

    return vao;
}

//function to traverse and draw the scenegraph
unsafe fn draw_scene(node: &scene_graph::SceneNode,
    view_projection_matrix: &glm::Mat4) {
    // Check if node is drawable, set uniforms, draw
    //avoid drawing the rootnode
    if node.vao_id != 0 {
        let mvp=view_projection_matrix*node.current_transformation_matrix;
        gl::BindVertexArray(node.vao_id);
        gl::UniformMatrix4fv(4, 1, gl::FALSE, mvp.as_ptr());
        gl::UniformMatrix4fv(2, 1, gl::FALSE, node.current_transformation_matrix.as_ptr());
        gl::DrawElements(gl::TRIANGLES,node.index_count,gl::UNSIGNED_INT,ptr::null());
    }
    // Recurse
    for &child in &node.children {
        draw_scene(&*child, &view_projection_matrix);
    }
}

//function to traverse and update the scenegraph
unsafe fn update_node_transformations(node: &mut scene_graph::SceneNode,
    transformation_so_far: &glm::Mat4) {
    // Construct the correct transformation matrix
    let nref = -node.reference_point;
    let mut trans: glm::Mat4 = glm::identity();
    trans = glm::translation(&nref)*trans;
    trans = glm::scaling(&node.scale)*trans;
    trans = glm::rotation(node.rotation[0], &glm::vec3(1.0, 0.0, 0.0))*trans;
    trans = glm::rotation(node.rotation[1], &glm::vec3(0.0, 1.0, 0.0))*trans;
    trans = glm::rotation(node.rotation[2], &glm::vec3(0.0, 0.0, 1.0))*trans;
    trans = glm::translation(&node.reference_point)*trans;
    trans = glm::translation(&node.position)*trans;
    trans = transformation_so_far*trans;

    // Update the node's transformation matrix
    node.current_transformation_matrix=trans;
    // Recurse
    for &child in &node.children {
    update_node_transformations(&mut *child,
    &node.current_transformation_matrix);
    }
    }
    

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // Basic usage of shader helper:
        // The example code below returns a shader object, which contains the field `.program_id`.
        // The snippet is not enough to do the assignment, and will need to be modified (outside of
        // just using the correct path), but it only needs to be called once
        //
        unsafe {
                //attach and activate the shaders
                let shader = shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link();
                shader.activate();


        }

        //load the terrain model
        let mesh = mesh::Terrain::load("./resources/lunarsurface.obj");

        //set up new vao for the terrain model
        let terrain_vao= unsafe {
            let vek:     Vec<f32> = mesh.vertices;
            let indices: Vec<u32> = mesh.indices;
            let color:   Vec<f32> = mesh.colors;
            let normals: Vec<f32> = mesh.normals;
            setup_vao(&vek, &indices, &color, &normals)
        };


        //load the helicopter model
        let heli= mesh::Helicopter::load("./resources/helicopter.obj");
        
        //setup the vaos for the different parts of the heli
        let body_vao = unsafe {
            let vek:     Vec<f32> = heli.body.vertices;
            let indices: Vec<u32> = heli.body.indices;
            let color:   Vec<f32> = heli.body.colors;
            let normals: Vec<f32> = heli.body.normals;
            setup_vao(&vek, &indices, &color, &normals)
        };
        let door_vao = unsafe {
            let vek:     Vec<f32> = heli.door.vertices;
            let indices: Vec<u32> = heli.door.indices;
            let color:   Vec<f32> = heli.door.colors;
            let normals: Vec<f32> = heli.door.normals;
            setup_vao(&vek, &indices, &color, &normals)
        };
        let main_rotor_vao = unsafe {
            let vek:     Vec<f32> = heli.main_rotor.vertices;
            let indices: Vec<u32> = heli.main_rotor.indices;
            let color:   Vec<f32> = heli.main_rotor.colors;
            let normals: Vec<f32> = heli.main_rotor.normals;
            setup_vao(&vek, &indices, &color, &normals)
        };
        let tail_rotor_vao = unsafe {
            let vek:     Vec<f32> = heli.tail_rotor.vertices;
            let indices: Vec<u32> = heli.tail_rotor.indices;
            let color:   Vec<f32> = heli.tail_rotor.colors;
            let normals: Vec<f32> = heli.tail_rotor.normals;
            setup_vao(&vek, &indices, &color, &normals)
        };

        //setup the scene graph

        //generate nodes for all objects
        let mut root = SceneNode::new();
        let mut terrain = SceneNode::from_vao(terrain_vao, mesh.index_count);
        let mut body = SceneNode::from_vao(body_vao, heli.body.index_count);
        let mut door = SceneNode::from_vao(door_vao, heli.door.index_count);
        let mut main_rotor = SceneNode::from_vao(main_rotor_vao, heli.main_rotor.index_count);
        let mut tail_rotor = SceneNode::from_vao(tail_rotor_vao, heli.tail_rotor.index_count);

        //set the reference points for the nodes
        tail_rotor.reference_point = glm::vec3(0.35, 2.3, 10.4);
        main_rotor.reference_point = glm::vec3(0.0, 2.3, 0.0);
        body.reference_point       = glm::vec3(0.0, 0.0, 0.0);

        //organize the graph
        root.add_child(&terrain);
        terrain.add_child(&body);
        body.add_child(&door);
        body.add_child(&main_rotor);
        body.add_child(&tail_rotor);



        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        //transformation matrix from keyboard input
        let mut trans: glm::Mat4=glm::identity();

        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;
            let mut matrise: glm::Mat4=glm::identity(); //the final transformation matrix
            

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::W => {
                            trans = glm::translation(&glm::vec3(0.0, -40.0*delta_time, 0.0))*trans
                        },
                        VirtualKeyCode::S => {
                            trans = glm::translation(&glm::vec3(0.0, 40.0*delta_time, 0.0))*trans
                        },
                        VirtualKeyCode::A => {
                            trans = glm::translation(&glm::vec3(40.0*delta_time, 0.0, 0.0))*trans
                        },
                        VirtualKeyCode::D => {
                            trans = glm::translation(&glm::vec3(-40.0*delta_time, 0.0, 0.0))*trans
                        },
                        VirtualKeyCode::Q => {
                            trans = glm::translation(&glm::vec3(0.0, 0.0, 40.0*delta_time))*trans
                        },
                        VirtualKeyCode::E => {
                            trans = glm::translation(&glm::vec3(0.0, 0.0, -40.0*delta_time))*trans
                        },
                        VirtualKeyCode::Up => {
                            trans = glm::rotation(0.03,&glm::vec3(-10.0, 0.0, 0.0))*trans
                        },
                        VirtualKeyCode::Down => {
                            trans = glm::rotation(0.03,&glm::vec3(10.0, 0.0, 0.0))*trans
                        },
                        VirtualKeyCode::Left => {
                            trans = glm::rotation(0.03,&glm::vec3(0.0, -10.0, 0.0))*trans
                        },
                        VirtualKeyCode::Right => {
                            trans = glm::rotation(0.03,&glm::vec3(0.0, 10.0, 0.0))*trans
                        },
                        VirtualKeyCode::R => {
                            trans = glm::identity();
                        },


                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {



                *delta = (0.0, 0.0);
            }

            unsafe {
                gl::ClearColor(0.4, 0.71372549, 0.94901961, 1.0); // moon raker, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                //gl::Uniform1f(3,elapsed.sin()/2.0);
                //perspective matrix to achieve depth
                let persp: glm::Mat4 =glm::perspective(16.0/9.0 as f32, std::f32::consts::PI/2.0, 1.0 as f32, 1000.0 as f32);
                //translate to make sure triangles don't go out of view
                let z_translation: glm::Mat4 = glm::translation(&glm::vec3(-10.0, 0.0, -10.0));
                //apply transformations
                matrise=persp*trans*z_translation;

                //transformations so far, to be updated by the traversal-function
                let sofar: glm::Mat4 = glm::identity();

                
                //setup the animations
                tail_rotor.rotation.x = elapsed*5.0;
                main_rotor.rotation.y = elapsed*5.0;

                //get the Heading
                let heading = toolbox::simple_heading_animation(elapsed);

                //setup for animation
                body.position.x = heading.x;
                body.position.z = heading.z;
                body.rotation.z = heading.roll;
                body.rotation.y = heading.yaw;
                body.rotation.x = heading.pitch;
                

                //update and draw the scene graph
                update_node_transformations(&mut root, &sofar);
                draw_scene(&root, &matrise);



                // Issue the necessary commands to draw your scene here
                //Bind the VAO and make the drawcall
                /*
                gl::BindVertexArray(terrain_vao);
                gl::DrawElements(gl::TRIANGLES,mesh.index_count,gl::UNSIGNED_INT,ptr::null());

                //draw the helicopter
                gl::BindVertexArray(body_vao);
                gl::DrawElements(gl::TRIANGLES,heli.body.index_count,gl::UNSIGNED_INT,ptr::null());

                gl::BindVertexArray(door_vao);
                gl::DrawElements(gl::TRIANGLES,heli.door.index_count,gl::UNSIGNED_INT,ptr::null());

                gl::BindVertexArray(main_rotor_vao);
                gl::DrawElements(gl::TRIANGLES,heli.main_rotor.index_count,gl::UNSIGNED_INT,ptr::null());

                gl::BindVertexArray(tail_rotor_vao);
                gl::DrawElements(gl::TRIANGLES,heli.tail_rotor.index_count,gl::UNSIGNED_INT,ptr::null());
                */
                





            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    },
                    /* 
                    Q => {
                        *control_flow = ControlFlow::Exit;
                    }
                    */ 
                    _ => { }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            },
            _ => { }
        }
    });
}
