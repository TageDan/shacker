use crate::database::Flag;

struct EditScreen {
    flag: Flag,
    focus: usize,
}

impl EditScreen {
    fn new(flag: Flag) -> Self {
        Self { flag, focus: 0 }
    }
}
