pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at(self.eye, self.target, self.up);
        let proj =
            cgmath::perspective(cgmath::Deg(self.fovy),
            self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn new(aspect: f32) -> Self {
        Self {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: 190.0 / aspect,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

pub struct CameraController {
    speed: f32,
    sens: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,

    look_left_amt: f32,
    look_up_amt: f32,
}

use winit::event::*;

impl CameraController {
    pub fn new(speed: f32, sens: f32) -> Self {
        Self {
            speed,
            sens,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,

            look_left_amt: 0.0,
            look_up_amt: 0.0,
        }
    }

    pub fn process_events(&mut self, event: &Event<crate::UserEventType>) -> bool {
        match event {
            Event::DeviceEvent {event, ..} => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.look_left_amt -= delta.0 as f32;
                    self.look_up_amt -= delta.1 as f32;
                    true
                },
                _ => false,
            }
            Event::WindowEvent {event, ..} => match event{
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                    ..
                } => {
                    let is_pressed = *state == ElementState::Pressed;
                    match keycode {
                        VirtualKeyCode::Space => {
                            self.is_up_pressed = is_pressed;
                            true
                        }
                        VirtualKeyCode::LShift => {
                            self.is_down_pressed = is_pressed;
                            true
                        }
                        VirtualKeyCode::W => {
                            self.is_forward_pressed = is_pressed;
                            true
                        }
                        VirtualKeyCode::A => {
                            self.is_left_pressed = is_pressed;
                            true
                        }
                        VirtualKeyCode::S => {
                            self.is_backward_pressed = is_pressed;
                            true
                        }
                        VirtualKeyCode::D => {
                            self.is_right_pressed = is_pressed;
                            true
                        }
                        //VirtualKeyCode::Up => {
                            //self.is_look_up_pressed = is_pressed;
                            //true
                        //}
                        //VirtualKeyCode::Left => {
                            //self.is_look_left_pressed = is_pressed;
                            //true
                        //}
                        //VirtualKeyCode::Down => {
                            //self.is_look_down_pressed = is_pressed;
                            //true
                        //}
                        //VirtualKeyCode::Right => {
                            //self.is_look_right_pressed = is_pressed;
                            //true
                        //}
                        _ => false,
                    }
                }
                _ => false,
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: std::time::Duration) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward = forward.normalize();

        let speed = self.speed * (dt.as_nanos() as f32 / 1_000_000.0);

        if self.is_forward_pressed {
            camera.eye += forward * speed;
            camera.target += forward * speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward * speed;
            camera.target -= forward * speed;
        }

        let right = forward.cross(camera.up).normalize();

        if self.is_right_pressed {
            camera.eye += right * speed;
            camera.target += right * speed;
        }
        if self.is_left_pressed {
            camera.eye -= right * speed;
            camera.target -= right * speed;
        }

        let up = camera.up;

        if self.is_up_pressed {
            camera.eye += up * speed;
            camera.target += up * speed;
        }
        if self.is_down_pressed {
            camera.eye -= up * speed;
            camera.target -= up * speed;
        }

        let look_vector = (camera.target - camera.eye).normalize();

        let uprot: cgmath::Quaternion<f32> = cgmath::Rotation3::from_axis_angle(right, cgmath::Deg(self.look_up_amt * speed * self.sens));
        let lrrot: cgmath::Quaternion<f32> = cgmath::Rotation3::from_axis_angle(right.cross(look_vector).normalize(), cgmath::Deg(self.look_left_amt * speed * self.sens));

        self.look_left_amt = 0.0;
        self.look_up_amt = 0.0;

        camera.target = camera.eye + uprot * lrrot * look_vector;
    }
}
