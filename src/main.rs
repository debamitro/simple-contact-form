use actix_cors::Cors;
use actix_web::{post, web, App, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
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
    // Send email
    let api_key = data.api_key.clone();
    match send_email(&form.name, &form.email, &form.message, &form.id, &api_key).await {
        Ok(_) => "Form submitted successfully! Email sent.".to_string(),
        Err(e) => {
            eprintln!("Failed to send email: {}", e);
            "Form submitted, but failed to send email.".to_string()
        }
    }
}

async fn send_email(
    name: &str,
    email: &str,
    message: &str,
    id: &str,
    resend_api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read accounts.json to find the recipient and sender email based on ID
    let accounts_json = include_str!("../accounts.json");
    let accounts: serde_json::Value = serde_json::from_str(&accounts_json)?;
    let accounts_array = accounts
        .get("accounts")
        .and_then(|acc| acc.as_array())
        .ok_or_else(|| "No 'accounts' array found in JSON".to_string())?;
    let account_entry = accounts_array
        .iter()
        .find(|entry| entry.get("id").and_then(|val| val.as_str()) == Some(id))
        .ok_or_else(|| format!("No account found for ID: {}", id))?;
    let to_email = account_entry
        .get("email")
        .and_then(|email| email.as_str())
        .ok_or_else(|| format!("No recipient email found for ID: {}", id))?
        .to_string();
    let from_email = account_entry
        .get("from_email")
        .and_then(|email| email.as_str())
        .ok_or_else(|| format!("No sender email found for ID: {}", id))?
        .to_string();

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
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(response.text().await?.into());
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    let api_key = env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set");
    let port: u16 = match env::var("PORT") {
        Ok(v) => v.parse::<u16>().unwrap_or(8080),
        Err(_) => 8080,
    };

    HttpServer::new(move || {
        let cors = Cors::default().allow_any_origin();

        App::new()
            .app_data(web::Data::new(AppState {
                api_key: api_key.clone(),
            }))
            .wrap(cors)
            .service(submit_form)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
