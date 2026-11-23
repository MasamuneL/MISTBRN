macro_rules! opaque_id {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }
            pub fn as_str(&self) -> &str { &self.0 }
        }
    )+};
}

opaque_id!(
    PlayerId,
    WorldId,
    NodeId,
    ResearchId,
    EventId,
    TransactionId
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metal {
    Iron,
    Steel,
    Tin,
    Pewter,
    Zinc,
    Brass,
    Copper,
    Bronze,
}

impl Metal {
    pub const ALL: [Self; 8] = [
        Self::Iron,
        Self::Steel,
        Self::Tin,
        Self::Pewter,
        Self::Zinc,
        Self::Brass,
        Self::Copper,
        Self::Bronze,
    ];
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn pair(self) -> Pair {
        match self {
            Self::Iron | Self::Steel => Pair::IronSteel,
            Self::Tin | Self::Pewter => Pair::TinPewter,
            Self::Zinc | Self::Brass => Pair::ZincBrass,
            Self::Copper | Self::Bronze => Pair::CopperBronze,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pair {
    IronSteel,
    TinPewter,
    ZincBrass,
    CopperBronze,
}

impl Pair {
    /// The pair's two metals, in `Metal::ALL` order.
    pub fn metals(self) -> [Metal; 2] {
        match self {
            Self::IronSteel => [Metal::Iron, Metal::Steel],
            Self::TinPewter => [Metal::Tin, Metal::Pewter],
            Self::ZincBrass => [Metal::Zinc, Metal::Brass],
            Self::CopperBronze => [Metal::Copper, Metal::Bronze],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}
