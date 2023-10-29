use tokio::io::{AsyncRead, AsyncWrite};

// Utils function to read and write data asynchronously from and to sockets
pub async fn read_write<R, W>(mut reader: R, mut writer: W)
where
    R: AsyncRead + Unpin + Sized + Send + 'static,
    W: AsyncWrite + Unpin + Sized + Send + 'static,
{
    // Passing static references to sockets, not moving them so the connection closes when the program terminates only
    // OR if one of the connection is closed at some point (e.g. with ctrl+c in our CLI app case)
    let client_read = tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut tokio::io::stdout())
            .await
            .unwrap();
    });

    let client_write = tokio::spawn(async move {
        tokio::io::copy(&mut tokio::io::stdin(), &mut writer)
            .await
            .unwrap();
    });

    // Runs both work concurrently and stops woring when one of the workers is stopped
    // Select is like "match", but for async code
    tokio::select! {
        _ = client_read => {},
        _ = client_write => {}
    };
}