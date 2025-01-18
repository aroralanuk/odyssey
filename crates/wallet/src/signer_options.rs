use alloy_primitives::ChainId;
use alloy_primitives::B256;
use async_trait::async_trait;
use alloy::consensus::SignableTransaction;
use alloy::hex;
use alloy::network::TxSigner;
use alloy::signers::Result;
use alloy::signers::Signature;
use alloy_primitives::Address;
use aws_config::Region;
use clap::Parser;

use aws_config::BehaviorVersion;
// use aws_sdk_kms::Client;

use alloy_signer_aws::{AwsSigner, AwsSignerError};
use alloy_signer_local::PrivateKeySigner;


#[derive(Debug, Parser, Clone)]
pub struct SignerOptions {
    #[clap(long, conflicts_with_all=["aws_kms_key_id", "gcp_kms_key_ref"])]
    pub secret_key: Option<String>,

    #[clap(long, conflicts_with_all=["secret_key", "gcp_kms_key_ref"])]
    pub aws_kms_key_id: Option<String>,

    #[clap(long, conflicts_with_all=["secret_key", "aws_kms_key_id"])]
    pub gcp_kms_key_ref: Option<String>,
}


#[derive(Debug)]
pub enum OdysseySigner {
    Local(PrivateKeySigner),
    Aws(AwsSigner),
    // Gcp(GcpKmsSigner),
}

#[derive(thiserror::Error, Debug)]
pub enum OdysseySignerError {
    #[error("AWS Signer Error: {0}")]
    Aws(#[from] AwsSignerError),

    // #[error("GCP Signer Error: {0}")]
    // Gcp(#[from] GcpSignerError),

    #[error("Invalid Private Key: {0}")]
    Local(#[from] alloy_signer_local::LocalSignerError),

    #[error("hex parse error: {0}")]
    Hex(#[from] hex::FromHexError),
}

impl OdysseySigner {
    pub async fn load(chain_id: Option<ChainId>, signer_options: SignerOptions) -> std::result::Result<Self, OdysseySignerError> {
        // TODO: Implement this
        // if signer_options = Local, create a Local signer
        // if signer_options = Aws, create an Aws signer
        // if signer_options = Gcp, create a Gcp signer
        // if signer_options = None, return an error

        match (
            signer_options.secret_key,
            signer_options.aws_kms_key_id,
            signer_options.gcp_kms_key_ref,
        ) {
            (Some(secret_key), None, None) => {
                let signer = PrivateKeySigner::from_slice(secret_key.as_bytes())
                .map_err(|e| OdysseySignerError::Local(e.into()))?;
                Ok(OdysseySigner::Local(signer))
            }
            (None, Some(aws_kms_key_id), None) => {
                let config = aws_config::defaults(BehaviorVersion::latest())
                    // .region(Region::new("us-east-2"))
                    .load().await;


                let signer = AwsSigner::new(aws_sdk_kms::Client::new(&config), aws_kms_key_id, chain_id).await.map_err(|e| OdysseySignerError::Aws(e.into()))?;
                Ok(OdysseySigner::Aws(signer))
            }
            _ => Err(OdysseySignerError::Local(alloy_signer_local::LocalSignerError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, "No signer options provided")))),
        }

    }
}

#[async_trait]
impl TxSigner<Signature> for OdysseySigner {
    fn address(&self) -> Address {
        match self {
            OdysseySigner::Local(signer) => signer.address(),
            OdysseySigner::Aws(signer) => signer.address(),
            // OdysseySigner::Gcp(signer) => signer.address(),
        }
    }

    async fn sign_transaction(
        &self,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> Result<Signature> {
        match self {
            OdysseySigner::Local(signer) => signer.sign_transaction(tx).await,
            OdysseySigner::Aws(signer) => signer.sign_transaction(tx).await,
            // OdysseySigner::Gcp(signer) => signer.sign_transaction(tx).await,
        }
    }
}

#[async_trait]
impl alloy::signers::Signer<Signature> for OdysseySigner {
    async fn sign_hash(&self, hash: &B256) -> Result<Signature> {
        match self {
            OdysseySigner::Local(signer) => signer.sign_hash(hash).await,
            OdysseySigner::Aws(signer) => signer.sign_hash(hash).await,
            // OdysseySigner::Gcp(signer) => signer.sign_hash(hash).await,
        }
    }

    fn address(&self) -> Address {
        match self {
            OdysseySigner::Local(signer) => signer.address(),
            OdysseySigner::Aws(signer) => alloy::signers::Signer::address(signer),
            // OdysseySigner::Gcp(signer) => signer.address(),
        }
    }

    fn chain_id(&self) -> Option<ChainId> {
        match self {
            OdysseySigner::Local(signer) => signer.chain_id(),
            OdysseySigner::Aws(signer) => alloy::signers::Signer::chain_id(signer),
            // OdysseySigner::Gcp(signer) => signer.chain_id(),
        }
    }

    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        match self {
            OdysseySigner::Local(signer) => signer.set_chain_id(chain_id),
            OdysseySigner::Aws(signer) => alloy::signers::Signer::set_chain_id(signer, chain_id),
            // OdysseySigner::Gcp(signer) => signer.set_chain_id(chain_id),
        }
    }
}


