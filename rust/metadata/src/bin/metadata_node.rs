use clap::Parser;
use log::info;
use risingwave_common::util::addr::get_host_port;
use risingwave_metadata::rpc::server::rpc_serve;

#[derive(Parser)]
struct Opts {
    // The custom log4rs config file.
    #[clap(long, default_value = "config/log4rs.yaml")]
    log4rs_config: String,

    #[clap(long, default_value = "127.0.0.1:5688")]
    host: String,
}

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    log4rs::init_file(opts.log4rs_config, Default::default()).unwrap();

    let addr = get_host_port(opts.host.as_str()).unwrap();
    info!("Starting metadata server at {}", addr);
    let (join_handle, _shutdown_send) = rpc_serve(addr).await;
    join_handle.await.unwrap();
}