use crate::naming::core::{NamingActor, NamingCmd};
use crate::naming::model::{InstanceShortKey, ServiceKey};
use actix::prelude::*;
use bean_factory::{bean, BeanFactory, FactoryData, Inject};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

pub async fn probe_tcp(host: &str, port: u16, timeout_duration: Duration) -> anyhow::Result<()> {
    let addr = format!("{}:{}", host, port);
    timeout(timeout_duration, TcpStream::connect(&addr)).await??;
    Ok(())
}

#[derive(Debug, Clone)]
#[bean(inject)]
pub struct NetSniffing {
    pub naming_actor: Option<Addr<NamingActor>>,
    pub timeout_duration: Duration,
    pub retry_interval: Duration,
}

impl Actor for NetSniffing {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("NetSniffing started");
    }
}

impl Inject for NetSniffing {
    type Context = Context<Self>;
    fn inject(
        &mut self,
        factory_data: FactoryData,
        _factory: BeanFactory,
        _ctx: &mut Self::Context,
    ) {
        self.naming_actor = factory_data.get_actor();
    }
}

impl NetSniffing {
    pub fn new(timeout_duration: Duration, retry_interval: Duration) -> Self {
        Self {
            timeout_duration,
            retry_interval,
            naming_actor: None,
        }
    }

    async fn handle_msg(&self, msg: NetSniffingCmd) -> anyhow::Result<NetSniffingResult> {
        match msg {
            NetSniffingCmd::ProbeHost(host) => Ok(Self::probe(host, self.timeout_duration).await),
            NetSniffingCmd::ProbeServiceHost(host, service_keys) => {
                let mut success =
                    probe_tcp(host.ip.as_ref(), host.port as u16, self.timeout_duration)
                        .await
                        .is_ok();
                if !success {
                    //retry
                    tokio::time::sleep(self.retry_interval).await;
                    success = probe_tcp(host.ip.as_ref(), host.port as u16, self.timeout_duration)
                        .await
                        .is_ok();
                }
                if let Some(naming_actor) = &self.naming_actor {
                    naming_actor.do_send(NamingCmd::PerpetualHostSniffing {
                        host,
                        service_keys,
                        success,
                    })
                }
                Ok(NetSniffingResult::None)
            }
        }
    }

    async fn probe(host: InstanceShortKey, timeout_duration: Duration) -> NetSniffingResult {
        match probe_tcp(host.ip.as_ref(), host.port as u16, timeout_duration).await {
            Ok(_) => NetSniffingResult::ProbeResult(true),
            _ => NetSniffingResult::ProbeResult(false),
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "anyhow::Result<NetSniffingResult>")]
pub enum NetSniffingCmd {
    ProbeHost(InstanceShortKey),
    ProbeServiceHost(InstanceShortKey, Vec<ServiceKey>),
}

#[derive(Debug)]
pub enum NetSniffingResult {
    ProbeResult(bool),
    None,
}

impl Handler<NetSniffingCmd> for NetSniffing {
    type Result = ResponseActFuture<Self, anyhow::Result<NetSniffingResult>>;
    fn handle(&mut self, msg: NetSniffingCmd, _ctx: &mut Self::Context) -> Self::Result {
        let this = self.clone();
        let fut = async move { this.handle_msg(msg).await }
            .into_actor(self)
            .map(|result, _act, _ctx| result);
        Box::pin(fut)
    }
}
