pub trait ResultExt<R, E> {
    fn report(self) -> R;
}

impl<R, E: miette::Diagnostic + Sync + Send + 'static> ResultExt<R, E> for Result<R, E> {
    fn report(self) -> R {
        match self {
            Ok(res) => res,
            Err(err) => panic!("{:?}", miette::Report::new(err)),
        }
    }
}
