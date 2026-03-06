use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn assert_success(step: &str, value: &Value) -> Result<(), String> {
    let ok = value.get("success").and_then(Value::as_bool).unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(format!("[{step}] failed: {value}"))
    }
}

fn read_verify_code() -> Result<String, String> {
    print!("CONFIRMATION_CODE: ");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut code = String::new();
    io::stdin().read_line(&mut code).map_err(|e| e.to_string())?;
    let code = code.trim().to_string();
    if code.is_empty() {
        return Err("confirmation code cannot be empty".to_string());
    }
    Ok(code)
}

fn stop_services() -> Result<(), String> {
    let output = Command::new("bash")
        .arg("./docker/backend/run_container.sh")
        .arg("--stop")
        .output()
        .map_err(|e| format!("failed to execute stop command: {e}"))?;

    if output.status.success() {
        println!("[stop] services stopped");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("stop command failed: {}", stderr.trim()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_base = env_or_default("API_BASE", "http://localhost:3000");
    let password = env_or_default("PASSWORD", "StrongPass123");
    let name = env_or_default("NAME", "Post Workflow User");
    let title = env_or_default("TITLE", "My first post from Rust workflow");
    let content =
        env_or_default("CONTENT", "This post was created by examples/post_api_workflow.rs");
    let email = env::var("EMAIL")
        .unwrap_or_else(|_| format!("workflow_{}_{}@example.com", unix_seconds(), std::process::id()));

    println!("== Workflow: register -> verify -> login -> create post -> get post ==");
    println!("API_BASE: {api_base}");
    println!("EMAIL: {email}");

    let client = Client::new();

    // 1) Register
    let register_payload = json!({
        "email": email,
        "name": name,
        "password": password
    });
    let register_resp: Value = client
        .post(format!("{api_base}/api/auth/register"))
        .json(&register_payload)
        .send()
        .await?
        .json()
        .await?;
    assert_success("register", &register_resp).map_err(io::Error::other)?;
    println!("[register] ok");

    // 2) Verify (read from VERIFY_CODE env or prompt)
    let verify_code = match env::var("VERIFY_CODE") {
        Ok(code) if !code.trim().is_empty() => code,
        _ => {
            println!();
            println!("Enter the confirmation code sent to {email}");
            read_verify_code().map_err(io::Error::other)?
        },
    };

    let verify_payload = json!({
        "email": email,
        "code": verify_code
    });
    let verify_resp: Value = client
        .post(format!("{api_base}/api/auth/verify"))
        .json(&verify_payload)
        .send()
        .await?
        .json()
        .await?;
    assert_success("verify", &verify_resp).map_err(io::Error::other)?;
    println!("[verify] ok");

    // 3) Login
    let login_payload = json!({
        "email": email,
        "password": password
    });
    let login_resp: Value = client
        .post(format!("{api_base}/api/auth/login"))
        .json(&login_payload)
        .send()
        .await?
        .json()
        .await?;
    assert_success("login", &login_resp).map_err(io::Error::other)?;

    let token = login_resp["data"]["access_token"]
        .as_str()
        .ok_or_else(|| io::Error::other("[login] missing access_token"))?
        .to_string();
    println!("[login] ok");

    // 4) Create post
    let create_payload = json!({
        "title": title,
        "content": content,
        "status": "draft",
        "tags": ["workflow", "example", "rust"]
    });
    let create_resp: Value = client
        .post(format!("{api_base}/api/posts"))
        .bearer_auth(&token)
        .json(&create_payload)
        .send()
        .await?
        .json()
        .await?;
    assert_success("create_post", &create_resp).map_err(io::Error::other)?;

    let post_id = create_resp["data"]["id"]
        .as_str()
        .ok_or_else(|| io::Error::other("[create_post] missing post id"))?
        .to_string();
    println!("[create_post] ok (id={post_id})");

    // 5) Get post
    let get_resp: Value = client
        .get(format!("{api_base}/api/posts/{post_id}"))
        .bearer_auth(&token)
        .send()
        .await?
        .json()
        .await?;
    assert_success("get_post", &get_resp).map_err(io::Error::other)?;
    println!("[get_post] ok");

    println!("\n== Final post payload ==");
    println!("{}", serde_json::to_string_pretty(&get_resp["data"])?);

    // 6) Stop docker services after successful workflow.
    stop_services().map_err(io::Error::other)?;

    Ok(())
}
