use tonic::transport::Server;
use tonic::{Request, Response, Status};

use tonic_grpc_example::post::{
    blogpost_server::{Blogpost, BlogpostServer},
    Post, PostId, PostList, PostPerPage, ProcessStatus,
};

use entity::{
    post::{self, Entity as PostEntity},
    sea_orm::{entity::*, query::*, DatabaseConnection},
};
use migration::{Migrator, MigratorTrait};

use std::env;

#[derive(Default)]
pub struct MyServer {
    connection: DatabaseConnection,
}

#[tonic::async_trait]
impl Blogpost for MyServer {
    async fn get_posts(&self, request: Request<PostPerPage>) -> Result<Response<PostList>, Status> {
        let mut response = PostList { post: Vec::new() };

        let posts = PostEntity::find()
            .order_by_asc(post::Column::Id)
            .limit(request.into_inner().per_page)
            .all(&self.connection)
            .await
            .unwrap();

        for post in posts {
            response.post.push(Post {
                id: post.id,
                title: post.title,
                content: post.text,
            });
        }

        Ok(Response::new(response))
    }

    async fn add_post(&self, request: Request<Post>) -> Result<Response<PostId>, Status> {
        let input = request.into_inner();
        let insert_details = post::ActiveModel {
            title: Set(input.title.clone()),
            text: Set(input.content.clone()),
            ..Default::default()
        };

        let response = PostId {
            id: insert_details.insert(&self.connection).await.unwrap().id,
        };

        Ok(Response::new(response))
    }

    async fn update_post(&self, request: Request<Post>) -> Result<Response<ProcessStatus>, Status> {
        let input = request.into_inner();
        let mut update_post: post::ActiveModel = PostEntity::find_by_id(input.id)
            .one(&self.connection)
            .await
            .unwrap()
            .unwrap()
            .into();

        update_post.title = Set(input.title.clone());
        update_post.text = Set(input.content.clone());

        let update = update_post.update(&self.connection).await;

        match update {
            Ok(_) => Ok(Response::new(ProcessStatus { success: true })),
            Err(_) => Ok(Response::new(ProcessStatus { success: false })),
        }
    }

    async fn delete_post(
        &self,
        request: Request<PostId>,
    ) -> Result<Response<ProcessStatus>, Status> {
        let delete_post: post::ActiveModel = PostEntity::find_by_id(request.into_inner().id)
            .one(&self.connection)
            .await
            .unwrap()
            .unwrap()
            .into();

        let status = delete_post.delete(&self.connection).await;

        match status {
            Ok(_) => Ok(Response::new(ProcessStatus { success: true })),
            Err(_) => Ok(Response::new(ProcessStatus { success: false })),
        }
    }

    async fn get_post_by_id(&self, request: Request<PostId>) -> Result<Response<Post>, Status> {
        let post = PostEntity::find_by_id(request.into_inner().id)
            .one(&self.connection)
            .await
            .unwrap()
            .unwrap();

        let response = Post {
            id: post.id,
            title: post.title,
            content: post.text,
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // establish database connection
    let connection = sea_orm::Database::connect(&database_url).await?;
    Migrator::up(&connection, None).await?;

    let hello_server = MyServer { connection };
    Server::builder()
        .add_service(BlogpostServer::new(hello_server))
        .serve(addr)
        .await?;

    Ok(())
}
