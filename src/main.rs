use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use std::path::PathBuf;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};

// 视频信息结构
#[derive(Serialize, Deserialize, Clone)]
struct VideoInfo {
    id: String,
    path: String,
    name: String,
}

// 状态管理结构
struct AppState {
    videos: Mutex<HashMap<String, VideoInfo>>,
}

// 注册新视频
#[derive(Deserialize)]
struct CreateVideoRequest {
    path: String,
    name: String,
}

async fn register_video(
    data: web::Data<AppState>,
    video_req: web::Json<CreateVideoRequest>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let video = VideoInfo {
        id: id.clone(),
        path: video_req.path.clone(),
        name: video_req.name.clone(),
    };
    
    let mut videos = data.videos.lock().unwrap();
    videos.insert(id.clone(), video.clone());
    
    HttpResponse::Ok().json(video)
}

// 获取视频流
async fn stream_video(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> actix_web::Result<NamedFile> {
    let videos = data.videos.lock().unwrap();
    if let Some(video) = videos.get(&id.into_inner()) {
        let path = PathBuf::from(&video.path);
        Ok(NamedFile::open(path)?)
    } else {
        Err(actix_web::error::ErrorNotFound("Video not found"))
    }
}

// 获取所有视频列表
async fn list_videos(data: web::Data<AppState>) -> impl Responder {
    let videos = data.videos.lock().unwrap();
    let video_list: Vec<VideoInfo> = videos.values().cloned().collect();
    HttpResponse::Ok().json(video_list)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        videos: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/api/videos", web::post().to(register_video))
            .route("/api/videos", web::get().to(list_videos))
            .route("/api/videos/{id}", web::get().to(stream_video))
    })
    .bind("0.0.0.0:8888")?
    .run()
    .await
}
