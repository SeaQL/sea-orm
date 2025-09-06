#[cfg_attr(
    any(
        feature = "runtime-async-std",
        feature = "runtime-async-std-native-tls",
        feature = "runtime-async-std-rustls"
    ),
    async_std::main
)]
#[cfg_attr(
    any(
        feature = "runtime-tokio",
        feature = "runtime-tokio-native-tls",
        feature = "runtime-tokio-rustls",
        // TODO: Remove this if the async-std is not the default runtime
        not(feature = "runtime-async-std-native-tls")
    ),
    tokio::main
)]
async fn main() {
    sea_orm_cli::main().await
}
