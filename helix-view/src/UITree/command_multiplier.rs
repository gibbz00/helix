use std::num::NonZeroU32;

pub struct CommandMultiplier {
    // u32 because char.to_digit() returns u32
    multiplier: Option<NonZeroU32>
}

impl CommandMultiplier {
    pub fn new() -> Self {
        Self {
            multiplier: None
        }
    }
    
    pub fn set(&self, multiplier: NonZeroU32) {
        self.multiplier = Some(multiplier);
    }

    pub fn clear(&self) {
        self.multiplier = None;
    }

    pub fn get(&self) -> Option<NonZeroU32> {
        self.multiplier
    }

    pub fn unwrap_or_one(&self) -> NonZeroU32 {
        match self.multiplier {
            Some(multiplier) => multiplier,
            None => unsafe { NonZeroU32::new_unchecked(1) }
        }
        if let Some(multiplier) = self.multiplier {
            multiplier
        }
        
    }

    pub fn push_digit(&self, digit: NonZeroU32) {
        // At least for now, tad bit easier than separating
        // and then shifting digits numerically.
        self.multiplier = Some(
            (self.multiplier.map(|multiplier| multiplier.get()).unwrap_or_default().to_string() + digit.to_string()).parse()
            .expect("Added digit to UITree.command_multiplier should not overflow an u32"))
    }
}
