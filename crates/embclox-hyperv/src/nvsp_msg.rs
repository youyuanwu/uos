//! Parsed NVSP and RNDIS message types for type-safe protocol handling.
//!
//! Provides zero-copy parsing of received messages and builder functions
//! for constructing outgoing messages.

// FFI union types can't use struct literal syntax — field reassignment after Default is unavoidable.
#![allow(clippy::field_reassign_with_default)]

use crate::ffi;
use crate::nvsp_types::*;
use core::mem;

// ── NVSP message size ───────────────────────────────────────────────────

/// Minimum NVSP message size expected by the host (header + union).
pub const NVSP_MESSAGE_SIZE: usize = mem::size_of::<ffi::nvsp_message>();

/// RNDIS header size: ndis_msg_type(4) + msg_len(4).
pub const RNDIS_HEADER_SIZE: usize = 8;

// ── NVSP parsed responses ───────────────────────────────────────────────

/// A parsed NVSP response received from the host.
///
/// Created by [`parse_nvsp_response`] from a raw message type + payload.
#[derive(Debug)]
pub enum NvspResponse<'a> {
    /// Response to NVSP_MSG_TYPE_INIT.
    InitComplete {
        negotiated_version: u32,
        max_mdl_chain_len: u32,
        status: u32,
    },
    /// Response to NVSP_MSG1_TYPE_SEND_RECV_BUF.
    RecvBufComplete {
        status: u32,
        num_sections: u32,
        sections: &'a [ffi::nvsp_1_receive_buffer_section],
    },
    /// Response to NVSP_MSG1_TYPE_SEND_SEND_BUF.
    SendBufComplete { status: u32, section_size: u32 },
    /// Response to NVSP_MSG1_TYPE_SEND_RNDIS_PKT.
    RndisPktComplete { status: u32 },
    /// Unrecognized message type.
    Unknown(u32),
}

/// Parse an NVSP response from the payload bytes (after the vmpacket_descriptor).
///
/// `data` should contain the full nvsp_message (header + body).
pub fn parse_nvsp_response(data: &[u8]) -> Option<NvspResponse<'_>> {
    if data.len() < mem::size_of::<ffi::nvsp_message_header>() {
        return None;
    }
    // Read msg_type directly (nvsp_message is packed, can't take field references)
    let msg_type = u32::from_le_bytes(data[0..4].try_into().ok()?);
    let body = &data[mem::size_of::<ffi::nvsp_message_header>()..];

    match msg_type {
        x if x == NvspMessageType::InitComplete.as_u32() => {
            let c = unsafe { cast_ref::<ffi::nvsp_message_init_complete>(body)? };
            Some(NvspResponse::InitComplete {
                negotiated_version: c.negotiated_protocol_ver,
                max_mdl_chain_len: c.max_mdl_chain_len,
                status: c.status,
            })
        }
        x if x == NvspMessageType::SendReceiveBufferComplete.as_u32() => {
            let c = unsafe { cast_ref::<ffi::nvsp_1_message_send_receive_buffer_complete>(body)? };
            let section_size = mem::size_of::<ffi::nvsp_1_receive_buffer_section>();
            let sections_data = &body[8..]; // after status(4) + num_sections(4)
            let n = (sections_data.len() / section_size).min(c.num_sections as usize);
            let sections = if n > 0 {
                unsafe {
                    core::slice::from_raw_parts(
                        sections_data.as_ptr() as *const ffi::nvsp_1_receive_buffer_section,
                        n,
                    )
                }
            } else {
                &[]
            };
            Some(NvspResponse::RecvBufComplete {
                status: c.status,
                num_sections: c.num_sections,
                sections,
            })
        }
        x if x == NvspMessageType::SendSendBufferComplete.as_u32() => {
            let c = unsafe { cast_ref::<ffi::nvsp_1_message_send_send_buffer_complete>(body)? };
            Some(NvspResponse::SendBufComplete {
                status: c.status,
                section_size: c.section_size,
            })
        }
        x if x == NvspMessageType::SendRndisPktComplete.as_u32() => {
            let c = unsafe { cast_ref::<ffi::nvsp_1_message_send_rndis_packet_complete>(body)? };
            Some(NvspResponse::RndisPktComplete { status: c.status })
        }
        _ => Some(NvspResponse::Unknown(msg_type)),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

/// Cast a byte slice to a reference to a packed FFI struct.
///
/// # Safety
/// The caller must ensure the byte slice contains valid data for type `T`.
/// Safe for our `repr(C, packed)` FFI types which have alignment 1.
unsafe fn cast_ref<T>(data: &[u8]) -> Option<&T> {
    if data.len() < mem::size_of::<T>() {
        return None;
    }
    Some(&*(data.as_ptr() as *const T))
}

/// View an `nvsp_message` as a byte slice.
pub fn nvsp_message_as_bytes(msg: &ffi::nvsp_message) -> &[u8] {
    unsafe { core::slice::from_raw_parts(msg as *const _ as *const u8, NVSP_MESSAGE_SIZE) }
}

/// View an `rndis_message` as a byte slice.
pub fn rndis_message_as_bytes(msg: &ffi::rndis_message) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            msg as *const _ as *const u8,
            mem::size_of::<ffi::rndis_message>(),
        )
    }
}

// ── NVSP message builders ───────────────────────────────────────────────

/// Build an NVSP_MSG_TYPE_INIT message.
pub fn build_nvsp_init(version: u32) -> ffi::nvsp_message {
    let mut msg = ffi::nvsp_message::default();
    msg.hdr.msg_type = NvspMessageType::Init.as_u32();
    unsafe {
        let init = msg.msg.init_msg.as_mut();
        init.init.min_protocol_ver = version;
        init.init.max_protocol_ver = version;
    }
    msg
}

/// Build an NVSP_MSG1_TYPE_SEND_NDIS_VER message.
pub fn build_nvsp_send_ndis_version(major: u32, minor: u32) -> ffi::nvsp_message {
    let mut msg = ffi::nvsp_message::default();
    msg.hdr.msg_type = NvspMessageType::SendNdisVersion.as_u32();
    unsafe {
        let ver = msg.msg.v1_msg.as_mut().send_ndis_ver.as_mut();
        ver.ndis_major_ver = major;
        ver.ndis_minor_ver = minor;
    }
    msg
}

/// Build an NVSP_MSG2_TYPE_SEND_NDIS_CONFIG message.
pub fn build_nvsp_send_ndis_config(mtu: u32, capabilities: u64) -> ffi::nvsp_message {
    let mut msg = ffi::nvsp_message::default();
    msg.hdr.msg_type = NvspMessageType::SendNdisConfig.as_u32();
    unsafe {
        let v2 = msg.msg.v2_msg.as_mut();
        v2.send_ndis_config.mtu = mtu;
        v2.send_ndis_config.capability.data = capabilities;
    }
    msg
}

/// Build an NVSP_MSG1_TYPE_SEND_RECV_BUF message.
pub fn build_nvsp_send_recv_buf(gpadl_handle: u32, id: u16) -> ffi::nvsp_message {
    let mut msg = ffi::nvsp_message::default();
    msg.hdr.msg_type = NvspMessageType::SendReceiveBuffer.as_u32();
    unsafe {
        let buf = msg.msg.v1_msg.as_mut().send_recv_buf.as_mut();
        buf.gpadl_handle = gpadl_handle;
        buf.id = id;
    }
    msg
}

/// Build an NVSP_MSG1_TYPE_SEND_SEND_BUF message.
pub fn build_nvsp_send_send_buf(gpadl_handle: u32, id: u16) -> ffi::nvsp_message {
    let mut msg = ffi::nvsp_message::default();
    msg.hdr.msg_type = NvspMessageType::SendSendBuffer.as_u32();
    unsafe {
        let buf = msg.msg.v1_msg.as_mut().send_send_buf.as_mut();
        buf.gpadl_handle = gpadl_handle;
        buf.id = id;
    }
    msg
}

/// Build an NVSP_MSG1_TYPE_SEND_RNDIS_PKT message.
///
/// `channel_type`: 0 = data, 1 = control.
/// `send_buf_section_index`: index into send buffer, or 0xFFFFFFFF if not using send buffer.
/// `send_buf_section_size`: size of send buffer section used, or 0.
pub fn build_nvsp_send_rndis_pkt(
    channel_type: u32,
    send_buf_section_index: u32,
    send_buf_section_size: u32,
) -> ffi::nvsp_message {
    let mut msg = ffi::nvsp_message::default();
    msg.hdr.msg_type = NvspMessageType::SendRndisPkt.as_u32();
    unsafe {
        let pkt = msg.msg.v1_msg.as_mut().send_rndis_pkt.as_mut();
        pkt.channel_type = channel_type;
        pkt.send_buf_section_index = send_buf_section_index;
        pkt.send_buf_section_size = send_buf_section_size;
    }
    msg
}

// ── RNDIS parsed responses ──────────────────────────────────────────────

/// A parsed RNDIS response received from the host.
///
/// Created by [`parse_rndis_response`] from raw message bytes.
#[derive(Debug)]
pub enum RndisResponse<'a> {
    /// RNDIS_MSG_INIT_C: response to INITIALIZE.
    InitComplete {
        req_id: u32,
        status: RndisStatus,
        major_ver: u32,
        minor_ver: u32,
        max_pkt_per_msg: u32,
        max_xfer_size: u32,
        pkt_alignment_factor: u32,
    },
    /// RNDIS_MSG_QUERY_C: response to QUERY.
    QueryComplete {
        req_id: u32,
        status: RndisStatus,
        info: &'a [u8],
    },
    /// RNDIS_MSG_SET_C: response to SET.
    SetComplete { req_id: u32, status: RndisStatus },
    /// RNDIS_MSG_KEEPALIVE_C: response to KEEPALIVE.
    KeepAliveComplete { req_id: u32, status: RndisStatus },
    /// RNDIS_MSG_INDICATE: unsolicited status indication.
    Indicate {
        status: RndisStatus,
        status_buf: &'a [u8],
    },
    /// RNDIS_MSG_PACKET: data packet.
    Packet {
        data_offset: u32,
        data_len: u32,
        /// The full message bytes for further PPI parsing.
        raw: &'a [u8],
    },
    /// Unrecognized message type.
    Unknown(u32),
}

/// Parse an RNDIS response from message bytes.
///
/// `data` should contain the full rndis_message starting from ndis_msg_type.
pub fn parse_rndis_response(data: &[u8]) -> Option<RndisResponse<'_>> {
    if data.len() < RNDIS_HEADER_SIZE {
        return None;
    }
    let msg_type = u32::from_le_bytes(data[0..4].try_into().ok()?);
    let msg_len = u32::from_le_bytes(data[4..8].try_into().ok()?) as usize;
    let body = &data[RNDIS_HEADER_SIZE..data.len().min(msg_len)];

    match RndisMessageType::from_u32(msg_type) {
        Some(RndisMessageType::InitComplete) => {
            let c = unsafe { cast_ref::<ffi::rndis_initialize_complete>(body)? };
            Some(RndisResponse::InitComplete {
                req_id: c.req_id,
                status: RndisStatus(c.status),
                major_ver: c.major_ver,
                minor_ver: c.minor_ver,
                max_pkt_per_msg: c.max_pkt_per_msg,
                max_xfer_size: c.max_xfer_size,
                pkt_alignment_factor: c.pkt_alignment_factor,
            })
        }
        Some(RndisMessageType::QueryComplete) => {
            let c = unsafe { cast_ref::<ffi::rndis_query_complete>(body)? };
            let info_buflen = c.info_buflen as usize;
            let info_buf_offset = c.info_buf_offset as usize;
            let info = if info_buflen > 0 && info_buf_offset + info_buflen <= body.len() {
                &body[info_buf_offset..info_buf_offset + info_buflen]
            } else {
                &[]
            };
            Some(RndisResponse::QueryComplete {
                req_id: c.req_id,
                status: RndisStatus(c.status),
                info,
            })
        }
        Some(RndisMessageType::SetComplete) => {
            let c = unsafe { cast_ref::<ffi::rndis_set_complete>(body)? };
            Some(RndisResponse::SetComplete {
                req_id: c.req_id,
                status: RndisStatus(c.status),
            })
        }
        Some(RndisMessageType::KeepAliveComplete) => {
            let c = unsafe { cast_ref::<ffi::rndis_keepalive_complete>(body)? };
            Some(RndisResponse::KeepAliveComplete {
                req_id: c.req_id,
                status: RndisStatus(c.status),
            })
        }
        Some(RndisMessageType::Indicate) => {
            let c = unsafe { cast_ref::<ffi::rndis_indicate_status>(body)? };
            let status_buflen = c.status_buflen as usize;
            let status_buf_offset = c.status_buf_offset as usize;
            let status_buf = if status_buflen > 0 && status_buf_offset + status_buflen <= body.len()
            {
                &body[status_buf_offset..status_buf_offset + status_buflen]
            } else {
                &[]
            };
            Some(RndisResponse::Indicate {
                status: RndisStatus(c.status),
                status_buf,
            })
        }
        Some(RndisMessageType::Packet) => {
            let c = unsafe { cast_ref::<ffi::rndis_packet>(body)? };
            Some(RndisResponse::Packet {
                data_offset: c.data_offset,
                data_len: c.data_len,
                raw: data,
            })
        }
        _ => Some(RndisResponse::Unknown(msg_type)),
    }
}

// ── RNDIS message builders ──────────────────────────────────────────────

/// Build an RNDIS_MSG_INIT message.
pub fn build_rndis_init(req_id: u32, max_xfer_size: u32) -> ffi::rndis_message {
    let mut msg = ffi::rndis_message::default();
    msg.ndis_msg_type = RndisMessageType::Init.as_u32();
    msg.msg_len = (RNDIS_HEADER_SIZE + mem::size_of::<ffi::rndis_initialize_request>()) as u32;
    unsafe {
        let init = msg.msg.init_req.as_mut();
        init.req_id = req_id;
        init.major_ver = ffi::RNDIS_MAJOR_VERSION;
        init.minor_ver = ffi::RNDIS_MINOR_VERSION;
        init.max_xfer_size = max_xfer_size;
    }
    msg
}

/// Build an RNDIS_MSG_HALT message.
pub fn build_rndis_halt(req_id: u32) -> ffi::rndis_message {
    let mut msg = ffi::rndis_message::default();
    msg.ndis_msg_type = RndisMessageType::Halt.as_u32();
    msg.msg_len = (RNDIS_HEADER_SIZE + mem::size_of::<ffi::rndis_halt_request>()) as u32;
    unsafe {
        msg.msg.halt_req.as_mut().req_id = req_id;
    }
    msg
}

/// Build an RNDIS_MSG_QUERY message into `buf`. Returns bytes written.
///
/// Uses a buffer because the message has variable-length `info_buf` appended
/// after the fixed `rndis_query_request` fields.
pub fn build_rndis_query(req_id: u32, oid: NdisOid, info_buf: &[u8], buf: &mut [u8]) -> usize {
    let query_req_size = mem::size_of::<ffi::rndis_query_request>();
    let msg_len = RNDIS_HEADER_SIZE + query_req_size + info_buf.len();
    assert!(buf.len() >= msg_len);

    // Write header + fixed fields via struct
    let mut msg = ffi::rndis_message::default();
    msg.ndis_msg_type = RndisMessageType::Query.as_u32();
    msg.msg_len = msg_len as u32;
    unsafe {
        let q = msg.msg.query_req.as_mut();
        q.req_id = req_id;
        q.oid = oid.0;
        q.info_buflen = info_buf.len() as u32;
        q.info_buf_offset = query_req_size as u32;
    }

    // Copy struct bytes, then append info_buf
    let struct_bytes = rndis_message_as_bytes(&msg);
    let copy_len = (RNDIS_HEADER_SIZE + query_req_size).min(struct_bytes.len());
    buf[..copy_len].copy_from_slice(&struct_bytes[..copy_len]);
    if !info_buf.is_empty() {
        buf[copy_len..copy_len + info_buf.len()].copy_from_slice(info_buf);
    }
    msg_len
}

/// Build an RNDIS_MSG_SET message into `buf`. Returns bytes written.
///
/// Uses a buffer because the message has variable-length `info_buf` appended
/// after the fixed `rndis_set_request` fields.
pub fn build_rndis_set(req_id: u32, oid: NdisOid, info_buf: &[u8], buf: &mut [u8]) -> usize {
    // rndis_set_request has a flexible array member, so use the fixed field size
    let set_req_fixed_size = 20; // req_id + oid + info_buflen + info_buf_offset + dev_vc_handle
    let msg_len = RNDIS_HEADER_SIZE + set_req_fixed_size + info_buf.len();
    assert!(buf.len() >= msg_len);

    let mut msg = ffi::rndis_message::default();
    msg.ndis_msg_type = RndisMessageType::Set.as_u32();
    msg.msg_len = msg_len as u32;
    unsafe {
        let s = msg.msg.set_req.as_mut();
        s.req_id = req_id;
        s.oid = oid.0;
        s.info_buflen = info_buf.len() as u32;
        s.info_buf_offset = set_req_fixed_size as u32;
    }

    let struct_bytes = rndis_message_as_bytes(&msg);
    let copy_len = (RNDIS_HEADER_SIZE + set_req_fixed_size).min(struct_bytes.len());
    buf[..copy_len].copy_from_slice(&struct_bytes[..copy_len]);
    if !info_buf.is_empty() {
        buf[copy_len..copy_len + info_buf.len()].copy_from_slice(info_buf);
    }
    msg_len
}

/// Build an RNDIS_MSG_KEEPALIVE message.
pub fn build_rndis_keepalive(req_id: u32) -> ffi::rndis_message {
    let mut msg = ffi::rndis_message::default();
    msg.ndis_msg_type = RndisMessageType::KeepAlive.as_u32();
    msg.msg_len = (RNDIS_HEADER_SIZE + mem::size_of::<ffi::rndis_keepalive_request>()) as u32;
    unsafe {
        msg.msg.keep_alive_req.as_mut().req_id = req_id;
    }
    msg
}
