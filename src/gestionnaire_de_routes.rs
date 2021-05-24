use chrono::{DateTime, Local};
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::vec::Vec;

//dbg!(routes_groupees.clone());

#[derive(Debug, Clone)]
pub struct Route {
    pub interface: String,
    pub route: String,
    pub metrique: Option<i32>,
    pub note: Option<f32>,
}

impl Route {
    pub fn new(interface: String, route: String, metrique: Option<i32>, note: Option<f32>) -> Self {
        Self {
            interface,
            route,
            metrique,
            note,
        }
    }
}

pub struct Interface {
    pub nom: String,
    pub durees: BTreeMap<DateTime<Local>, Option<Duration>>,
    pub duree_moyenne: Option<Duration>,
}

impl Interface {
    pub fn new(nom: String) -> Self {
        Self {
            nom,
            durees: BTreeMap::new(),
            duree_moyenne: None,
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
pub fn tester_route(interface: &String, interfaces: &mut Interfaces) -> Option<Duration> {
    let commande = Command::new("ping")
        .arg("-c 1")
        .arg("-w 5")
        .arg(format!("{}{}", "-I", interface))
        .arg("1.1.1.1")
        .stdout(Stdio::piped())
        .output()
        .unwrap();

    let resultat_commande = String::from_utf8(commande.stdout).unwrap();

    let mut duree: Option<Duration> = None;
    let regex = Regex::new(r"icmp_seq=1 ttl=[0-9]{1,100} time=([0-9.]{1,100}) ms").unwrap();
    for element in regex.captures_iter(&resultat_commande) {
        match element[1].parse::<f32>() {
            Ok(duree_ms) => {
                duree = Some(Duration::from_micros((duree_ms * 1000.0) as u64));
            }
            Err(e) => println!("Erreur tester_route {}", e),
        }
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
        let mut somme_durees = Duration::new(0, 0);

        for (&clef, valeur) in &mut details_interface.durees {
            if Local::now().signed_duration_since(clef) > chrono::Duration::minutes(15) {
                clefs_a_retirer.insert(clef);
            }
            match *valeur {
                Some(x) => somme_durees = somme_durees + x,
                None => somme_durees = somme_durees + Duration::from_secs(5),
            }
        }
        details_interface.duree_moyenne =
            Some(somme_durees / details_interface.durees.len() as u32);

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
                    Route::new(cap[1].to_owned(), route.trim().to_owned(), None, None),
                );
            }
        }
    }
    return routes;
}

///Trier les routes.
pub fn trier_routes(
    interface_privilegiee: &str,
    routes: HashMap<String, Route>,
    interfaces: &mut Interfaces,
) -> Vec<Route> {
    let mut routes_triees = Vec::new();

    for (interface, mut route) in routes {
        let interface_trouvee;

        //Vérifier que l'interface est listée.
        if let Some(i) = interfaces.liste_interfaces.get(&interface) {
            interface_trouvee = i;

            //Vérifier que l'interface a une note
            if let Some(duree_moyenne) = interface_trouvee.duree_moyenne {
                route.note = Some(100.0 / duree_moyenne.as_millis() as f32);

                //Si l'interface est l'interface est privilégiée, augmenter la note
                if interface_trouvee.nom == interface_privilegiee {
                    route.note = Some(route.note.unwrap() * 2.0);
                }
            }
        }
        routes_triees.push(route);
    }

    routes_triees.sort_by(|a, b| b.note.partial_cmp(&a.note).unwrap());

    //Attribuer les métriques
    let mut metrique = 100;
    for mut route in &mut routes_triees {
      route.metrique = Some(metrique);
        metrique = metrique+1;
    }

    return routes_triees;
}
