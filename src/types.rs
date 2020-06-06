// a tristate output pin
#[derive(Eq, PartialEq, Debug)]
pub enum Tristate {
    HiZ,
    High,
    Low,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum Signal {
    M1,
    ST,
    MREQ,
    RD,
    WAIT,
}
