// Copyright (C) 2026, Cloudflare, Inc.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//
//     * Redistributions in binary form must reproduce the above copyright
//       notice, this list of conditions and the following disclaimer in the
//       documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO,
// THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
// PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! No Congestion Control
//!
//! This module implements a no-op congestion control algorithm that sets the
//! congestion window to its maximum value and never reduces it. Useful for
//! testing and controlled network environments.

use std::time::Instant;

use super::rtt::RttStats;
use super::Acked;
use super::Congestion;
use super::CongestionControlOps;
use super::Sent;

pub(crate) static NONE: CongestionControlOps = CongestionControlOps {
    on_init,
    on_packet_sent,
    on_packets_acked,
    congestion_event,
    checkpoint,
    rollback,
    #[cfg(feature = "qlog")]
    state_str,
    debug_fmt,
};

fn on_init(r: &mut Congestion) {
    r.congestion_window = usize::MAX;
}

fn on_packet_sent(
    _r: &mut Congestion, _sent_bytes: usize, _bytes_in_flight: usize,
    _now: Instant,
) {
}

fn on_packets_acked(
    _r: &mut Congestion, _bytes_in_flight: usize, packets: &mut Vec<Acked>,
    _now: Instant, _rtt_stats: &RttStats,
) {
    packets.clear();
}

fn congestion_event(
    _r: &mut Congestion, _bytes_in_flight: usize, _lost_bytes: usize,
    _largest_lost_pkt: &Sent, _now: Instant,
) {
}

fn checkpoint(_r: &mut Congestion) {}

fn rollback(_r: &mut Congestion) -> bool {
    true
}

#[cfg(feature = "qlog")]
fn state_str(_r: &Congestion, _now: Instant) -> &'static str {
    "none"
}

fn debug_fmt(
    _r: &Congestion, f: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    write!(f, "none")
}

#[cfg(test)]
mod tests {
    use crate::CongestionControlAlgorithm;

    use crate::recovery::congestion::recovery::LegacyRecovery;
    use crate::recovery::congestion::test_sender::TestSender;
    use crate::recovery::RecoveryOps;

    fn test_sender() -> TestSender {
        TestSender::new(CongestionControlAlgorithm::None, false)
    }

    #[test]
    fn none_init() {
        let mut cfg = crate::Config::new(crate::PROTOCOL_VERSION).unwrap();
        cfg.set_cc_algorithm(CongestionControlAlgorithm::None);

        let r = LegacyRecovery::new(&cfg);

        assert_eq!(r.cwnd(), usize::MAX);
        assert_eq!(r.bytes_in_flight(), 0);
    }

    #[test]
    fn none_no_window_reduction_on_loss() {
        let mut sender = test_sender();
        let size = sender.max_datagram_size;

        sender.send_packet(size);
        sender.lose_n_packets(1, size, None);

        assert_eq!(sender.congestion_window, usize::MAX);
    }

    #[test]
    fn none_window_unchanged_after_ack() {
        let mut sender = test_sender();
        let size = sender.max_datagram_size;

        for _ in 0..5 {
            sender.send_packet(size);
        }

        let cwnd_before = sender.congestion_window;
        sender.ack_n_packets(3, size);

        assert_eq!(sender.congestion_window, cwnd_before);
    }

    #[test]
    fn none_from_string() {
        let algo: CongestionControlAlgorithm = "none".parse().unwrap();
        assert_eq!(algo, CongestionControlAlgorithm::None);
    }
}
