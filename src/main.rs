use crate::cpu::CPU;
use crate::cpuregisters::CPURegisters;
use crate::ram_controller::RamController;
use crate::cartridge::Cartridge;
use crate::ppu_registers::PPURegisters;
use crate::vram_controller::VRAMController;
use crate::ppu::{PPU, PPUResult};
use std::cell::{Cell, RefCell};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::texture::Texture;
use crate::renderer_gl::{Shader, Program};
use std::io::{self, Write};

mod cpu;
mod cpuregisters;
mod ram_controller;
mod opcodes;
mod cartridge;
mod ppu_registers;
mod vram_controller;
mod ppu;
mod stack;
mod window;
mod texture;
mod renderer_gl;

fn main()
{
    let sdl = sdl2::init().unwrap();
    let window = window::Window::create(&sdl).unwrap();

    use std::ffi::CString;
    let vert_shader = Shader::from_vert_source(&CString::new(include_str!("triangle.vert")).unwrap()).unwrap();
    let frag_shader = Shader::from_frag_source(&CString::new(include_str!("triangle.frag")).unwrap()).unwrap();

    let shader_program = Program::from_shaders(
        &[vert_shader, frag_shader]
    ).unwrap();

    let vertices: Vec<f32> = vec![
        -1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, // uppe vänster?
        1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, // uppe höger
        1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, // nere höger

        -1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, // uppe vänster?
        1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, // nere höger
        -1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // nere vänster
    ];

    let mut vbo: gl::types::GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl::VertexAttribPointer(
            0, // index of the generic vertex attribute ("layout (location = 0)")
            3, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            std::ptr::null(), // offset of the first component
        );

        gl::EnableVertexAttribArray(1); // this is "layout (location = 1)" in vertex shader
        gl::VertexAttribPointer(
            1, // index of the generic vertex attribute ("layout (location = 1)")
            3, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
        );

        gl::EnableVertexAttribArray(2); // this is "layout (location = 2)" in vertex shader
        gl::VertexAttribPointer(
            2, // index of the generic vertex attribute ("layout (location = 1)")
            2, // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            (6 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
        );

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    let mut pixels: [u8; 256 * 240 * 3] = [0; 256 * 240 * 3];


    let texture = Texture::from_pixels(256, 240, pixels.to_vec()).unwrap();
    texture.bind();

    let vram = RefCell::new(VRAMController::new());
    let ppu_regs = Cell::new(PPURegisters::new());
    let mut memory = RamController::new(&ppu_regs, &vram);
    // let c = Cartridge::load("./roms/nestest.nes");
    // let c = Cartridge::load("../../roms/nestest.nes");
    // let c = Cartridge::load("/Users/emil/code/rustnes/roms/Balloon Fight (E).nes");
    let c = Cartridge::load("/Users/emil/code/rustnes/roms/Donkey_Kong_JU.nes");
    // let c = Cartridge::load("/Users/emil/code/rustnes/roms/nestest.nes");
    // let c = Cartridge::load("/Users/emil/code/rustnes/roms/full_palette.nes");

    if c.prg_rom_banks().len() > 1 {
        memory.load_prg_bank1(&c.prg_rom_banks()[0]);
        memory.load_prg_bank1(&c.prg_rom_banks()[1]);
    } else {
        memory.load_prg_bank1(&c.prg_rom_banks()[0]);
        memory.load_prg_bank2(&c.prg_rom_banks()[0]);
    }

    if c.chr_rom_banks().len() > 1 {
        panic!("I do not support multiple chr rom banks!! :(");
    }

    vram.borrow_mut().load_chr_rom(&c.chr_rom_banks()[0]);



    let mut cpu = CPU::new(&mut memory);
    let mut ppu = PPU::new(&vram, &ppu_regs);
    cpu.reset();

    let mut total_cycles = 7;
    let mut should_break = false;

    let mut event_pump = sdl.event_pump().unwrap();

    let mut foo = false;

    let mut iteration = 0;


    // ppu.process(total_cycles * 3);

    'running: loop {
        iteration += 1;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(Keycode::LShift), .. } => {
                    foo = true;
                }
                _ => {}
            }
        }

        if total_cycles == 27399 {
            let ffff = 2323;
        }

        print!("{:04X}  ", cpu.registers.pc());
        io::stdout().flush().unwrap();
        let regs_copy = cpu.registers.clone();

        let cycles = cpu.process_instruction();
        io::stdout().flush().unwrap();

        let pixel = ppu.pixel();
        let scanline = ppu.scanline();

        if ppu.process(cycles * 3) == PPUResult::VBlankNMI {
            cpu.trigger_nmi();
        }

        print!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{: >3},{: >3} CYC:{}",
               regs_copy.accumulator(), regs_copy.x(), regs_copy.y(), regs_copy.status(),
               regs_copy.stack() & 0xFF, pixel, scanline, total_cycles);
        total_cycles += cycles;
        println!();
        io::stdout().flush().unwrap();

        if regs_copy.pc() == 0xC66E {
            break 'running;
        }

        if foo {
            foo = false;
            let vramb = vram.borrow();
            cpu.print_foo();

            println!("oam start");
            for x in vramb.oam.iter() {
                println!("{:02X}", x);
            }
            println!("oam end");

            let nametable = vramb.nametable0();
            for tiley in 0..30 {
                for tilex in 0..32 {
                    let val = nametable[tiley * 32 + tilex];

                    let x = tilex * 8;
                    let y = tiley * 8;

                    let mut red = 0;

                    if val > 0 {
                        red = 255;
                    }

                    for suby in 0..8 {
                        for subx in 0..8 {
                            pixels[((y + suby) * 256 * 3) + ((x + subx) * 3)] = red;
                            pixels[((y + suby) * 256 * 3) + ((x + subx) * 3) + 1] = 0;
                            pixels[((y + suby) * 256 * 3) + ((x + subx) * 3) + 2] = 0;
                        }
                    }
                }
            }

            texture.set_pixels(256, 240, pixels.to_vec());
            texture.bind();
        }

        shader_program.set_used();
        unsafe {
            gl::BindVertexArray(vao);
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                6,
            );
        }

        if iteration == 200 {
            iteration = 0;
            window.swap();
        }
    }
}
