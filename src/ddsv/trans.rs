use super::data::SharedVars;

#[derive(Copy)]
pub struct Trans {
    pub label: String,
    pub location: String,
    pub guard: Box<dyn Fn(SharedVars) -> bool>,
    pub action: Box<dyn Fn(SharedVars) -> SharedVars>,
}

impl Trans {
    pub fn new(
        label: String,
        location: String,
        guard: Box<dyn Fn(SharedVars) -> bool>,
        action: Box<dyn Fn(SharedVars) -> SharedVars>,
    ) -> Trans {
        Trans {
            label: label,
            location: location,
            guard: guard,
            action: action,
        }
    }
}
