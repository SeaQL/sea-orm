#[cfg_attr(
    all(
        any(
            feature = "runtime-async-std",
            feature = "runtime-async-std-native-tls",
            feature = "runtime-async-std-rustls"
        ),
        not(any(
            feature = "runtime-tokio",
            feature = "runtime-tokio-native-tls",
            feature = "runtime-tokio-rustls",
        ))
    ),
    async_std::main
)]
#[cfg_attr(
    any(
        feature = "runtime-tokio",
        feature = "runtime-tokio-native-tls",
        feature = "runtime-tokio-rustls",
    ),
    tokio::main
)]
async fn main() {
    sea_orm_cli::main().await
}
