fn main() {
    println!("Hello, world!");
}

use async_stream::try_stream;
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::codec::{FramedRead, BytesCodec};
use std::os::unix::prelude::MetadataExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncReadExt;

pub struct MyService {}

#[async_trait]
impl ByteStream for MyService {
    type ReadStream =
        std::pin::Pin<Box<dyn Stream<Item = std::result::Result<ReadResponse, ()>> + Send + Sync>>;

    async fn read(&self, _request: ReadRequest) -> Result<Self::ReadStream, ()> {

        let mut blob_reader = FileBlobReader::new(PathBuf::new(), 0, 0, 1_000_000).await?;

        let reads = try_stream! {
            while let Some(data) = blob_reader.read().await? {
                yield ReadResponse{data};
            }
        };
        Ok(Box::pin(reads) as Self::ReadStream)
    }
}

#[async_trait]
pub trait ByteStream: Send + Sync + 'static {
    type ReadStream: Stream<Item = Result<ReadResponse, ()>> + Send + Sync + 'static;

    async fn read(&self, request: ReadRequest) -> Result<Self::ReadStream, ()>;
}

pub struct ReadResponse {
    pub data: Bytes,
}

pub struct ReadRequest {
    pub resource_name: String,
    pub read_offset: i64,
    pub read_limit: i64,
}

#[async_trait]
pub trait BlobReader {
    async fn read(&mut self) -> Result<Option<Bytes>,()>;
}

#[derive(Debug)]
pub struct FileBlobReader {
    stream: FramedRead<tokio::io::Take<File>, BytesCodec>,
}

impl FileBlobReader {
    async fn new(path: PathBuf, read_offset: u64,
                 read_limit: u64,
                 max_chunk_size: u64,) -> Result<Self, ()> {
        let size = path.metadata().unwrap().size();
        if size < read_offset {
            Err(())
        } else {
            let mut file = File::open(path.as_path())
                .await
                .map_err(|_| ())?;
            file.seek(std::io::SeekFrom::Start(read_offset))
                .await
                .map_err(|_| ())?;

            let file = if read_limit > 0 { file.take(read_limit) } else { file.take(size) };
            let stream = FramedRead::with_capacity(file, BytesCodec::new(), max_chunk_size as usize);

            Ok(Self { stream })
        }
    }
}

#[async_trait]
impl BlobReader for FileBlobReader {
    async fn read(&mut self) -> Result<Option<Bytes>,()> {
        use futures::TryStreamExt;
        Ok(self.stream
            .try_next()
            .await
            .map_err(|_| ())?
            .map(|data| data.freeze()))
    }
}
