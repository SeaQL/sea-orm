#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use std::path::PathBuf;

use axum::{body::Body, debug_handler, extract::Multipart};
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use tokio::{fs, io::AsyncWriteExt};
use tokio_util::io::ReaderStream;

use crate::models::_entities::files;

const UPLOAD_DIR: &str = "./uploads";

#[debug_handler]
pub async fn upload(
    _auth: auth::JWT,
    Path(notes_id): Path<i32>,
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> Result<Response> {
    // Collect all uploaded files
    let mut files = Vec::new();

    // Iterate all files in the POST body
    while let Some(field) = multipart.next_field().await.map_err(|err| {
        tracing::error!(error = ?err,"could not readd multipart");
        Error::BadRequest("could not readd multipart".into())
    })? {
        // Get the file name
        let file_name = match field.file_name() {
            Some(file_name) => file_name.to_string(),
            _ => return Err(Error::BadRequest("file name not found".into())),
        };

        // Get the file content as bytes
        let content = field.bytes().await.map_err(|err| {
            tracing::error!(error = ?err,"could not readd bytes");
            Error::BadRequest("could not readd bytes".into())
        })?;

        // Create a folder to store the uploaded file
        let now = chrono::offset::Local::now()
            .format("%Y%m%d_%H%M%S")
            .to_string();
        let uuid = uuid::Uuid::new_v4().to_string();
        let folder = format!("{now}_{uuid}");
        let upload_folder = PathBuf::from(UPLOAD_DIR).join(&folder);
        fs::create_dir_all(&upload_folder).await?;

        // Write the file into the newly created folder
        let path = upload_folder.join(file_name);
        let mut f = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await?;
        f.write_all(&content).await?;
        f.flush().await?;

        // Record the file upload in database
        let file = files::ActiveModel {
            notes_id: ActiveValue::Set(notes_id),
            file_path: ActiveValue::Set(
                path.strip_prefix(UPLOAD_DIR)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?;

        files.push(file);
    }

    format::json(files)
}

#[debug_handler]
pub async fn list(
    _auth: auth::JWT,
    Path(notes_id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Fetch all files uploaded for a specific notes
    let files = files::Entity::find()
        .filter(files::Column::NotesId.eq(notes_id))
        .order_by_asc(files::Column::Id)
        .all(&ctx.db)
        .await?;

    format::json(files)
}

#[debug_handler]
pub async fn view(
    _auth: auth::JWT,
    Path(files_id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Fetch the file info from database
    let file = files::Entity::find_by_id(files_id)
        .one(&ctx.db)
        .await?
        .expect("File not found");

    // Stream the file
    let file = fs::File::open(format!("{UPLOAD_DIR}/{}", file.file_path)).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(format::render().response().body(body)?)
}

pub fn routes() -> Routes {
    // Bind the routes
    Routes::new()
        .prefix("files")
        .add("/upload/:notes_id", post(upload))
        .add("/list/:notes_id", get(list))
        .add("/view/:files_id", get(view))
}
