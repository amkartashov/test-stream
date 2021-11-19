Build err:

```
test-stream$ cargo build
   Compiling test-stream v0.1.0 (/Users/me/tmp/test-stream)
error: future cannot be shared between threads safely
  --> src/main.rs:32:12
   |
32 |         Ok(Box::pin(reads) as Self::ReadStream)
   |            ^^^^^^^^^^^^^^^ future created by async block is not `Sync`
   |
   = help: the trait `Sync` is not implemented for `dyn futures::Future<Output = Result<Option<bytes::Bytes>, ()>> + std::marker::Send`
note: future is not `Sync` as it awaits another future which is not `Sync`
  --> src/main.rs:27:21
   |
27 |           let reads = try_stream! {
   |  _____________________^
28 | |             while let Some(data) = blob_reader.read().await? {
29 | |                 yield ReadResponse{data};
30 | |             }
31 | |         };
   | |_________^ await occurs here on type `Pin<Box<dyn futures::Future<Output = Result<Option<bytes::Bytes>, ()>> + std::marker::Send>>`, which is not `Sync`
   = note: required for the cast to the object type `dyn futures::Stream<Item = Result<ReadResponse, ()>> + Sync + std::marker::Send`
   = note: this error originates in the macro `$crate::async_stream_impl::try_stream_inner` (in Nightly builds, run with -Z macro-backtrace for more info)

error: could not compile `test-stream` due to previous error
```

