use chrono::{DateTime, Local};
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::process::{Command, Stdio};
use std::time::Duration;

//dbg!(routes_groupees.clone());

pub struct Route {
    pub interface: String,
    pub duree: Option<Duration>,
    pub route: String,
}

impl Route {
    pub fn new(interface: String, route: String, duree: Option<Duration>) -> Self {
        Self {
            interface,
            route,
            duree: duree,
        }
    }
}

pub struct Interface {
    pub interface: String,
    pub durees: BTreeMap<DateTime<Local>, Duration>,
    pub duree_moyenne: Duration,
}

impl Interface {
    pub fn new(interface: String) -> Self {
        Self {
            interface,
            durees: BTreeMap::new(),
            duree_moyenne: Duration::from_secs(1000)
        }
    }
}

pub struct Interfaces {
    pub liste_interfaces: HashMap<String, Interface>,
}

impl Interfaces {
    pub fn new() -> Self {
        Self {
            liste_interfaces: HashMap::new(),
        }
    }
}

/// Valider si la route est fonctionnelle et calculer la durée d'une requête ICMP.
/// Si la route n'est pas fonctionnelle, la durée retournée est de 1000 secondes.
pub fn tester_route(interface: &String, interfaces: &mut Interfaces) -> Duration {
    let commande = Command::new("ping")
        .arg("-c 1")
        .arg("-w 5")
        .arg(format!("{}{}", "-I", interface))
        .arg("1.1.1.1")
        .stdout(Stdio::piped())
        .output()
        .unwrap();

    let resultat_commande = String::from_utf8(commande.stdout).unwrap();

    let mut duree: Duration = Duration::from_secs(1000);
    let regex = Regex::new(r"icmp_seq=1 ttl=[0-9]{1,100} time=([0-9.]{1,100}) ms").unwrap();
    for element in regex.captures_iter(&resultat_commande) {
        //microsecondes
        let duree_us = element[1].parse::<f32>().unwrap() * 1000.0;
        duree = Duration::from_micros(duree_us as u64);
    }

    let interface_a_mettre_a_jour = interfaces
        .liste_interfaces
        .entry(interface.to_owned())
        .or_insert(Interface::new(interface.to_owned()));
    interface_a_mettre_a_jour.durees.insert(Local::now(), duree);
    return duree;
}

/// Calculer la durée moyenne des requêtes ICMP des interfaces et retirer les valeurs les plus obsolètes.
pub fn calculer_duree_moyenne(interfaces: &mut Interfaces) {

        for (_interface, details_interface) in &mut interfaces.liste_interfaces {
        let mut clefs_a_retirer = HashSet::new();
        let mut somme_durees = Duration::new(0,0);

       
        for (&clef,  valeur) in &mut  details_interface.durees {
            if Local::now().signed_duration_since(clef) > chrono::Duration::minutes(15){
                clefs_a_retirer.insert(clef);
            }
            somme_durees = somme_durees + *valeur;
        }
        details_interface.duree_moyenne = somme_durees/details_interface.durees.len() as u32;

        //Retirer les valeurs agées de 15 minutes ou plus
        for clef in clefs_a_retirer {
            details_interface.durees.remove(&clef);
        }
    }
}

/// Lister les routes par défaut.
pub fn lister_routes() -> HashMap<String, Route> {
    let mut liste_routes = Command::new("ip");
    liste_routes.arg("route").arg("show").arg("default");

    let routes = liste_routes.output().expect("process failed to execute");
    let routes_groupees = String::from_utf8(routes.stdout).unwrap();

    let mut routes = HashMap::new();

    for route in routes_groupees.split("\n") {
        if route != "" {
            let regex = Regex::new(r"^default .* dev (.*) proto .*").unwrap();
            for cap in regex.captures_iter(route) {
                routes.insert(
                    cap[1].to_owned(),
                    Route::new(cap[1].to_owned(), route.trim().to_owned(), None),
                );
            }
        }
    }
    return routes;
}
