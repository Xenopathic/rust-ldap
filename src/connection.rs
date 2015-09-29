
use std::io::prelude::*;
use std::io::{self, Cursor};
use std::net::{TcpStream, ToSocketAddrs};

use ber::{Tag, Type, Class, Payload};
use search::{Entry, Attribute, Scope, derefAlias};
use tag::LDAPTag;
use err;


pub struct LDAPConnection
{
    tcp_stream: TcpStream,
    tcp_buffer: [u8; 1024],
    message_id: u8,
}

impl LDAPConnection
{
    pub fn new<A: ToSocketAddrs>(address: A) -> Result<Self, err::Error>
    {
        let tcp_stream = try!(TcpStream::connect(address));
        Ok(LDAPConnection
        {
            tcp_stream: tcp_stream,
            tcp_buffer: [0; 1024],
            message_id: 0,
        })
    }

    /// Syncronously send a tag to the server
    fn send_tag(&mut self, operation: Tag) -> io::Result<()>
    {
        let message_id = Tag::new(
            Class::Universal(Type::Integer),
            Payload::Primitive(vec![self.message_id]),
        );

        let ldap_message = Tag::new(
            Class::Universal(Type::Sequence),
            Payload::Constructed(vec![message_id, operation]),
        );

        // Reset this every bind operation
        self.message_id += 1;

        try!(ldap_message.write(&mut self.tcp_stream));

        Ok(())
    }

    fn recv_tag(&mut self) -> Result<Tag, err::Error>
    {
        let result = try!(Tag::read(&mut self.tcp_stream));

        let mut tags = result.into_payload().into_inner_constructed().unwrap();

        Ok(tags.remove(1))
    }

    // fn try_read_tag(&mut self) -> Option<Tag>

    pub fn simple_bind(&mut self, username: String, password: String) -> Result<(), err::Error>
    {
        let version = Tag::new(
            Class::Universal(Type::Integer),
            Payload::Primitive(vec![0x3]));
        let name = username.into_tag();
        let authentication = password.into_tag();

        let bind_request = Tag::new(
            Class::Application(0),
            Payload::Constructed(vec![version, name, authentication])
        );

        try!(self.send_tag(bind_request));
        let response = try!(self.recv_tag());

        if response.is_class(Class::Application(1))
        {
            let en = response.into_payload().into_inner_constructed().unwrap().remove(0);
            return match en.into_payload()
            {
                Payload::Constructed(_) => Err(err::Error::new(err::Kind::other, None)),
                Payload::Primitive(ref t) => Ok(()),
            }
        }

        Ok(())
    }

    pub fn search(&mut self,
                  base: String,
                  scope: Scope,
                  alias: derefAlias,
                  size_limit: i32,
                  time_limit: i32,
                  types_only: bool,
                  filters: Tag, // TODO: Figure something out...
                  attributes: Vec<String>
           ) -> Result<Vec<Entry>, err::Error>
    {
        let search_base = base.into_tag();
        let scope = scope.into_tag();
        let alias = alias.into_tag();
        let size_limit = size_limit.into_tag();
        let time_limit = time_limit.into_tag();
        let types_only = types_only.into_tag();
        // let filters = filters.into_tag();
        let attributes = attributes.into_tag();

        let search_request = Tag::new(Class::Application(3),
                 Payload::Constructed(vec![
                     search_base,
                     scope,
                     alias,
                     size_limit,
                     time_limit,
                     types_only,
                     filters,
                     attributes
                 ]));

        try!(self.send_tag(search_request));

        loop
        {
            let response = try!(self.recv_tag());
            // Response is either Application(5) (Search Done) or Application(4) (Search Entry)
            match response.class
            {
                Class::Application(5) => break,
                Class::Application(4) =>
                {
                    // Parse search entry and add to list?
                },
                _ => return Err(err::Error::new(err::Kind::other, None)),
            }
        }

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind() {
        let mut conn = LDAPConnection::new(("127.0.0.1", 3890)).unwrap();

        conn.simple_bind("cn=root".to_string(), "secret".to_string()).unwrap();

        assert!(false)
    }
}