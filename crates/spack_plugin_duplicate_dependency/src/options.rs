pub type CompilationHookFn =
    Box<dyn Fn(DuplicateDependencyPluginResponse) -> BoxFuture<'static, Result<()>> + Sync + Send>;

#[derive(Debug)]
pub struct DuplicateDependencyPluginOptions {
    #[debug(skip)]
    pub on_detected: Option<CompilationHookFn>,
}
