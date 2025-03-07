use std::sync::Arc;
use reqwest::Client;
use std::error::Error;
use std::fmt;
use tokio::sync::mpsc::{self, Sender, Receiver};

#[derive(Debug)]
pub enum NotificationError {
    SendError(String),
    NetworkError(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationError::SendError(msg) => write!(f, "Failed to send notification: {}", msg),
            NotificationError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl Error for NotificationError {}

pub struct NotificationManager {
    http_client: Client,
    email_endpoint: String,
    sms_endpoint: String,
    event_sender: Option<Sender<NotificationEvent>>,
}

#[derive(Debug, Clone)]
pub enum NotificationEvent {
    Email { subject: String, body: String, recipient: String },
    SMS { message: String, recipient: String },
    Push { title: String, message: String, device_id: String },
}

impl NotificationManager {
    pub fn new(email_endpoint: String, sms_endpoint: String) -> Self {
        let (sender, receiver) = mpsc::channel(100);
        
        let manager = Self {
            http_client: Client::new(),
            email_endpoint,
            sms_endpoint,
            event_sender: Some(sender),
        };
        
        // Start background listener
        manager.start_event_listener(receiver);
        
        manager
    }
    
    fn start_event_listener(&self, mut receiver: Receiver<NotificationEvent>) {
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    NotificationEvent::Email { subject, body, recipient } => {
                        println!("Email to {}: {}", recipient, subject);
                        // In a real implementation, this would send the email
                    }
                    NotificationEvent::SMS { message, recipient } => {
                        println!("SMS to {}: {}", recipient, message);
                        // In a real implementation, this would send the SMS
                    }
                    NotificationEvent::Push { title, message, device_id } => {
                        println!("Push to {}: {}", device_id, title);
                        // In a real implementation, this would send the push notification
                    }
                }
            }
        });
    }
    
    pub async fn send_email(&self, subject: &str, body: &str) -> Result<(), NotificationError> {
        // In a real implementation, this would send a HTTP request to the email service
        // For testing, we just simulate success
        Ok(())
    }
    
    pub async fn send_sms(&self, message: &str) -> Result<(), NotificationError> {
        // In a real implementation, this would send a HTTP request to the SMS service
        // For testing, we just simulate success
        Ok(())
    }
    
    pub async fn send_notification(&self, subject: &str, body: &str) -> Result<(), NotificationError> {
        // Try to send email first
        match self.send_email(subject, body).await {
            Ok(_) => return Ok(()),
            Err(_) => {
                // If email fails, try SMS
                println!("Email failed, falling back to SMS");
                self.send_sms(&format!("{}: {}", subject, body)).await?;
            }
        }
        
        Ok(())
    }
}