// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::Result;
use clap::Parser;
use warp::Filter;

use crate::hyper_fetcher::HyperFetcher;
use crate::linux_commands::{create_certificate_request_pem, create_private_key_pem};

#[derive(Debug, Parser)]
pub struct Opts {
    #[clap(long)]
    port: u16,
    /// Directory URL of ACME server
    #[clap(long)]
    acme_server: String,
    #[clap(long)]
    email: String,
    #[clap(long)]
    domain: String,
    #[clap(long, default_value_t=String::from("acme_account_private_key.pem"))]
    acme_account_private_key_file: String,
    #[clap(long, default_value_t=String::from("privkey.pem"))]
    sxg_private_key_file: String,
    #[clap(long, default_value_t=String::from("cert.csr"))]
    sxg_cert_request_file: String,
    #[clap(long)]
    agreed_terms_of_service: String,
}

fn start_warp_server(port: u16, answer: String) -> tokio::sync::oneshot::Sender<()> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let routes =
        warp::path!(".well-known" / "acme-challenge" / String).map(move |_name| answer.to_string());
    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
            rx.await.ok();
        });
    tokio::spawn(server);
    tx
}

pub async fn main(opts: Opts) -> Result<()> {
    let acme_private_key = {
        let private_key_pem = create_private_key_pem(&opts.acme_account_private_key_file)?;
        sxg_rs::crypto::EcPrivateKey::from_sec1_pem(&private_key_pem)?
    };
    let sxg_cert_request_der = {
        create_private_key_pem(&opts.sxg_private_key_file)?;
        let cert_request_pem = create_certificate_request_pem(
            &opts.domain,
            &opts.sxg_private_key_file,
            &opts.sxg_cert_request_file,
        )?;
        sxg_rs::crypto::get_der_from_pem(&cert_request_pem, "CERTIFICATE REQUEST")?
    };
    let signer = acme_private_key.create_signer()?;
    let fetcher = HyperFetcher::new();
    let ongoing_certificate_request =
        sxg_rs::acme::create_request_and_get_challenge_answer(sxg_rs::acme::AcmeStartupParams {
            directory_url: &opts.acme_server,
            agreed_terms_of_service: &opts.agreed_terms_of_service,
            external_account_binding: None, // TODO: Add EAB to CLI params.
            email: &opts.email,
            domain: &opts.domain,
            public_key: acme_private_key.public_key,
            cert_request_der: sxg_cert_request_der,
            fetcher,
            signer,
        })
        .await?;
    let tx = start_warp_server(
        opts.port,
        ongoing_certificate_request.challenge_answer.clone(),
    );
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let certificate_pem = sxg_rs::acme::continue_challenge_validation_and_get_certificate(
        ongoing_certificate_request,
    )
    .await?;
    let _ = tx.send(());
    println!("{}", certificate_pem);
    Ok(())
}