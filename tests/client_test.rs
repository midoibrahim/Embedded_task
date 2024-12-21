use embedded_recruitment_task::{
    message::{client_message, server_message, AddRequest, EchoMessage},
    server::Server,
};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use env_logger;
mod client;

fn setup_server_thread(server: Arc<Server>) -> JoinHandle<()> {
    thread::spawn(move || {
        server.run().expect("Server encountered an error");
    })
}

pub fn create_server(addr: &str) -> Arc<Server> {
    Server::new(addr).expect("Failed to create server")
}

#[test]
fn test_client_connection() {
    let _ = env_logger::builder().is_test(true).try_init();
    // Set up the server in a separate thread
    let server = create_server("localhost:8080");
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );

}

#[test]
fn test_client_echo_message() {
    let _ = env_logger::builder().is_test(true).try_init();
    // Set up the server in a separate thread
    let server = create_server("localhost:8080");
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the echoed message
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for EchoMessage"
    );

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(
                echo.content, echo_message.content,
                "Echoed message content does not match"
            );
        }
        _ => panic!("Expected EchoMessage, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );

}

#[test]
fn test_multiple_echo_messages() {
    let _ = env_logger::builder().is_test(true).try_init();
    // Set up the server in a separate thread
    let server = create_server("localhost:8080");
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message);

        // Send the message to the server
        assert!(client.send(message).is_ok(), "Failed to send message");

        // Receive the echoed message
        let response = client.receive();
        assert!(
            response.is_ok(),
            "Failed to receive response for EchoMessage"
        );

        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(
                    echo.content, message_content,
                    "Echoed message content does not match"
                );
            }
            _ => panic!("Expected EchoMessage, but received a different message"),
        }
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
fn test_multiple_clients() {
    let _ = env_logger::builder().is_test(true).try_init();
    // Set up the server in a separate thread
    let server = create_server("localhost:8080");
    let handle: JoinHandle<()> = setup_server_thread(server.clone());

    // Create and connect multiple clients
    let mut clients = vec![
        client::Client::new("localhost", 8080, 1000),
        client::Client::new("localhost", 8080, 1000),
        client::Client::new("localhost", 8080, 1000),
    ];

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages for each client
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        for client in clients.iter_mut() {
            // Send the message to the server
            assert!(
                client.send(message.clone()).is_ok(),
                "Failed to send message"
            );

            // Receive the echoed message
            let response = client.receive();
            assert!(
                response.is_ok(),
                "Failed to receive response for EchoMessage"
            );

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content, message_content,
                        "Echoed message content does not match"
                    );
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            }
        }
    }

    // Disconnect the clients
    for client in clients.iter_mut() {
        assert!(
            client.disconnect().is_ok(),
            "Failed to disconnect from the server"
        );
    }

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
fn test_client_add_request() {
    let _ = env_logger::builder().is_test(true).try_init();
    // Set up the server in a separate thread
    let server = create_server("localhost:8080");
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 20;
    let message = client_message::Message::AddRequest(add_request.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the response
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for AddRequest"
    );

    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(
                add_response.result,
                add_request.a + add_request.b,
                "AddResponse result does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

#[test]
fn test_multiple_clients_with_different_message_types() {
    let _ = env_logger::builder().is_test(true).try_init();
    // Set up the server in a separate thread
    let server1 = create_server("localhost:2050");
    let server2 = create_server("localhost:2010");
    let handle1: JoinHandle<()> = setup_server_thread(server1.clone());
    let handle2: JoinHandle<()> = setup_server_thread(server2.clone());

    // Create and connect multiple clients
    let mut clients = vec![
        client::Client::new("localhost", 2050, 1000),
        client::Client::new("localhost", 2010, 1000),
        client::Client::new("localhost", 2010, 1000),
    ];

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    // Prepare multiple messages
    let echo_messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    let add_requests = vec![
        (1, 2),
        (10, 20),
        (100, 200),
    ];

    // Send and receive EchoMessages for each client
    for message_content in echo_messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        for client in clients.iter_mut() {
            // Send the message to the server
            assert!(
                client.send(message.clone()).is_ok(),
                "Failed to send EchoMessage"
            );

            // Receive the echoed message
            let response = client.receive();
            assert!(
                response.is_ok(),
                "Failed to receive response for EchoMessage"
            );

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content, message_content,
                        "Echoed message content does not match"
                    );
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            }
        }
    }

    // Send and receive AddRequests for each client
    for (a, b) in add_requests {
        let mut add_request = AddRequest::default();
        add_request.a = a;
        add_request.b = b;
        let message = client_message::Message::AddRequest(add_request.clone());

        for client in clients.iter_mut() {
            // Send the message to the server
            assert!(
                client.send(message.clone()).is_ok(),
                "Failed to send AddRequest"
            );

            // Receive the response
            let response = client.receive();
            assert!(
                response.is_ok(),
                "Failed to receive response for AddRequest"
            );

            match response.unwrap().message {
                Some(server_message::Message::AddResponse(add_response)) => {
                    assert_eq!(
                        add_response.result,
                        add_request.a + add_request.b,
                        "AddResponse result does not match"
                    );
                }
                _ => panic!("Expected AddResponse, but received a different message"),
            }
        }
    }

    // Disconnect the clients
    for client in clients.iter_mut() {
        assert!(
            client.disconnect().is_ok(),
            "Failed to disconnect from the server"
        );
    }

    // Stop the server and wait for thread to finish
    server1.stop();
    server2.stop();
    assert!(
        handle1.join().is_ok(),
        "Server1 thread panicked or failed to join"
    );
    assert!(
        handle2.join().is_ok(),
        "Server2 thread panicked or failed to join"
    );
}
