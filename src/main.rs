use std::env;

use clap::Parser;
use deepgram::{transcription::prerecorded::{audio_source::AudioSource, options::{Language, Options, Tier}}, Deepgram};
use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Youtube video url to transcribe
    #[arg(short, long)]
    youtube_url: String,
}


#[tokio::main]
async fn main() -> anyhow::Result<()>{
    dotenvy::dotenv().expect(".env file not found");
    
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("the DEEPGRAM_API_KEY environmental variable is required");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let args = Args::parse();
    let yt_url = args.youtube_url;

    let transcript = yt_url_to_text(&yt_url, &dg_client).await?;

    println!("video transcript {transcript}");

    Ok(())
}


async fn yt_url_to_text(
    youtube_url: &str,
    deepgram_client: &Deepgram,
) -> anyhow::Result<String> {
    let video_options = VideoOptions {
        quality: VideoQuality::HighestAudio,
        filter: VideoSearchOptions::Audio,
        ..Default::default()
    };

    // TODO; figure out why this constantly errors out

    println!("youtube_url - {youtube_url}");

    let video = Video::new_with_options(youtube_url, video_options)?;

    let stream = video.stream().await?;

    let mut audio_bytes = Vec::new();
    while let Some(chunk) = stream.chunk().await.unwrap() {
        audio_bytes.extend(chunk);
    }

    println!("downloading audio to memory");

    let source = AudioSource::from_buffer(audio_bytes);

    // Adds Read and Seek to the bytes via Cursor
    let options = Options::builder()
        .punctuate(true)
        .tier(Tier::Enhanced)
        .language(Language::en_US)
        .build();

    let response = deepgram_client
        .transcription()
        .prerecorded(source, &options)
        .await?;

    println!("transcribing audio with deepgram");

    println!("transcription complete");

    Ok(response.results.channels[0].alternatives[0]
        .transcript
        .clone()
        .to_owned())
}
