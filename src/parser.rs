use std::collections::HashMap;

use reqwest::Error;
use serde_json::Value;

#[derive(Debug)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub description: String,
    pub versions: Vec<String>,
    pub downloads: i64
}

#[tokio::main]
pub async fn search_projects(query: &str) -> Result<Vec<Project>, Error>{
    let response = reqwest::get(format!("https://api.modrinth.com/v2/search?query={query}&facets=[[\"categories:fabric\"],[\"project_type:mod\"]]")).await?;

    let body = response.json::<HashMap<String, Value>>().await?;

    let out = if let Value::Array(hits) = &body["hits"] {
        hits.iter().map(|p| {
            Project {
                id: p["project_id"].as_str().unwrap().to_string(),
                title: p["title"].as_str().unwrap().to_string(),
                description: p["description"].as_str().unwrap().to_string(),
                versions: p["versions"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect(),
                downloads: p["downloads"].as_i64().unwrap()
            }
        }).collect()
    } else {
        vec![]
    };
    
    Ok(out)
}

#[tokio::main]
pub async fn get_link_by_id(project_id: &str, version: &str) -> Result<String, Error> {
    let response = reqwest::get(format!("https://api.modrinth.com/v2/project/{project_id}/version")).await?;

    let body = response.json::<Vec<HashMap<String, Value>>>().await?;

    let mut out = String::new();

    for project_version in &body {
        let mut found = false;

        if let Value::Array(game_versions) = &project_version["game_versions"] {
            game_versions.iter().for_each(|v| {
                if v == version {
                    found = true;
                }
            })
        };

        if found {
            if let Value::Array(files) = &project_version["files"] {
                if let Value::Object(object) = &files[0] {
                    out = if let Value::String(url) = &object["url"] {
                        url.to_string()
                    } else {
                        String::new()
                    };
                }
            }
        }       
    }
    
    Ok(out)

}

#[tokio::main]
pub async fn download_project(url: &str) -> Result<(), Error> {

    let response = reqwest::get(url).await.expect("request failed");
    let body = response.text().await.expect("body invalid");
    let mut out = std::fs::File::create(format!("tmp/{}", url.split("/").last().unwrap())).expect("failed to create file");
    std::io::copy(&mut body.as_bytes(), &mut out).expect("failed to copy content");


    Ok(())
}