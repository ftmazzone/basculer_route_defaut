use std::{ time::Duration};

pub trait FormateurOption {
    fn formater_option(self) ->String;
}

impl FormateurOption for Option<i32> {
    fn formater_option(self) ->String {
        let valeur_formatee: String;
        match self {
            None => valeur_formatee = String::new(),
            Some(v) => valeur_formatee = format!("{}", v),
        }
        return valeur_formatee;
    }
}

impl FormateurOption for Option<f32> {
    fn formater_option(self) ->String {
        let valeur_formatee: String;
        match self {
            None => valeur_formatee = String::new(),
            Some(v) => valeur_formatee = format!("{}", v),
        }
        return valeur_formatee;
    }
}

impl FormateurOption for Option<Duration> {
    fn formater_option(self) ->String {
        let valeur_formatee: String;
        match self {
            None => valeur_formatee = String::new(),
            Some(v) => valeur_formatee = format!("{:?}", v),
        }
        return valeur_formatee;
    }
}
