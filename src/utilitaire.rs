use std::{ time::Duration};
use std::net::IpAddr;

pub trait FormateurOption {
    fn formater(self) ->String;
}

impl FormateurOption for Option<i32> {
    fn formater(self) ->String {
        let valeur_formatee: String;
        match self {
            None => valeur_formatee = String::new(),
            Some(v) => valeur_formatee = format!("{}", v),
        }
        return valeur_formatee;
    }
}

impl FormateurOption for Option<f32> {
    fn formater(self) ->String {
        let valeur_formatee: String;
        match self {
            None => valeur_formatee = String::new(),
            Some(v) => valeur_formatee = format!("{}", v),
        }
        return valeur_formatee;
    }
}

impl FormateurOption for Option<Duration> {
    fn formater(self) ->String {
        let valeur_formatee: String;
        match self {
            None => valeur_formatee = String::new(),
            Some(v) => valeur_formatee = format!("{:?}", v),
        }
        return valeur_formatee;
    }
}

impl FormateurOption for Option<IpAddr> {
    fn formater(self) ->String {
        let valeur_formatee: String;
        match self {
            None => valeur_formatee = String::new(),
            Some(v) => valeur_formatee = format!("{:?}", v),
        }
        return valeur_formatee;
    }
}
