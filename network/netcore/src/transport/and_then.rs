// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::transport::{ConnectionOrigin, Transport};
use cedra_types::{network_address::NetworkAddress, PeerId};
use futures::{future::Future, stream::Stream};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// An [`AndThen`] is a transport which applies a closure (F) to all connections created by the
/// underlying transport.
pub struct AndThen<T, F> {
    transport: T,
    function: F,
}

impl<T, F> AndThen<T, F> {
    pub(crate) fn new(transport: T, function: F) -> Self {
        Self {
            transport,
            function,
        }
    }
}

impl<T, F, Fut, O> Transport for AndThen<T, F>
where
    T: Transport,
    F: (FnOnce(T::Output, NetworkAddress, ConnectionOrigin) -> Fut) + Send + Unpin + Clone,
    // Pin the error types to be the same for now
    // TODO don't require the error types to be the same
    Fut: Future<Output = Result<O, T::Error>> + Send,
{
    type Error = T::Error;
    type Inbound = AndThenFuture<T::Inbound, Fut, F>;
    type Listener = AndThenStream<T::Listener, F>;
    type Outbound = AndThenFuture<T::Outbound, Fut, F>;
    type Output = O;

    fn listen_on(
        &self,
        addr: NetworkAddress,
    ) -> Result<(Self::Listener, NetworkAddress), Self::Error> {
        let (listener, addr) = self.transport.listen_on(addr)?;
        let listener = AndThenStream::new(listener, self.function.clone());

        Ok((listener, addr))
    }

    fn dial(&self, peer_id: PeerId, addr: NetworkAddress) -> Result<Self::Outbound, Self::Error> {
        let fut = self.transport.dial(peer_id, addr.clone())?;
        let origin = ConnectionOrigin::Outbound;
        let f = self.function.clone();

        Ok(AndThenFuture::new(fut, f, addr, origin))
    }
}

/// Listener stream returned by [listen_on](Transport::listen_on) on an AndThen transport.
#[pin_project]
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct AndThenStream<St, F> {
    #[pin]
    stream: St,
    f: F,
}

impl<St, Fut1, O1, Fut2, O2, E, F> AndThenStream<St, F>
where
    St: Stream<Item = Result<(Fut1, NetworkAddress), E>>,
    Fut1: Future<Output = Result<O1, E>>,
    Fut2: Future<Output = Result<O2, E>>,
    F: FnOnce(O1, NetworkAddress, ConnectionOrigin) -> Fut2 + Clone,
    E: ::std::error::Error,
{
    fn new(stream: St, f: F) -> Self {
        Self { stream, f }
    }
}

impl<St, Fut1, O1, Fut2, O2, E, F> Stream for AndThenStream<St, F>
where
    St: Stream<Item = Result<(Fut1, NetworkAddress), E>>,
    Fut1: Future<Output = Result<O1, E>>,
    Fut2: Future<Output = Result<O2, E>>,
    F: FnOnce(O1, NetworkAddress, ConnectionOrigin) -> Fut2 + Clone,
    E: ::std::error::Error,
{
    type Item = Result<(AndThenFuture<Fut1, Fut2, F>, NetworkAddress), E>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        match self.as_mut().project().stream.poll_next(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(Some(Ok((fut1, addr)))) => Poll::Ready(Some(Ok((
                AndThenFuture::new(
                    fut1,
                    self.f.clone(),
                    addr.clone(),
                    ConnectionOrigin::Inbound,
                ),
                addr,
            )))),
        }
    }
}

#[pin_project(project = AndThenChainProj)]
#[derive(Debug)]
enum AndThenChain<Fut1, Fut2, F> {
    First(#[pin] Fut1, Option<(F, NetworkAddress, ConnectionOrigin)>),
    Second(#[pin] Fut2),
    Empty,
}

/// Future generated by the [`AndThen`] transport.
///
/// Takes a future (Fut1) generated from an underlying transport, runs it to completion and applies
/// a closure (F) to the result to create another future (Fut2) which is then run to completion.
#[pin_project]
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct AndThenFuture<Fut1, Fut2, F> {
    #[pin]
    chain: AndThenChain<Fut1, Fut2, F>,
}

impl<Fut1, O1, Fut2, O2, E, F> AndThenFuture<Fut1, Fut2, F>
where
    Fut1: Future<Output = Result<O1, E>>,
    Fut2: Future<Output = Result<O2, E>>,
    F: FnOnce(O1, NetworkAddress, ConnectionOrigin) -> Fut2,
    E: ::std::error::Error,
{
    fn new(fut1: Fut1, f: F, addr: NetworkAddress, origin: ConnectionOrigin) -> Self {
        Self {
            chain: AndThenChain::First(fut1, Some((f, addr, origin))),
        }
    }
}

// Inspired by: https://github.com/rust-lang-nursery/futures-rs/blob/master/futures-util/src/future/chain.rs
impl<Fut1, O1, Fut2, O2, E, F> Future for AndThenFuture<Fut1, Fut2, F>
where
    Fut1: Future<Output = Result<O1, E>>,
    Fut2: Future<Output = Result<O2, E>>,
    F: FnOnce(O1, NetworkAddress, ConnectionOrigin) -> Fut2,
    E: ::std::error::Error,
{
    type Output = Result<O2, E>;

    fn poll(self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let mut this = self.project();
        loop {
            let (output, (f, addr, origin)) = match this.chain.as_mut().project() {
                // Step 1: Drive Fut1 to completion
                AndThenChainProj::First(fut1, data) => match fut1.poll(context) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Ready(Ok(output)) => (output, data.take().expect("must be initialized")),
                },
                // Step 4: Drive Fut2 to completion
                AndThenChainProj::Second(fut2) => return fut2.poll(context),
                AndThenChainProj::Empty => unreachable!(),
            };

            // Step 2: Ensure that Fut1 is dropped
            this.chain.set(AndThenChain::Empty);
            // Step 3: Run F on the output of Fut1 to create Fut2
            let fut2 = f(output, addr, origin);
            this.chain.set(AndThenChain::Second(fut2));
        }
    }
}
