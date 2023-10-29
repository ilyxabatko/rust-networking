pub async fn client() -> Result<(), String> {
    let client = tokio::net::TcpStream::connect("localhost:2323").await.map_err(|e| format!("Error connecting to the server: {}", e))?;

    let (mut reader, mut writer) = client.into_split();

    let client_read = tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await.unwrap();
    });

    let client_write = tokio::spawn(async move {
        tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await.unwrap();
    });


    // Runs both work concurrently and stops woring when one of the workers is stopped
    // Select is like "match", but for async code
    tokio::select! {
        _ = client_read => {},
        _ = client_write => {}
    };

    Ok(())
}

pub async fn server() -> Result<(), String> {
    let listener = tokio::net::TcpListener::bind("localhost:2323").await.map_err(|e| format!("Error binding to the address: {}", e))?;

    let (handle, _) = listener.accept().await.map_err(|e| format!("Error accepting the connection: {}", e))?;

    let (mut reader, mut writer) = handle.into_split();

    let server_read = tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await.unwrap();
    });

    let server_write = tokio::spawn(async move {
        tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await.unwrap();
    });


    // Runs both work concurrently and stops woring when one of the workers is stopped
    tokio::select! {
        _ = server_read => {},
        _ = server_write => {}
    };

    Ok(())
}