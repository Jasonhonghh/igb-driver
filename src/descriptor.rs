pub trait Descriptor {
    fn new( addr: u64)->Self;
}

#[derive(Clone, Copy)]
pub union AdvTxDesc {
    pub read: AdvTxDescRead,
    pub write: AdvTxDescWB,
}

impl Descriptor for AdvTxDesc {
    fn new(addr: u64) -> Self{
        AdvTxDesc {
            read: AdvTxDescRead {
                buffer_addr: addr,
                paylen:0,
                ports_idx:0,
                idx_sta:0,
                dcmd:0,
                dtalen:0
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvTxDescRead {
    pub buffer_addr: u64,
    pub paylen:u16,
    pub ports_idx:u8,
    pub idx_sta:u8,
    pub dcmd:u16,
    pub dtalen:u16,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvTxDescWB {
    //暂时不用修改
    pub rsvd: u64,
    pub nxtseq_seed: u32,
    pub status: u32,
}

#[derive(Clone, Copy)]
pub union AdvRxDesc {
    pub read: AdvRxDescRead,
    pub write: AdvRxDescWB,
}

impl Descriptor for AdvRxDesc {
    fn new(addr: u64) -> Self {
        AdvRxDesc {
            read: AdvRxDescRead {
                pkt_addr: addr,
                hdr_addr: 0,
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvRxDescRead {
    pub pkt_addr: u64,
    pub hdr_addr: u64,
}
#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvRxDescWB {
    pub rsshash:u32,
    pub head_packet_info:u32,
    pub vlan:u16,
    pub pkt_len:u16,
    pub err__status:u32,
}

#[derive(Clone, Copy)]
pub union LoDword {
    pub data: u32,
    pub hs_rss: HsRss,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct HsRss {
    pub pkt_info: u16,
    pub hdr_info: u16,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union HiDword {
    pub rss: u32, // RSS Hash
    pub csum_ip: CsumIp,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CsumIp {
    pub ip_id: u16, // IP id
    pub csum: u16,  // Packet Checksum
}
