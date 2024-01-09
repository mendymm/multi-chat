use misc::{get_live_stream_id, get_ytcfg};

mod misc;

#[tokio::main]
async fn main() {
    let creator_name = "RyanHallYall";


    let ytcfg = get_ytcfg(creator_name).await.unwrap();
    dbg!(ytcfg);
    // let video_id = match get_live_stream_id(creator_name).await {
    //     Some(video_id) => video_id,
    //     None => panic!(
    //         "Error when getting stream id for youtube channel @{}",
    //         creator_name
    //     ),
    // };
}
