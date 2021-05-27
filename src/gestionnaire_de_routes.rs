use chrono::{DateTime, Local};
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::vec::Vec;
use std::{fmt, thread, time::Duration, time::Instant};

use crate::utilitaire;
use utilitaire::FormateurOption;

static DUREE_ATTENTE_MAXIMUM_SECONDES: u64 = 5;
static DUREE_VERIFICATION_CONNECTIVITE_INTERFACES_SECONDES: u64 = 300;

//dbg!(routes_groupees.clone());

#[derive(Debug, Clone)]
pub struct Route {
    pub interface: String,
    pub metrique: Option<i32>,
    pub duree_moyenne: Option<Duration>,
    pub note: Option<f32>,
    pub metrique_desiree: Option<i32>,
    pub route: String,
}

impl Route {
    pub fn new(
        interface: String,
        route: String,
        metrique: Option<i32>,
        duree_moyenne: Option<Duration>,
        note: Option<f32>,
        metrique_desiree: Option<i32>,
    ) -> Self {
        Self {
            interface,
            route,
            metrique,
            duree_moyenne,
            note,
            metrique_desiree,
        }
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nom_interface = self.interface.to_owned();
        let metrique = self.metrique.formater();
        let note = self.note.formater();
        let duree_moyenne = self.duree_moyenne.formater();
        let metrique_desiree = self.metrique_desiree.formater();
        let details = self.route.to_owned();
        write!(f, "Interface : {} Métrique : {} Note : {} Durée moyenne : {} Métrique désirée : {} Détails : {}",
         nom_interface,metrique,note,duree_moyenne,metrique_desiree,details)
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

/// Tester pendant la dureée spécifiée les routes avant de réévaluer la meilleure route
/// Si aucune durée n'est spécifiée, la valeur par défaut est 300 secondes.
pub fn verifier_connectivite_interfaces(
    running: &Arc<AtomicBool>,
    routes: &HashMap<String, Route>,
    interfaces: &mut Interfaces,
    duree: Option<Duration>,
) {
    let duree_verification: Duration;
    match duree {
        Some(dv) => duree_verification = dv,
        None => {
            duree_verification =
                Duration::from_secs(DUREE_VERIFICATION_CONNECTIVITE_INTERFACES_SECONDES)
        }
    }

    let mut debut_test = Some(Instant::now());
    loop {
        for (interface, route) in routes {
            // Si la route actuelle n'est plus fonctionnelle (2 tests) : réévaluer la meilleure route sans attendre
            if None == verifier_connectivite_interface(&interface, interfaces)
                && route.metrique == Some(100)
                && None == verifier_connectivite_interface(&interface, interfaces)
            {
                debut_test = None;
                println!(
                    "L'interface par défaut principale n'est pas fonctionnelle. {}",
                    route
                );
            }
        }
        if None != debut_test
            && (Instant::now() - debut_test.unwrap()) < duree_verification
            && running.load(Ordering::SeqCst)
        {
            thread::sleep(Duration::from_secs(5));
        } else {
            break;
        }
    }
}

/// Vérifier que la c est fonctionnelle et calculer la durée d'une requête ICMP.
/// Si la route n'est pas fonctionnelle, la durée retournée est de 1000 secondes.
pub fn verifier_connectivite_interface(
    interface: &String,
    interfaces: &mut Interfaces,
) -> Option<Duration> {
    let commande = Command::new("ping")
        .arg("-c 1")
        .arg("-w 5")
        .arg(format!("{}{}", "-I", interface))
        .arg("1.1.1.1")
        .stdout(Stdio::piped())
        .output()
        .unwrap();

    if commande.stderr.len() != 0 {
        eprintln!(
            "verifier_connectivite_interface erreur : '{}'",
            String::from_utf8(commande.stderr).unwrap_or_default(),
        );
    }

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
    let duree_maximum = Duration::from_secs(DUREE_ATTENTE_MAXIMUM_SECONDES);

    for (_interface, details_interface) in &mut interfaces.liste_interfaces {
        let mut clefs_a_retirer = HashSet::new();
        let mut somme_durees = Duration::new(0, 0);

        let mut derniere_duree = None;

        for (&clef, valeur) in &mut details_interface.durees {
            if Local::now().signed_duration_since(clef) > chrono::Duration::minutes(15) {
                clefs_a_retirer.insert(clef);
            }
            if let Some(x) = *valeur {
                somme_durees = somme_durees + x;
                derniere_duree = Some(x);
            } else {
                somme_durees = somme_durees + duree_maximum;
                derniere_duree = None;
            }
        }

        if None != derniere_duree {
            details_interface.duree_moyenne =
                Some(somme_durees / details_interface.durees.len() as u32);
        } else {
            details_interface.duree_moyenne = Some(duree_maximum);
        }

        //Retirer les valeurs agées de 15 minutes ou plus
        for clef in clefs_a_retirer {
            details_interface.durees.remove(&clef);
        }
    }
}

/// Lister les routes par défaut.
pub fn lister_routes() -> HashMap<String, Route> {
    let commande = Command::new("ip")
        .arg("route")
        .arg("show")
        .arg("default")
        .stdout(Stdio::piped())
        .output()
        .unwrap();

    if commande.stderr.len() != 0 {
        eprintln!(
            "lister_routes erreur : '{}'",
            String::from_utf8(commande.stderr).unwrap_or_default(),
        );
    }

    let routes_groupees = String::from_utf8(commande.stdout).unwrap();

    let mut routes = HashMap::new();

    for route in routes_groupees.split("\n") {
        if route != "" {
            let regex =
                Regex::new(r"^default .* dev (.*) proto .* metric ([0-9]{1,10}).*").unwrap();
            for cap in regex.captures_iter(route) {
                let mut metrique = None;

                match cap[2].parse::<i32>() {
                    Ok(m) => metrique = Some(m),
                    Err(e) => println!("Erreur lister_routes {}", e),
                }

                routes.insert(
                    cap[1].to_owned(),
                    Route::new(
                        cap[1].to_owned(),
                        route.trim().to_owned(),
                        metrique,
                        None,
                        None,
                        None,
                    ),
                );
            }
        }
    }
    return routes;
}

///Trier les routes.
pub fn trier_routes(
    interface_privilegiee: String,
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
                route.duree_moyenne = interface_trouvee.duree_moyenne;

                //Si l'interface est l'interface est privilégiée, augmenter la note
                if interface_trouvee.nom == interface_privilegiee {
                    route.note = Some(route.note.unwrap() * 4.0);
                }
            }
        }
        routes_triees.push(route);
    }

    routes_triees.sort_by(|a, b| b.note.partial_cmp(&a.note).unwrap());

    //Attribuer les métriques
    let mut metrique_desiree = 100;
    for mut route in &mut routes_triees {
        route.metrique_desiree = Some(metrique_desiree);
        metrique_desiree = metrique_desiree + 1;
    }

    return routes_triees;
}

/// Reconfigurer les métriques pour chaque route si la valeur de la métrique désirée ne correspond pas à la métrique actuelle.
pub fn commuter_reseaux(routes: &[Route]) {
    let mut commutation_necessaire = false;

    for route in routes {
        if route.metrique != route.metrique_desiree {
            commutation_necessaire = true;
            break;
        }
    }

    if commutation_necessaire {
        for route in routes {
            let commande = Command::new("ip")
                .arg("route")
                .arg("delete")
                .arg("default")
                .stdout(Stdio::piped())
                .output()
                .unwrap();

            let mut erreur = String::new();
            if commande.stderr.len() != 0 {
                erreur = String::from_utf8(commande.stderr).unwrap_or_default();
            }

            println!("supprimer route {} {} ", erreur, route);
        }

        for route in routes {
            let regex = Regex::new(
                r"^default via ([0-9.]{7,15}) dev ([0-9a-z]{1,20}) proto .* metric [0-9]{1,10}.*",
            )
            .unwrap();
            for element in regex.captures_iter(&route.route[..]) {
                let adresse_passerelle = &element[1][..];

                let commande = Command::new("ip")
                    .arg("route")
                    .arg("add")
                    .arg("default")
                    .arg("via")
                    .arg(adresse_passerelle)
                    .arg("dev")
                    .arg(route.interface.to_owned())
                    .arg("proto")
                    .arg("dhcp")
                    .arg("metric")
                    .arg(route.metrique_desiree.unwrap_or(10000).to_string())
                    .stdout(Stdio::piped())
                    .output()
                    .unwrap();

                let mut erreur = String::new();
                if commande.stderr.len() != 0 {
                    erreur = String::from_utf8(commande.stderr).unwrap_or_default();
                }

                println!("ajouter route {} {} ", erreur, route);
            }
        }
    }
}
