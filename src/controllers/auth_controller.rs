use axum::{
    body::{self, Full},
    extract::{State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json
};

use mime_guess::mime;

use mongodb::{Client, bson::doc, Collection};

use serde::Serialize;
use serde_json::{json};

use jsonwebtoken::Algorithm;
use std::{env, time::{SystemTime, UNIX_EPOCH}};

use crypto::{digest::Digest, sha3::Sha3};

#[path ="../models.rs"]
mod models;

use models::{auth_model::{UserAccount}, resp_model::ServerApiResponse};

use self::models::auth_model::{UserPayload, Claims};

fn hash_string(hash_str: String) -> String {
    let mut hasher = Sha3::sha3_384();

    hasher.input(hash_str.as_bytes());

    hasher.result_str()
}

async fn user_registered(accounts: Collection<UserAccount>, account: UserAccount) -> Option<UserAccount> {
    let user_doc = doc!{"username": account.username};
    accounts.find_one(user_doc, None).await.unwrap_or(Option::None)
}

async fn user_matches(accounts: Collection<UserAccount>, account: UserAccount) -> Option<UserAccount> {
    let user_doc = doc!{"username": account.username, "password": hash_string(account.password)};
    accounts.find_one(user_doc, None).await.unwrap_or(Option::None)
}

fn return_invalid_data() -> Response {
    let response = ServerApiResponse {status: "err".to_string(), message: "Invalid user data provided".to_string() };

    Response::builder()
    .status(StatusCode::OK)
    .header(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
    )
    .body(body::boxed(Full::from(json!(response).to_string())))
    .unwrap()
}

fn acquire_jwt<T: Serialize>(payload: T) -> String {
    // Create secret from ENV VAR
    let secret = jsonwebtoken::EncodingKey::from_secret(env::var("JWT_SECRET_PRIMARY").expect("Make sure that JWT_SECRET_PRIMARY is set").as_ref());

    // Now
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Create the JWT
    let claims = Claims { name: json!(payload).to_string(), exp: now + (12 * 3600), iat: now };
    
    // Create a new header
    let header = jsonwebtoken::Header::new(Algorithm::HS384);

    jsonwebtoken::encode(&header, &claims, &secret).unwrap()
}

fn verify_jwt(jwt: String) -> bool {
    let secret = env::var("JWT_SECRET_PRIMARY").expect("Make sure that JWT_SECRET_PRIMARY is set");
    
    let jwt_split = jwt.split('.');
    let jwt_split:Vec<_> = jwt_split.collect();

    if jwt_split.len() == 3 {
        let message = jwt_split[0].to_owned() + "." + jwt_split[1];
        let signature = jwt_split[2];
        let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
        return jsonwebtoken::crypto::verify(signature, message.as_bytes(), &key, Algorithm::HS384).unwrap_or(false)
    }

    false
}

// pub async fn validate_jwt(jwt: String) -> bool {
//     let decodingKey = jsonwebtoken::DecodingKey::from_secret(env::var("JWT_SECRET_PRIMARY").expect("Make sure that JWT_SECRET_PRIMARY is set").as_ref());
    
//     // sign: message, key, algorithm
//     // verify: signature, message, key, algorithm
//     jsonwebtoken::crypto::verify(jwt.as_ref(), &decodingKey, );
// }

pub async fn handle_login(State(client) : State<Client>, Json(payload): Json<UserAccount>) -> impl IntoResponse {
    // Not too sure about MongoDB's SQL-injection best practices.
    // Going to trust the crate to handle that for me.
    
    // First goal: Retrieve username + password from JSON
    let username: String = payload.username;
    let password: String = payload.password;

    if username.is_empty() || password.is_empty() {
        return return_invalid_data()
    }

    // Non-variable
    let accounts = client.database("chat").collection::<UserAccount>("accounts");

    // Check if the user is logging in successfully
    if user_matches(accounts.clone(), UserAccount {username: username.clone(), password: password.clone()}).await.is_some() {

        // Payload is a serialized username
        let new_payload = UserPayload { uid: username };

        // Sign the JWT
        let token = acquire_jwt(new_payload);
        
        // Generate the response to client
        let response = ServerApiResponse {status: "jwt".to_string(), message: token };

        return Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
        )
        .body(body::boxed(Full::from(json!(response).to_string())))
        .unwrap()
    }

    let response = ServerApiResponse {status: "err".to_string(), message: "Incorrect login details".to_string() };

    return Response::builder()
    .status(StatusCode::OK)
    .header(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
    )
    .body(body::boxed(Full::from(json!(response).to_string())))
    .unwrap()
}

pub async fn handle_new_user(State(client) : State<Client>, Json(payload): Json<UserAccount>) -> impl IntoResponse {
    // Not too sure about MongoDB's SQL-injection best practices.
    // Going to trust the crate to handle that for me.
    
    // First goal: Retrieve username + password from JSON
    let username: String = payload.username;
    let password: String = payload.password;

    if username.is_empty() || password.is_empty() {
        return return_invalid_data()
    }

    // Non-variable
    let accounts = client.database("chat").collection::<UserAccount>("accounts");

    // Check if account already exists
    if user_registered(accounts.clone(), UserAccount {username: username.clone(), password: password.clone()}).await.is_some() {
        let response = ServerApiResponse {status: "err".to_string(), message: "User account already exists".to_string() };

        return Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
        )
        .body(body::boxed(Full::from(json!(response).to_string())))
        .unwrap()
    }

    // Create a new account
    let account = UserAccount {username: username, password: hash_string(password)};
    
    accounts.insert_one(account, None).await.expect("Account creation failed");

    let response = ServerApiResponse {status: "ok".to_string(), message: "User account created successfully".to_string() };

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime::APPLICATION_JSON.as_ref()).unwrap(),
        )
        .body(body::boxed(Full::from(json!(response).to_string())))
        .unwrap()
}

pub async fn test_api(Json(payload) : Json<UserPayload>) -> impl IntoResponse {

    println!("{}", payload.uid);

    if verify_jwt(payload.uid) {
        println!("Verified");
        return (StatusCode::OK, "Verified")
    }
    println!("Failed!");
    (StatusCode::OK, "Verification failed")
}
