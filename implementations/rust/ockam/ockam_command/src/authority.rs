use crate::util::{embedded_node, multiaddr_to_route};
use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use minicbor::Decoder;
use ockam::identity::{Identity, IdentityVault};
use ockam::{Context, TcpTransport, identity::TrustIdentifierPolicy};
use ockam::authenticated_storage::InMemoryStorage;
use ockam_api::{Request, Response, Status};
use ockam_api::authority::types::{CredentialRequest, Oauth2, Signature, Signed};
use ockam_vault::KeyId;
use crate::old::identity::load_or_create_identity;
use ockam_multiaddr::MultiAddr;

#[derive(Clone, Debug, Args)]
pub struct AuthorityCommand {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Oauth2 {
        /// Address to connect to.
        #[clap(long)]
        addr: MultiAddr,
        
        /// Their identifier.
        #[clap(long)]
        id: String,
        
        /// Access token.
        #[clap(long)]
        token: String
    }
}

impl AuthorityCommand {
    pub fn run(c: AuthorityCommand) {
        embedded_node(run_impl, c.cmd)
    }
}

async fn run_impl(ctx: Context, cmd: Command) -> Result<()> {
    TcpTransport::create(&ctx).await?;
    let this = load_or_create_identity(&ctx, false).await?;
    let kid  = this.get_root_secret_key().await?;
    let store = InMemoryStorage::new();
    match &cmd {
        Command::Oauth2 { addr, id, token } => {
            let route = multiaddr_to_route(addr)
                .ok_or_else(|| anyhow!("failed to parse address: {addr}"))?;
            let policy = TrustIdentifierPolicy::new(id.as_str().try_into()?);
            let channel = this.create_secure_channel(route, policy, &store).await?;
            let req = oauth2_request(&this, &kid, token.as_str()).await?;
            let vec: Vec<u8> = ctx.send_and_receive(channel, req).await?;
            let mut dec = Decoder::new(&vec);
            let res: Response = dec.decode()?;
            if res.status() == Some(Status::Ok) {
                let a: Signed = dec.decode()?;
                dbg!(a);
            } else {
                return Err(anyhow!("todo"))
            }
        }
    }
    Ok(())
}

async fn oauth2_request<V>(this: &Identity<V>, kid: &KeyId, token: &str) -> Result<Vec<u8>>
where
    V: IdentityVault
{
    let dat = minicbor::to_vec(Oauth2::new(token))?;
    let sig = this.vault().sign(&kid, &dat).await?;
    let req = Request::post("/sign")
        .body(CredentialRequest::Oauth2 {
            data: &dat,
            signature: Signature::new(kid, sig.as_ref())
        })
        .to_vec()?;
    Ok(req)
}
