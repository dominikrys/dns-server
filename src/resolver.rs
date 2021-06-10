use crate::dns_packet::{BufferIO, Packet, Query, QueryType, ReturnCode};
use crate::packet_buffer::PacketBuffer;

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Resolver {
    socket: UdpSocket,
}

impl Resolver {
    pub fn new(bind_addr: &str, port: u16) -> Result<Self> {
        let socket = UdpSocket::bind((bind_addr, port))?;

        Ok(Resolver { socket })
    }

    fn send_packet(
        &self,
        mut packet: Packet,
        socket: &UdpSocket,
        dst_socket: &(IpAddr, u16),
    ) -> Result<()> {
        let mut buf = PacketBuffer::new();
        packet.write_to_buffer(&mut buf)?;
        socket.send_to(buf.get_range(0, buf.pos())?, dst_socket)?;

        Ok(())
    }

    fn receive_packet(&self, socket: &UdpSocket) -> Result<(Packet, SocketAddr)> {
        let mut raw_buf: [u8; 512] = [0; 512];
        let (_, src_socket) = socket.recv_from(&mut raw_buf)?;
        let mut buf = PacketBuffer::from_u8_array(raw_buf);
        let packet = Packet::from_buffer(&mut buf)?;

        Ok((packet, src_socket))
    }

    pub fn handle_query(&self) -> Result<()> {
        let (req_packet, src_socket) = self.receive_packet(&self.socket)?;

        let mut res_packet = Packet::new();
        res_packet.header.id = req_packet.header.id;
        res_packet.header.recursion_desired = true;
        res_packet.header.recursion_available = true;
        res_packet.header.response = true;

        if req_packet.queries.is_empty() {
            res_packet.header.return_code = ReturnCode::FORMERR;
        }

        for query in req_packet.queries.iter() {
            println!("Received query: {:?}", query);

            if let Ok(result) = self.recursive_lookup(&query.qname, query.qtype) {
                res_packet.queries.push(query.clone());
                res_packet.header.return_code = result.header.return_code;

                for rec in result.answer_records {
                    println!("Answer record: {:?}", rec);
                    res_packet.answer_records.push(rec);
                }
                for rec in result.authoritative_records {
                    println!("Authority record: {:?}", rec);
                    res_packet.authoritative_records.push(rec);
                }
                for rec in result.additional_records {
                    println!("Additional record: {:?}", rec);
                    res_packet.additional_records.push(rec);
                }
            } else {
                res_packet.header.return_code = ReturnCode::SERVFAIL;
            }
        }

        self.send_packet(
            res_packet,
            &self.socket,
            &(src_socket.ip(), src_socket.port()),
        )?;

        Ok(())
    }

    fn lookup(&self, qname: &str, qtype: QueryType, server: (IpAddr, u16)) -> Result<Packet> {
        let lookup_socket = UdpSocket::bind(("0.0.0.0", 43210))?;

        let mut req_packet = Packet::new();
        req_packet.header.id = 1234;
        req_packet.header.queries_total = 1;
        req_packet.header.recursion_desired = true;
        req_packet
            .queries
            .push(Query::new(qname.to_string(), qtype));

        self.send_packet(req_packet, &lookup_socket, &server)?;

        let (res_packet, _) = self.receive_packet(&lookup_socket)?;
        Ok(res_packet)
    }

    fn recursive_lookup(&self, qname: &str, qtype: QueryType) -> Result<Packet> {
        let a_root_servers_net_ip = "198.41.0.4";
        let mut ns = IpAddr::V4(a_root_servers_net_ip.parse::<Ipv4Addr>().unwrap());

        loop {
            println!("Performing lookup of {:?} {} with ns {}", qtype, qname, ns);

            let server = (ns, 53);
            let response = self.lookup(qname, qtype, server)?;

            if (!response.answer_records.is_empty()
                && response.header.return_code == ReturnCode::NOERROR)
                || response.header.return_code == ReturnCode::NXDOMAIN
            {
                return Ok(response);
            }

            if let Some(&new_ns) = response.get_ns_from_additional_records(qname).last() {
                ns = IpAddr::V4(*new_ns);
                continue;
            }

            let new_ns_host = match response.get_ns_hosts(qname).last() {
                Some(&host) => host,
                None => return Ok(response),
            };

            let recursive_response = self.recursive_lookup(&new_ns_host, qtype)?;

            ns = match recursive_response.get_answer_a_records().last() {
                Some(&new_ns) => IpAddr::V4(*new_ns),
                None => return Ok(response),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recursive_lookup() -> Result<()> {
        /* Arrange */
        let localhost_str = "127.0.0.1";
        let localhost_addr = localhost_str.parse::<Ipv4Addr>()?;

        // Resolver
        let resolver_port = 2053;
        let resolver = Resolver::new(localhost_str, resolver_port)?;

        // Client
        let socket = UdpSocket::bind((localhost_str, 2054))?;

        // Query Packet
        let qname = "google.com";
        let qtype = QueryType::A;

        let mut packet = Packet::new();
        packet.header.id = 123;
        packet.header.queries_total = 1;
        packet.header.recursion_desired = true;
        packet.queries.push(Query::new(qname.to_string(), qtype));

        let mut req_buffer = PacketBuffer::new();
        packet.write_to_buffer(&mut req_buffer)?;

        /* Act */
        socket.send_to(
            req_buffer.get_range(0, req_buffer.pos())?,
            (localhost_addr, resolver_port),
        )?;

        resolver.handle_query()?;

        let mut raw_buf: [u8; 512] = [0; 512];
        socket.recv_from(&mut raw_buf)?;
        let mut res_buf = PacketBuffer::from_u8_array(raw_buf);
        let res_packet = Packet::from_buffer(&mut res_buf)?;

        /* Assert */
        assert!(res_packet.queries.len() > 0);
        assert_eq!(res_packet.queries[0].qname, "google.com");

        assert!(res_packet.answer_records.len() > 0);
        match res_packet.answer_records[0] {
            crate::dns_packet::ResourceRecord::A { ref domain, .. } => {
                assert_eq!("google.com", domain);
            }
            _ => panic!(),
        }

        Ok(())
    }
}
