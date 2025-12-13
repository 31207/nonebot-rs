use std::option;

use base64::prelude::*;
use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Deserialize, Serialize)]
struct ApiUploadResponse {
    pub image_id: String,
}
pub async fn upload_image(url: &str) -> Result<String, String>{
    let client = reqwest::Client::new();
    let resp = client.post("http://127.0.0.1:2233/image/upload").json(&serde_json::json!({
        "type": "url",
        "url": url
    })).send().await;
    match resp {
        Ok(resp) => {
            let json_resp: Result<serde_json::Value, reqwest::Error> = resp.json().await;
            match json_resp {
                Ok(json_resp) => {
                    match serde_json::from_value::<ApiUploadResponse>(json_resp) {
                        Ok(api_response) => return Ok(api_response.image_id),
                        Err(_) => {
                            return Err("Failed to parse API response".to_string());
                        }
                    }
                },
                Err(_) => {
                    return Err("Failed to parse response JSON".to_string());
                }
            };
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };

}


pub async fn get_image_base64(image_id: &str) -> Result<String, String>{
    let client = reqwest::Client::new();
    let resp = client.get(format!("http://127.0.0.1:2233/image/{}", image_id).as_str()).send().await;
    match resp {
        Ok(resp) => {
            let bytes = resp.bytes().await;
            match bytes {
                Ok(bytes) => {
                    let base64_str = BASE64_STANDARD.encode(&bytes);
                    return Ok(base64_str);
                },
                Err(err) => {
                    return Err(err.to_string());
                }
            }
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };
}
#[derive(Deserialize, Serialize)]
struct ApiImage {
    name: String,
    id: String,
}

#[derive(Deserialize, Serialize)]
struct ApiMemeRequest {
    texts: Vec<String>,
    images: Vec<ApiImage>,
    options: serde_json::Value,
}
pub async fn memes(key: &str, texts: Vec<String>, image_ids: Vec<String>, meme_options: serde_json::Value) -> Result<String, String>{
    let images: Vec<ApiImage> = image_ids.into_iter().map(|id| ApiImage {
        name: id.clone(),
        id,
    }).collect();
    let client = reqwest::Client::new();
    let req = ApiMemeRequest {
        texts,
        images,
        options: meme_options,
    };
    let req_json = serde_json::to_value(req).unwrap();
    let resp = client.post(format!("http://127.0.0.1:2233/memes/{}", key).as_str()).json(&req_json).send().await;
    match resp {
        Ok(resp) => {
            // println!("{}",&resp.text().await.unwrap());
            // return Err("Not implemented".to_string());

            let json_resp: Result<serde_json::Value, reqwest::Error> = resp.json().await;
            match json_resp {
                Ok(json_resp) => {
                    match json_resp.get("image_id") {
                        Some(image_id_value) => {
                            match image_id_value.as_str() {
                                Some(image_id) => return Ok(image_id.to_string()),
                                None => return Err("Invalid image_id format".to_string()),
                            }
                        },
                        None => return Err("image_id not found in response".to_string()),
                    }
                },
                Err(err) => {
                    return Err(format!("Failed to parse response JSON: {}", err));
                }
            };
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };
}