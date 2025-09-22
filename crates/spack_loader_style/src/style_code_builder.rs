pub struct StyleCodeBuilder {
  pub raw: String,
}

impl StyleCodeBuilder {
  pub fn new(raw: String) -> Self {
    Self { raw }
  }

  pub fn build(&self) -> String {
    self.raw.clone()
  }
}
