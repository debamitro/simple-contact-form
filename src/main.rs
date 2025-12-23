use actix_web::{web, App, HttpServer, Responder, post, get};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::env;

use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct FormData {
    id: String,
    name: String,
    email: String,
    message: String,
}

// Wrapper to include API key in app data
struct AppState {
    api_key: String,
}

#[post("/v1/submit")]
async fn submit_form(form: web::Form<FormData>, data: web::Data<AppState>) -> impl Responder {
    // Log the form submission
    println!("Received form submission with ID: {}", form.id);
    println!("From: {} ({})", form.name, form.email);
    println!("Message: {}", form.message);

    // Send email
    let api_key = data.api_key.clone();
    match send_email(&form.name, &form.email, &form.message, &form.id, &api_key).await {
        Ok(_) => {
            "Form submitted successfully! Email sent.".to_string()
        }
        Err(e) => {
            eprintln!("Failed to send email: {}", e);
            "Form submitted, but failed to send email.".to_string()
        }
    }
}

async fn send_email(name: &str, email: &str, message: &str, id: &str, resend_api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Resend API configuration
    let from_email = "noreply@eastcoastsoft.com"; // Replace with your verified "from" email in Resend

    // Read accounts.json to find the recipient email based on ID
    let accounts_json = include_str!("../accounts.json");
    let accounts: serde_json::Value = serde_json::from_str(&accounts_json)?;
    let accounts_array = accounts.get("accounts")
        .and_then(|acc| acc.as_array())
        .ok_or_else(|| "No 'accounts' array found in JSON".to_string())?;
    let to_email = accounts_array.iter()
        .find(|entry| entry.get("id").and_then(|val| val.as_str()) == Some(id))
        .and_then(|entry| entry.get("email"))
        .and_then(|email| email.as_str())
        .ok_or_else(|| format!("No email found for ID: {}", id))?.to_string();

    // Build the email payload
    let email_body = format!(
        "New form submission:<br><br><strong>ID:</strong> {}<br><strong>Name:</strong> {}<br><strong>Email:</strong> {}<br><strong>Message:</strong> {}",
        id, name, email, message
    );
    let payload = serde_json::json!({
        "from": from_email,
        "to": [to_email],
        "subject": "New Form Submission",
        "html": email_body
    });

    // Send the email via Resend API
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", resend_api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send().await?;

    // Print the response status and body even if the request fails
    println!("Response status: {}", response.status());
    if let Ok(json_response) = response.json::<serde_json::Value>().await {
        println!("Response body: {}", json_response);
    } else {
        println!("Failed to parse response as JSON");
    }

    // Still return an error if the status is not successful
    //response.error_for_status()?;

    //println!("Email sent successfully to {}. Response: {:?}", to_email, response.json::<serde_json::Value>().await.unwrap_or_else(|_| serde_json::json!({"error": "Failed to parse JSON response"})));

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Example: Access an environment variable
    let api_key = env::var("MAILERSEND_API_KEY").expect("MAILERSEND_API_KEY must be set");

    // Log or use the variable as needed
    println!("Using API Key: {}", api_key);

    println!("Starting server at http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                api_key: api_key.clone(),
            }))
            .service(submit_form)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}