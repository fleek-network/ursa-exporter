use std::{
    pin::Pin,
    task::{Context, Poll},
};

use libp2p::core::{
    multiaddr::{Multiaddr, Protocol},
    transport::{ListenerId, TransportError, TransportEvent},
    Transport,
};
use log::debug;

// Wrapper around a libp2p `Transport` dropping all dial requests to non-global
// IP addresses.
#[derive(Debug, Clone, Default)]
pub struct GlobalIpOnly<T> {
    inner: T,
}

impl<T> GlobalIpOnly<T> {
    pub fn new(transport: T) -> Self {
        GlobalIpOnly { inner: transport }
    }
}

impl<T: Transport + Unpin> Transport for GlobalIpOnly<T> {
    type Output = <T as Transport>::Output;
    type Error = <T as Transport>::Error;
    type ListenerUpgrade = <T as Transport>::ListenerUpgrade;
    type Dial = <T as Transport>::Dial;

    fn listen_on(&mut self, addr: Multiaddr) -> Result<ListenerId, TransportError<Self::Error>> {
        self.inner.listen_on(addr)
    }

    fn remove_listener(&mut self, id: ListenerId) -> bool {
        self.inner.remove_listener(id)
    }

    fn dial(&mut self, addr: Multiaddr) -> Result<Self::Dial, TransportError<Self::Error>> {
        match addr.iter().next() {
            Some(Protocol::Ip4(a)) => {
                if a.is_global() {
                    self.inner.dial(addr)
                } else {
                    debug!("Not dialing non global IP address {:?}.", a);
                    Err(TransportError::MultiaddrNotSupported(addr))
                }
            }
            Some(Protocol::Ip6(a)) => {
                if a.is_global() {
                    self.inner.dial(addr)
                } else {
                    debug!("Not dialing non global IP address {:?}.", a);
                    Err(TransportError::MultiaddrNotSupported(addr))
                }
            }
            _ => {
                debug!("Not dialing unsupported Multiaddress {:?}.", addr);
                Err(TransportError::MultiaddrNotSupported(addr))
            }
        }
    }

    fn dial_as_listener(
        &mut self,
        addr: Multiaddr,
    ) -> Result<Self::Dial, TransportError<Self::Error>> {
        self.inner.dial_as_listener(addr)
    }

    fn address_translation(&self, listen: &Multiaddr, observed: &Multiaddr) -> Option<Multiaddr> {
        self.inner.address_translation(listen, observed)
    }

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<TransportEvent<Self::ListenerUpgrade, Self::Error>> {
        Pin::new(&mut self.inner).poll(cx)
    }
}
