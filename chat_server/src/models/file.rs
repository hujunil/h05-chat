use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String,
    pub hash: String,
}

impl ChatFile {
    pub fn new(ws_id: u64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: filename.split('.').last().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}", self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}/{}.{}", self.ws_id, part1, part2, part3, self.ext)
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    // convert /files/s/339/807/e635afbeab088ce33206fdf4223a6bb156.png to ChatFile
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError(format!(
                "Invalid chat file path: {}",
                s
            )));
        };

        let parts = s.split('/').collect::<Vec<_>>();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError(format!(
                "File path {} does not valid",
                s
            )));
        }

        let Ok(ws_id) = parts[0].parse::<u64>() else {
            return Err(AppError::ChatFileError(format!(
                "Invalid workspace id: {}",
                parts[0]
            )));
        };

        let Some((part3, ext)) = parts[3].split_once('.') else {
            return Err(AppError::ChatFileError(format!(
                "Invalid file name: {}",
                parts[3]
            )));
        };

        let hash = format!("{}{}{}", parts[1], parts[2], part3);

        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_file_new_should_work() {
        let data = b"hello world";
        let chat_file = ChatFile::new(1, "test.txt", data);
        assert_eq!(chat_file.ws_id, 1);
        assert_eq!(chat_file.ext, "txt");
        assert_eq!(chat_file.hash.len(), 40);
        assert_eq!(chat_file.hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");

        let url = chat_file.url();
        assert_eq!(
            url,
            "/files/1/2aa/e6c/35c94fcfb415dbe95f408b9ce91ee846ed.txt"
        );
    }
}
