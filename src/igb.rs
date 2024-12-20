use core::{ptr::NonNull, time::Duration};

use log::debug;

use crate::{err::IgbError, sleep,phy_to_vir};
use alloc::vec::Vec;

use crate::{
    descriptor::{AdvRxDesc, AdvTxDesc},
    phy::Phy,
    regs::{Reg, CTRL, CTRL_EXT, RCTL, STATUS, TCTL},
    ring::{Ring, DEFAULT_RING_SIZE},
};
use crate::regs::{FlagReg, EIMS, RDBAH0, RDBAL0, RDH0, RDLEN0, RDT0, RXDCTL0, SRRCTL0, TDBAH0, TDBAL0, TDH0, TDT0, TDLEN0, TXDCTL0};
use core::slice::from_raw_parts_mut;
use crate::descriptor::{AdvRxDescRead, AdvTxDescRead};

pub struct Igb {
    reg: Reg,
    tx_ring: Ring<AdvTxDesc>,
    rx_ring: Ring<AdvRxDesc>,
    phy: Phy,
}

impl Igb {
    pub fn new(bar0: NonNull<u8>) -> Result<Self, IgbError> {
        let reg = Reg::new(bar0);
        let tx_ring = Ring::new(reg, DEFAULT_RING_SIZE)?;
        let rx_ring = Ring::new(reg, DEFAULT_RING_SIZE)?;

        Ok(Self {
            reg,
            tx_ring,
            rx_ring,
            phy: Phy::new(reg),
        })
    }

    pub fn open(&mut self) -> Result<(), IgbError> {
        self.reg.disable_interrupts();

        self.reg.write_reg(CTRL::RST);

        self.reg.wait_for(
            |reg: CTRL| !reg.contains(CTRL::RST),
            Duration::from_millis(1),
            Some(1000),
        )?;
        self.reg.disable_interrupts();

        self.reg
            .modify_reg(|reg: CTRL_EXT| CTRL_EXT::DRV_LOAD | reg);

        self.setup_phy_and_the_link()?;

        self.init_stat();

        self.init_rx();
        self.init_tx();

        self.enable_interrupts();

        self.reg
            .write_reg(CTRL::SLU | CTRL::FD | CTRL::SPD_1000 | CTRL::FRCDPX | CTRL::FRCSPD);

        Ok(())
    }

    fn init_stat(&mut self) {
        //TODO
    }
    /// 4.5.9 Receive Initialization
    fn init_rx(&mut self) {
        // disable rx when configing.
        self.reg.write_reg(RCTL::empty());

        //allocate pkt buffer and store the physical address of the buffer in the descriptor
        self.rx_ring.init();

        // get the physical address of the rx ring  and write it to the RDBAL and RDBAH registers and set the length of the rx ring
        let rx_ring_phys = self.rx_ring.descriptors.bus_addr();
        self.reg.write_32(RDBAL0::REG, rx_ring_phys as u32);
        self.reg.write_32(RDBAH0::REG, (rx_ring_phys >> 32) as u32);
        self.reg.write_32(RDLEN0::REG, (self.rx_ring.descriptors.len() as u32)*128);

        // initialize the head and tail pointers of the rx ring
        // default value is 0, need to set tail to the last descriptor
        self.reg.write_32(RDH0::REG, 0);
        self.reg.write_32(RDT0::REG, (DEFAULT_RING_SIZE-1) as u32);

        // set srrctrl desctype,deault Receive Buffer Size is 2048B
        self.reg.write_reg(SRRCTL0::DESCTYPE);

        // start queue0
        self.reg.write_reg(RXDCTL0::WTHRESH|RXDCTL0::EN);

        self.reg.wait_for(
            |reg: RXDCTL0| reg.contains(RXDCTL0::WTHRESH|RXDCTL0::EN),
            Duration::from_millis(1),
            Some(1000),
        ).unwrap();
        // MRQC disabled as default
        self.reg.write_reg(RCTL::RXEN);
        self.reg.wait_for(
            |reg: RCTL| reg.contains(RCTL::RXEN),
            Duration::from_millis(1),
            Some(1000),
        ).unwrap();
        debug!("rx queue 0 enabled, rx initialized");
    }

    fn init_tx(&mut self) {
        self.reg.write_reg(TCTL::empty());

        self.tx_ring.init();//just clear all descriptors, but nothing to do

        let tx_ring_phys = self.tx_ring.descriptors.bus_addr();
        self.reg.write_32(TDBAL0::REG, tx_ring_phys as u32);
        self.reg.write_32(TDBAH0::REG, (tx_ring_phys >> 32) as u32);
        self.reg.write_32(TDLEN0::REG, (self.rx_ring.descriptors.len() as u32)*128);

        // initialize the head and tail pointers of the rx ring
        // default value is 0, need to set tail to the last descriptor
        self.reg.write_32(TDH0::REG, 0);
        self.reg.write_32(TDT0::REG, 0);

        // start queue0
        self.reg.write_reg(TXDCTL0::WTHRESH|TXDCTL0::EN);

        self.reg.wait_for(
            |reg: TXDCTL0| reg.contains(TXDCTL0::WTHRESH|TXDCTL0::EN),
            Duration::from_millis(1),
            Some(1000),
        ).unwrap();

        self.reg.write_reg(TCTL::EN);
        self.reg.wait_for(
            |reg: TCTL| reg.contains(TCTL::EN),
            Duration::from_millis(1),
            Some(1000),
        ).unwrap();
        debug!("tx queue 0 enabled, tx initialized")
    }

    fn setup_phy_and_the_link(&mut self) -> Result<(), IgbError> {
        self.phy.power_up()?;
        Ok(())
    }

    pub fn mac(&self) -> [u8; 6] {
        self.reg.read_mac()
    }

    fn enable_interrupts(&self) {
        //set rxtxq and other cause
        let rxtxq = 0xFFFF as u32;
        let other = 1 << 31 as u32;
        self.reg.write_32(EIMS,rxtxq|other);
        debug!("enable interrupts")
    }

    pub fn status(&self) -> IgbStatus {
        let raw = self.reg.read_reg::<STATUS>();
        let speed_raw = (raw.bits() >> 6) & 0b11;

        IgbStatus {
            link_up: raw.contains(STATUS::LU),
            speed: match speed_raw {
                0 => Speed::Mb10,
                1 => Speed::Mb100,
                0b10 => Speed::Mb1000,
                _ => Speed::Mb1000,
            },
            full_duplex: raw.contains(STATUS::FD),
            phy_reset_asserted: raw.contains(STATUS::PHYRA),
        }
    }
    pub fn send(&mut self, packet: &[u8]) -> i32 {
        let tindex = self.reg.read_32(TDT0::REG) as usize;
        debug!("Read TDT0 = {:#x}", tindex);
        let buf_addr = unsafe {
            self.tx_ring.descriptors.get(tindex).unwrap().read.buffer_addr
        };


        let mut length = packet.len();

        let packet_buf = unsafe { from_raw_parts_mut(phy_to_vir(buf_addr) as *mut u8,length)};
        packet_buf.copy_from_slice(packet);

        debug!(">>>>>>>>> TX PKT {}", length);
        debug!("\n\r");

        //set txdescriptor
        let txdesc = unsafe{
            AdvTxDesc {
                read: AdvTxDescRead {
                    buffer_addr: buf_addr,
                    paylen:0,
                    ports_idx:0,
                    idx_sta:0,
                    dcmd:0b0011<<4,//set dcyp
                    dtalen:length as u16,
                }
            }
        };
        self.tx_ring.descriptors.set(tindex,txdesc);

        self.reg.write_32(TDT0::REG,((tindex + 1) % DEFAULT_RING_SIZE) as u32);


        length as i32
    }
    pub fn receive(&mut self) -> Option<Vec<Vec<u8>>> {

        let mut recv_packets = Vec::new();
        let mut rindex = (self.reg.read_32(RDT0::REG) as usize )/DEFAULT_RING_SIZE;



        debug!("Read RDT0 + 1 = {:#x}", rindex);
        let mut len = unsafe{self.rx_ring.descriptors.get(rindex).unwrap().write.pkt_len};
        let buf_addr = self.rx_ring.buf_addr[rindex];

        let pkt_buf = unsafe { from_raw_parts_mut(phy_to_vir(buf_addr) as *mut u8, len as usize) };
        debug!("RX PKT {} <<<<<<<<<", len);
        //recv_packets.push_back(mbuf.to_vec());
        recv_packets.push(pkt_buf.to_vec());

        let rxdesc = unsafe{
            AdvRxDesc {
                read: AdvRxDescRead {
                    pkt_addr: buf_addr,
                    hdr_addr:0,
                }
            }
        };
        self.rx_ring.descriptors.set(rindex,rxdesc);

        self.reg.write_32(RDT0::REG,((rindex + 1) % DEFAULT_RING_SIZE) as u32);


        if recv_packets.len() > 0 {
            debug!("receive a pakcet");
            Some(recv_packets)
        } else {
            None
        }
    }

}

#[derive(Debug, Clone)]
pub struct IgbStatus {
    pub full_duplex: bool,
    pub link_up: bool,
    pub speed: Speed,
    pub phy_reset_asserted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Speed {
    Mb10,
    Mb100,
    Mb1000,
}
