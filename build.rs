#[cfg(feature = "voice")]
compile_error!("The `voice` feature cannot currently be enabled.");

#[cfg(all(any(feature = "http", feature = "gateway"),
    not(feature = "native_tls_backend")))]
compile_error!("You have the `http` or `gateway` feature enabled, \
    the native_tls_backend` feature must be
    selected to let Serenity use `http` or `gateway`.\n\
    - `native_tls_backend` uses SChannel on Windows, Secure Transport on macOS, \
    and OpenSSL on other platforms.\n");

fn main() {}
