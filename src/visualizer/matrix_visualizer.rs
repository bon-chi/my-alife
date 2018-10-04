use algorithm::gray_scott::MatrixTuple;
use failure;
use glium::{glutin, index, texture, Display, Program, Surface, VertexBuffer};
use ndarray::{ArrayBase, Dim, OwnedRepr};
use std::fs::File;
use std::thread;
use std::io;
use std::io::prelude::*;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use visualizer::WindowStatus;

/// 直交座標系(XY座標系)を用いてvisualizeする構造体
pub struct MatrixVisualizer {
    program: Program,
    events_loop: glutin::EventsLoop,
    vertex_buffer: VertexBuffer<Vertex>,
    indices: index::NoIndices,
    display: Display,
}

impl MatrixVisualizer {
    /// MatrixVisualizerインスタンスを生成する
    ///
    /// # Arguments
    /// * `title` - ウィンドウに表示するタイトル
    /// * `vertex_glsl_path` - バーテックスシェーダーのファイルを格納しているpath
    /// * `grafic_glsl_path` - グラフィックシェーダーのファイルを格納しているpath
    ///
    /// # Example
    /// ``````
    /// use my_alife::visualizer::matrix_visualizer::MatrixVisualizer;
    /// let matrix_visualize = MatrixVisualizer::new(
    ///   "Gray Scott",
    ///   "res/shaders/matrix_visualizer_vertex.glsl",
    ///   "res/shaders/matrix_visualizer_fragment.glsl",
    /// ).unwrap();
    ///
    ///
    pub fn new(
        title: &str,
        vertex_glsl_path: &str,
        faragment_glsl_path: &str,
    ) -> Result<MatrixVisualizer, failure::Error> {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_dimensions((600, 600).into())
            .with_title(title);
        let context = glutin::ContextBuilder::new();
        let display = Display::new(window, context, &events_loop).unwrap();
        let program = Program::from_source(
            &display,
            &Self::glsl(vertex_glsl_path)?,
            &Self::glsl(faragment_glsl_path)?,
            None,
        )?;

        let vertex_buffer = VertexBuffer::new(&display, &Self::shape()).unwrap();
        Ok(MatrixVisualizer {
            program: program,
            events_loop: events_loop,
            vertex_buffer: vertex_buffer,
            indices: index::NoIndices(index::PrimitiveType::TrianglesList),
            display: display,
        })
    }

    fn glsl(path: &str) -> Result<String, io::Error> {
        let mut contents = String::new();
        File::open(path)?.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn shape() -> Vec<Vertex> {
        let vertex1 = Vertex {
            a_position: [-1.0, -1.0],
            a_texcoord: [0.0, 1.0],
        };
        let vertex2 = Vertex {
            a_position: [1.0, -1.0],
            a_texcoord: [1.0, 1.0],
        };
        let vertex3 = Vertex {
            a_position: [1.0, 1.0],
            a_texcoord: [1.0, 0.0],
        };
        let vertex4 = Vertex {
            a_position: [-1.0, -1.0],
            a_texcoord: [0.0, 1.0],
        };
        let vertex5 = Vertex {
            a_position: [-1.0, 1.0],
            a_texcoord: [0.0, 0.0],
        };
        let vertex6 = Vertex {
            a_position: [1.0, 1.0],
            a_texcoord: [1.0, 0.0],
        };
        vec![vertex1, vertex2, vertex3, vertex4, vertex5, vertex6]
    }

    /// メインループ
    ///
    /// # Arguments
    /// * `initail_state` - 初期状態
    /// * `unpdate_fn` - 描画する状態をどのように変更するかの関数
    ///
    /// # Example
    pub fn draw_loop<T: 'static, F: 'static>(mut self, mut state: Arc<Mutex<T>>, f: f32, k: f32, update_fn: F) -> Result<(), failure::Error>
    where
        T: AsMut<Matrix<f32>> + Send,
        F: Fn(&mut T, f32, f32) + Send,
        // F: Fn(&mut Arc<Mutex<T>>, f32, f32),
        // F: Fn(&mut Arc<Mutex<T>>, f32, f32) + Send
    {
        let mut window_status = WindowStatus::Open;
        let mut cloned = state.clone();
        let _t = thread::spawn( move ||
          loop {
              update_fn(&mut cloned.lock().unwrap(), f, k);
          }
        );
        loop {
            if window_status == WindowStatus::Close {
                break;
            }
            self.draw(state.lock().unwrap().as_mut())?;

            window_status = self.hadling_event();
        }
        Ok(())
    }

    /// 実際に描画を行う
    ///
    /// # Arguments
    /// * `matrix` - 描画される内容
    ///
    pub fn draw(&self, matrix: &Matrix<f32>) -> Result<(), failure::Error> {
        let image = make_texture_image(matrix);
        let texture = texture::Texture2d::new(&self.display, image);
        let mut target = self.display.draw();
        target.clear_color(1.0, 0.0, 0.0, 1.0);
        target.draw(
            &self.vertex_buffer,
            &self.indices,
            &self.program,
            &uniform! {u_texture: texture?.sampled()},
            &Default::default(),
        )?;
        target.finish()?;
        Ok(())
    }

    /// event handler
    pub fn hadling_event(&mut self) -> WindowStatus {
        let mut status = WindowStatus::Open;
        self.events_loop.poll_events(|event| {
            // matchさせたいパターンが1つしかない場合、if let 形式で書ける
            // matchでやると
            // match event {
            //   glutin::Event::WindowEvent { event, .. } => { do_something() },
            //   _ => { // do nothing }}
            // }
            // みたいにcatch節的なものが必要になる(rustのパターンマッチは取り得る全パターンを明示的に書かせるため)
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::CloseRequested => status = WindowStatus::Close,
                    glutin::WindowEvent::KeyboardInput {
                        device_id: _,
                        input: keyboard_input,
                    } => match keyboard_input {
                        glutin::KeyboardInput { // 構造体の各fieldをdestructuringできる
                            virtual_keycode, // virtual_keycode: virtual_keycode を省略形
                            modifiers, // modifiers: my_modifiers の様に省略しないで別名をつけても良い
                            .. // 使わないfieldのscancode: _, state: _, を省略できる
                        } => match (virtual_keycode, modifiers) { // 複数のパターンマッチにはタプルを使う
                            #[cfg(target_os = "linux")] // conditional compile https://doc.rust-lang.org/reference/attributes.html#conditional-compilation
                            (Some(glutin::VirtualKeyCode::W), glutin::ModifiersState { ctrl, .. }) => {
                              if ctrl { status = WindowStatus::Close }
                            },
                            #[cfg(target_os = "macos")]
                            (Some(glutin::VirtualKeyCode::W), glutin::ModifiersState { logo, .. }) => {
                              if logo { status = WindowStatus::Close }
                            },
                            (_, _) => {}
                        },
                    },
                    _ => {}
                }
            };
        });
        return status;
    }
}

/// 直交座標系(XY座標系)においてどの座標にどんな色(グレースケール)を表示するかを表現する。  
/// 実体は2次元配列
pub type Matrix<T> = ArrayBase<OwnedRepr<T>, Dim<[usize; 2]>>;

#[derive(Copy, Clone)]
struct Vertex {
    a_position: [f32; 2],
    a_texcoord: [f32; 2],
}
implement_vertex!(Vertex, a_position, a_texcoord);

fn make_texture_image<'a>(u: &Matrix<f32>) -> texture::RawImage2d<'a, u8> {
    let mut texture_data = Vec::new();
    for row in u.outer_iter() {
        for e in row.iter() {
            let v = (if *e < 0.0 {
                0.0
            } else if *e > 1.0 {
                1.0
            } else {
                *e
            } * 255.0) as u8;

            texture_data.push(v);
            texture_data.push(v);
            texture_data.push(v);
            texture_data.push(v);
        }
    }
    texture::RawImage2d::from_raw_rgba(texture_data, (u.shape()[0] as u32, u.shape()[1] as u32))
}
