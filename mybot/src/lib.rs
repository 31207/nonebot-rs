pub mod helper;
pub mod meme_api;
pub mod meme_info;
pub mod meme_parser;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::fs;
    #[test]
    fn meme_info_parse_test() {
        let content = fs::read_to_string("../meme_info.txt");
        match content {
            Ok(content) => {
                let val: Result<Vec<meme_info::MemeInfo>, serde_json::Error> =
                    serde_json::from_str(content.as_str());
                match val {
                    Ok(val) => {
                        assert!(true);
                    }
                    Err(err) => {
                        panic!("{}", err);
                    }
                }
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    #[tokio::test]
    async fn meme_api_upload_and_get_image_test() {
        let url = "https://dustwind.xyz/upload/7.gif";
        let result = meme_api::upload_image(url).await;
        match result {
            Ok(image_id) => {
                println!("Uploaded image ID: {}", image_id);
                let base64_result = meme_api::get_image_base64(&image_id).await;
                match base64_result {
                    Ok(base64_str) => {
                        println!("Base64 string length: {}", base64_str.len());
                        assert!(base64_str.len() > 0);
                    }
                    Err(err) => {
                        panic!("Get image base64 failed: {}", err);
                    }
                }
            }
            Err(err) => {
                panic!("Upload failed: {}", err);
            }
        }

    }
}
