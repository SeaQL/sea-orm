use tonic::transport::Endpoint;
use tonic::Request;

use tonic_grpc_example::post::{blogpost_client::BlogpostClient, PostPerPage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = Endpoint::from_static("http://0.0.0.0:50051");

    /*
    Client code is not implemented in completely
     as it would just make the code base look too complicated ....
     and interface requires a lot of boilerplate code to implement.

     But a basic implementation is given below ....
     please refer it to implement other ways to make your code pretty
    */

    let mut client = BlogpostClient::connect(addr).await?;
    let request = Request::new(PostPerPage { per_page: 10 });
    let response = client.get_posts(request).await?;

    println!("total posts = {}", response.into_inner().post.len());

    Ok(())
}
