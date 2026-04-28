//! Safe Rust wrappers over the raw FFI bindings.
//!
//! Provides proper Rust enums for NVSP/RNDIS/VMBus protocol constants
//! and type-safe accessors for wire-format structs.

// Re-export raw bindings for direct access when needed.
pub use crate::ffi;

// ── NVSP protocol versions ──────────────────────────────────────────────

/// NVSP protocol version negotiated between VSC (guest) and VSP (host).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NvspVersion {
    V1 = ffi::NVSP_PROTOCOL_VERSION_1,
    V2 = ffi::NVSP_PROTOCOL_VERSION_2,
    V4 = ffi::NVSP_PROTOCOL_VERSION_4,
    V5 = ffi::NVSP_PROTOCOL_VERSION_5,
    V6 = ffi::NVSP_PROTOCOL_VERSION_6,
    V61 = ffi::NVSP_PROTOCOL_VERSION_61,
}

impl NvspVersion {
    /// Versions to try during negotiation, newest first.
    pub const NEGOTIATE_ORDER: &[NvspVersion] = &[
        NvspVersion::V61,
        NvspVersion::V6,
        NvspVersion::V5,
        NvspVersion::V4,
        NvspVersion::V2,
        NvspVersion::V1,
    ];

    pub const fn as_u32(self) -> u32 {
        self as u32
    }
}

// ── NVSP message types ──────────────────────────────────────────────────

/// NVSP message type sent/received on the VMBus channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NvspMessageType {
    None = ffi::NVSP_MSG_TYPE_NONE,
    Init = ffi::NVSP_MSG_TYPE_INIT,
    InitComplete = ffi::NVSP_MSG_TYPE_INIT_COMPLETE,

    // Version 1
    SendNdisVersion = ffi::NVSP_MSG1_TYPE_SEND_NDIS_VER,
    SendReceiveBuffer = ffi::NVSP_MSG1_TYPE_SEND_RECV_BUF,
    SendReceiveBufferComplete = ffi::NVSP_MSG1_TYPE_SEND_RECV_BUF_COMPLETE,
    RevokeReceiveBuffer = ffi::NVSP_MSG1_TYPE_REVOKE_RECV_BUF,
    SendSendBuffer = ffi::NVSP_MSG1_TYPE_SEND_SEND_BUF,
    SendSendBufferComplete = ffi::NVSP_MSG1_TYPE_SEND_SEND_BUF_COMPLETE,
    RevokeSendBuffer = ffi::NVSP_MSG1_TYPE_REVOKE_SEND_BUF,
    SendRndisPkt = ffi::NVSP_MSG1_TYPE_SEND_RNDIS_PKT,
    SendRndisPktComplete = ffi::NVSP_MSG1_TYPE_SEND_RNDIS_PKT_COMPLETE,

    // Version 2
    SendNdisConfig = ffi::NVSP_MSG2_TYPE_SEND_NDIS_CONFIG,

    // Version 4
    SendVfAssociation = ffi::NVSP_MSG4_TYPE_SEND_VF_ASSOCIATION,
    SwitchDataPath = ffi::NVSP_MSG4_TYPE_SWITCH_DATA_PATH,

    // Version 5
    SubChannel = ffi::NVSP_MSG5_TYPE_SUBCHANNEL,
    SendIndirectionTable = ffi::NVSP_MSG5_TYPE_SEND_INDIRECTION_TABLE,
}

impl NvspMessageType {
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            ffi::NVSP_MSG_TYPE_NONE => Some(Self::None),
            ffi::NVSP_MSG_TYPE_INIT => Some(Self::Init),
            ffi::NVSP_MSG_TYPE_INIT_COMPLETE => Some(Self::InitComplete),
            ffi::NVSP_MSG1_TYPE_SEND_NDIS_VER => Some(Self::SendNdisVersion),
            ffi::NVSP_MSG1_TYPE_SEND_RECV_BUF => Some(Self::SendReceiveBuffer),
            ffi::NVSP_MSG1_TYPE_SEND_RECV_BUF_COMPLETE => Some(Self::SendReceiveBufferComplete),
            ffi::NVSP_MSG1_TYPE_REVOKE_RECV_BUF => Some(Self::RevokeReceiveBuffer),
            ffi::NVSP_MSG1_TYPE_SEND_SEND_BUF => Some(Self::SendSendBuffer),
            ffi::NVSP_MSG1_TYPE_SEND_SEND_BUF_COMPLETE => Some(Self::SendSendBufferComplete),
            ffi::NVSP_MSG1_TYPE_REVOKE_SEND_BUF => Some(Self::RevokeSendBuffer),
            ffi::NVSP_MSG1_TYPE_SEND_RNDIS_PKT => Some(Self::SendRndisPkt),
            ffi::NVSP_MSG1_TYPE_SEND_RNDIS_PKT_COMPLETE => Some(Self::SendRndisPktComplete),
            ffi::NVSP_MSG2_TYPE_SEND_NDIS_CONFIG => Some(Self::SendNdisConfig),
            ffi::NVSP_MSG4_TYPE_SEND_VF_ASSOCIATION => Some(Self::SendVfAssociation),
            ffi::NVSP_MSG4_TYPE_SWITCH_DATA_PATH => Some(Self::SwitchDataPath),
            ffi::NVSP_MSG5_TYPE_SUBCHANNEL => Some(Self::SubChannel),
            ffi::NVSP_MSG5_TYPE_SEND_INDIRECTION_TABLE => Some(Self::SendIndirectionTable),
            _ => None,
        }
    }
}

// ── NVSP status ─────────────────────────────────────────────────────────

/// Status code returned in NVSP completion messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NvspStatus {
    None = ffi::NVSP_STAT_NONE,
    Success = ffi::NVSP_STAT_SUCCESS,
    Fail = ffi::NVSP_STAT_FAIL,
    ProtocolTooNew = ffi::NVSP_STAT_PROTOCOL_TOO_NEW,
    ProtocolTooOld = ffi::NVSP_STAT_PROTOCOL_TOO_OLD,
    InvalidRndisPkt = ffi::NVSP_STAT_INVALID_RNDIS_PKT,
    Busy = ffi::NVSP_STAT_BUSY,
    ProtocolUnsupported = ffi::NVSP_STAT_PROTOCOL_UNSUPPORTED,
}

impl NvspStatus {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            ffi::NVSP_STAT_NONE => Some(Self::None),
            ffi::NVSP_STAT_SUCCESS => Some(Self::Success),
            ffi::NVSP_STAT_FAIL => Some(Self::Fail),
            ffi::NVSP_STAT_PROTOCOL_TOO_NEW => Some(Self::ProtocolTooNew),
            ffi::NVSP_STAT_PROTOCOL_TOO_OLD => Some(Self::ProtocolTooOld),
            ffi::NVSP_STAT_INVALID_RNDIS_PKT => Some(Self::InvalidRndisPkt),
            ffi::NVSP_STAT_BUSY => Some(Self::Busy),
            ffi::NVSP_STAT_PROTOCOL_UNSUPPORTED => Some(Self::ProtocolUnsupported),
            _ => None,
        }
    }

    pub fn is_success(self) -> bool {
        self == Self::Success
    }
}

// ── RNDIS message types ─────────────────────────────────────────────────

/// RNDIS message type (ndis_msg_type field of rndis_message).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RndisMessageType {
    Packet = ffi::RNDIS_MSG_PACKET,
    Init = ffi::RNDIS_MSG_INIT,
    InitComplete = ffi::RNDIS_MSG_INIT_C,
    Halt = ffi::RNDIS_MSG_HALT,
    Query = ffi::RNDIS_MSG_QUERY,
    QueryComplete = ffi::RNDIS_MSG_QUERY_C,
    Set = ffi::RNDIS_MSG_SET,
    SetComplete = ffi::RNDIS_MSG_SET_C,
    Reset = ffi::RNDIS_MSG_RESET,
    ResetComplete = ffi::RNDIS_MSG_RESET_C,
    Indicate = ffi::RNDIS_MSG_INDICATE,
    KeepAlive = ffi::RNDIS_MSG_KEEPALIVE,
    KeepAliveComplete = ffi::RNDIS_MSG_KEEPALIVE_C,
}

impl RndisMessageType {
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Returns true if this is a completion (response) message type.
    pub const fn is_completion(self) -> bool {
        (self as u32) & ffi::RNDIS_MSG_COMPLETION != 0
    }

    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            ffi::RNDIS_MSG_PACKET => Some(Self::Packet),
            ffi::RNDIS_MSG_INIT => Some(Self::Init),
            ffi::RNDIS_MSG_INIT_C => Some(Self::InitComplete),
            ffi::RNDIS_MSG_HALT => Some(Self::Halt),
            ffi::RNDIS_MSG_QUERY => Some(Self::Query),
            ffi::RNDIS_MSG_QUERY_C => Some(Self::QueryComplete),
            ffi::RNDIS_MSG_SET => Some(Self::Set),
            ffi::RNDIS_MSG_SET_C => Some(Self::SetComplete),
            ffi::RNDIS_MSG_RESET => Some(Self::Reset),
            ffi::RNDIS_MSG_RESET_C => Some(Self::ResetComplete),
            ffi::RNDIS_MSG_INDICATE => Some(Self::Indicate),
            ffi::RNDIS_MSG_KEEPALIVE => Some(Self::KeepAlive),
            ffi::RNDIS_MSG_KEEPALIVE_C => Some(Self::KeepAliveComplete),
            _ => None,
        }
    }
}

// ── RNDIS status ────────────────────────────────────────────────────────

/// RNDIS status code returned in completion messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct RndisStatus(pub u32);

impl RndisStatus {
    pub const SUCCESS: Self = Self(ffi::RNDIS_STATUS_SUCCESS);
    pub const PENDING: Self = Self(ffi::RNDIS_STATUS_PENDING);
    pub const MEDIA_CONNECT: Self = Self(ffi::RNDIS_STATUS_MEDIA_CONNECT);
    pub const MEDIA_DISCONNECT: Self = Self(ffi::RNDIS_STATUS_MEDIA_DISCONNECT);
    pub const NETWORK_CHANGE: Self = Self(ffi::RNDIS_STATUS_NETWORK_CHANGE);
    pub const FAILURE: Self = Self(ffi::RNDIS_STATUS_FAILURE);
    pub const NOT_SUPPORTED: Self = Self(ffi::RNDIS_STATUS_NOT_SUPPORTED);
    pub const RESOURCES: Self = Self(ffi::RNDIS_STATUS_RESOURCES);
    pub const INVALID_DATA: Self = Self(ffi::RNDIS_STATUS_INVALID_DATA);

    pub fn is_success(self) -> bool {
        self.0 == ffi::RNDIS_STATUS_SUCCESS
    }

    pub fn is_error(self) -> bool {
        // NDIS error codes have bit 31 set (0xC0000000 range)
        self.0 & 0xC000_0000 == 0xC000_0000
    }
}

impl core::fmt::Display for RndisStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::SUCCESS => write!(f, "SUCCESS"),
            Self::PENDING => write!(f, "PENDING"),
            Self::MEDIA_CONNECT => write!(f, "MEDIA_CONNECT"),
            Self::MEDIA_DISCONNECT => write!(f, "MEDIA_DISCONNECT"),
            Self::FAILURE => write!(f, "FAILURE"),
            Self::NOT_SUPPORTED => write!(f, "NOT_SUPPORTED"),
            _ => write!(f, "{:#010x}", self.0),
        }
    }
}

// ── VMBus packet types ──────────────────────────────────────────────────

/// VMBus ring buffer packet type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum VmbusPacketType {
    DataInBand = ffi::VM_PKT_DATA_INBAND as u16,
    DataUsingXferPages = ffi::VM_PKT_DATA_USING_XFER_PAGES as u16,
    DataUsingGpaDirect = ffi::VM_PKT_DATA_USING_GPA_DIRECT as u16,
    Completion = ffi::VM_PKT_COMP as u16,
}

impl VmbusPacketType {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v as u32 {
            ffi::VM_PKT_DATA_INBAND => Some(Self::DataInBand),
            ffi::VM_PKT_DATA_USING_XFER_PAGES => Some(Self::DataUsingXferPages),
            ffi::VM_PKT_DATA_USING_GPA_DIRECT => Some(Self::DataUsingGpaDirect),
            ffi::VM_PKT_COMP => Some(Self::Completion),
            _ => None,
        }
    }
}

// ── NDIS OIDs ───────────────────────────────────────────────────────────

/// Common NDIS Object Identifiers used in RNDIS QUERY/SET messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct NdisOid(pub u32);

impl NdisOid {
    // General OIDs
    pub const GEN_SUPPORTED_LIST: Self = Self(ffi::OID_GEN_SUPPORTED_LIST);
    pub const GEN_HARDWARE_STATUS: Self = Self(ffi::OID_GEN_HARDWARE_STATUS);
    pub const GEN_MAXIMUM_FRAME_SIZE: Self = Self(ffi::OID_GEN_MAXIMUM_FRAME_SIZE);
    pub const GEN_LINK_SPEED: Self = Self(ffi::OID_GEN_LINK_SPEED);
    pub const GEN_CURRENT_PACKET_FILTER: Self = Self(ffi::OID_GEN_CURRENT_PACKET_FILTER);
    pub const GEN_CURRENT_LOOKAHEAD: Self = Self(ffi::OID_GEN_CURRENT_LOOKAHEAD);
    pub const GEN_MAXIMUM_TOTAL_SIZE: Self = Self(ffi::OID_GEN_MAXIMUM_TOTAL_SIZE);
    pub const GEN_MEDIA_CONNECT_STATUS: Self = Self(ffi::OID_GEN_MEDIA_CONNECT_STATUS);

    // 802.3 (Ethernet) OIDs
    pub const ETH_PERMANENT_ADDRESS: Self = Self(ffi::OID_802_3_PERMANENT_ADDRESS);
    pub const ETH_CURRENT_ADDRESS: Self = Self(ffi::OID_802_3_CURRENT_ADDRESS);
    pub const ETH_MAXIMUM_LIST_SIZE: Self = Self(ffi::OID_802_3_MAXIMUM_LIST_SIZE);
    pub const ETH_MULTICAST_LIST: Self = Self(ffi::OID_802_3_MULTICAST_LIST);

    // Offload OIDs
    pub const TCP_OFFLOAD_PARAMETERS: Self = Self(ffi::OID_TCP_OFFLOAD_PARAMETERS);
    pub const TCP_OFFLOAD_HARDWARE_CAPABILITIES: Self =
        Self(ffi::OID_TCP_OFFLOAD_HARDWARE_CAPABILITIES);

    // RSS OIDs
    pub const GEN_RECEIVE_SCALE_CAPABILITIES: Self = Self(ffi::OID_GEN_RECEIVE_SCALE_CAPABILITIES);
    pub const GEN_RECEIVE_SCALE_PARAMETERS: Self = Self(ffi::OID_GEN_RECEIVE_SCALE_PARAMETERS);
}

impl core::fmt::Display for NdisOid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::GEN_MAXIMUM_FRAME_SIZE => write!(f, "GEN_MAXIMUM_FRAME_SIZE"),
            Self::GEN_CURRENT_PACKET_FILTER => write!(f, "GEN_CURRENT_PACKET_FILTER"),
            Self::ETH_PERMANENT_ADDRESS => write!(f, "802_3_PERMANENT_ADDRESS"),
            Self::ETH_CURRENT_ADDRESS => write!(f, "802_3_CURRENT_ADDRESS"),
            _ => write!(f, "OID({:#010x})", self.0),
        }
    }
}

// ── NDIS packet filter (bitflags) ───────────────────────────────────────

/// NDIS packet filter flags for OID_GEN_CURRENT_PACKET_FILTER.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct NdisPacketFilter(pub u32);

impl NdisPacketFilter {
    pub const DIRECTED: Self = Self(ffi::NDIS_PACKET_TYPE_DIRECTED);
    pub const MULTICAST: Self = Self(ffi::NDIS_PACKET_TYPE_MULTICAST);
    pub const ALL_MULTICAST: Self = Self(ffi::NDIS_PACKET_TYPE_ALL_MULTICAST);
    pub const BROADCAST: Self = Self(ffi::NDIS_PACKET_TYPE_BROADCAST);
    pub const PROMISCUOUS: Self = Self(ffi::NDIS_PACKET_TYPE_PROMISCUOUS);

    /// Standard filter for normal operation: unicast + multicast + broadcast.
    pub const STANDARD: Self = Self(
        ffi::NDIS_PACKET_TYPE_DIRECTED
            | ffi::NDIS_PACKET_TYPE_MULTICAST
            | ffi::NDIS_PACKET_TYPE_BROADCAST,
    );
}

impl core::ops::BitOr for NdisPacketFilter {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for NdisPacketFilter {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

// ── VSC capability flags ────────────────────────────────────────────────

/// Flags for nvsp_2_vsc_capability.data (bitfield over u64).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VscCapability(pub u64);

impl VscCapability {
    pub const VMQ: u64 = 1 << 0;
    pub const CHIMNEY: u64 = 1 << 1;
    pub const SRIOV: u64 = 1 << 2;
    pub const IEEE8021Q: u64 = 1 << 3;
    pub const CORRELATION_ID: u64 = 1 << 4;
    pub const TEAMING: u64 = 1 << 5;
    pub const VSUBNETID: u64 = 1 << 6;
    pub const RSC: u64 = 1 << 7;
}

// ── Buffer constants ────────────────────────────────────────────────────

/// Well-known buffer IDs used during NVSP buffer registration.
pub mod buffer {
    pub const RECEIVE_BUFFER_ID: u16 = super::ffi::NETVSC_RECEIVE_BUFFER_ID as u16;
    pub const SEND_BUFFER_ID: u16 = super::ffi::NETVSC_SEND_BUFFER_ID as u16;

    pub const RECEIVE_BUFFER_SIZE: usize = super::ffi::NETVSC_RECEIVE_BUFFER_DEFAULT as usize;
    pub const SEND_BUFFER_SIZE: usize = super::ffi::NETVSC_SEND_BUFFER_DEFAULT as usize;

    pub const SEND_SECTION_SIZE: usize = super::ffi::NETVSC_SEND_SECTION_SIZE as usize;
    pub const RECV_SECTION_SIZE: usize = super::ffi::NETVSC_RECV_SECTION_SIZE as usize;

    pub const RNDIS_MAX_PKT_DEFAULT: u32 = super::ffi::RNDIS_MAX_PKT_DEFAULT;
    pub const RNDIS_PKT_ALIGN_DEFAULT: u32 = super::ffi::RNDIS_PKT_ALIGN_DEFAULT;
}
