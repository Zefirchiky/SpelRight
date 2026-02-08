pub trait Language {
    fn name() -> &'static str;
    fn full_name() -> &'static str;
}