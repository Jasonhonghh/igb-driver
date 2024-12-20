use dma_api::{DVec, Direction};
use crate::{descriptor::Descriptor, err::IgbError, regs::Reg,};

pub const DEFAULT_RING_SIZE: usize = 64;


pub struct Ring<D: Descriptor> {
    pub descriptors: DVec<D>,
    pub buf_addr:[u64;DEFAULT_RING_SIZE],
    reg: Reg,
}

impl<D: Descriptor> Ring<D> {
    pub fn new(reg: Reg, size: usize) -> Result<Self, IgbError> {
        let descriptors =
            DVec::zeros(size, 4096, Direction::Bidirectional).ok_or(IgbError::NoMemory)?;
        let buf_addr = [0 as u64;DEFAULT_RING_SIZE];
        Ok(Self { descriptors,buf_addr, reg })
    }


    pub fn init(&mut self){
        // For each descriptor, assign a pointer to a packet buffer
        for i in 0..self.descriptors.len() {
            let packet_buffer: DVec<u8> =
                DVec::zeros(1, 1024, Direction::Bidirectional)
                    .ok_or(IgbError::NoMemory).unwrap();// Allocate memory for packet buffer
            let packet_buffer_addr = packet_buffer.bus_addr();
            self.buf_addr[i]=packet_buffer_addr;
            //Get the physical address of the packet buffer
            let init_descriptor = D::new(packet_buffer_addr);
            self.descriptors.set(i, init_descriptor);
        }
    }
}
