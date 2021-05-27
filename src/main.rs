#[cfg(debug_assertions)]
macro_rules! journalisation_activee {
    () => {
        true
    };
}

#[cfg(not(debug_assertions))]
macro_rules! journalisation_activee {
    () => {
        false
    };
}

use gestionnaire_de_routes::{Interfaces, Route};
use simple_signal::{self, Signal};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

mod gestionnaire_de_routes;
mod utilitaire;

use utilitaire::FormateurOption;

static INTERFACE_PRIVILEGIEE: &str = "eth0";

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |signal_recu| {
        println!("Signal reçu : '{:?}'", signal_recu);
        r.store(false, Ordering::SeqCst);
    });

    //Vérifier quelle interface est privilégiée.
    let interface_privilegiee: String;
    match env::var("INTERFACE_PRIVILEGIEE") {
        Ok(valeur) => interface_privilegiee = valeur.to_string(),
        Err(e) => {
            if e != std::env::VarError::NotPresent {
                eprintln!("Interface privilégiée non reconnue : '{}'", e);
            }
            interface_privilegiee = String::from(INTERFACE_PRIVILEGIEE);
        }
    }
    println!("Interface privilégiée : {}", interface_privilegiee);

    let mut interfaces = Interfaces::new();

    //Tant que les signaux 'INT' et 'TERM' ne sont pas reçus
    while running.load(Ordering::SeqCst) {
        let routes = gestionnaire_de_routes::lister_routes();
        gestionnaire_de_routes::verifier_connectivite_interfaces(
            &running,
            &routes,
            &mut interfaces,
            None,
            // Some(Duration::from_secs(10)),
        );
        gestionnaire_de_routes::calculer_duree_moyenne(&mut interfaces);
        let routes_triees = gestionnaire_de_routes::trier_routes(
            interface_privilegiee.to_owned(),
            routes,
            &mut interfaces,
        );

        if journalisation_activee!() {
            afficher_routes(&routes_triees, &interfaces);
        }

        gestionnaire_de_routes::commuter_reseaux(&routes_triees);

        thread::sleep(Duration::from_secs(5));
    }
}

fn afficher_routes(routes: &Vec<Route>, interfaces: &Interfaces) {
    for route in routes {
        let interface = interfaces.liste_interfaces.get(&route.interface);

        let nom_interface = route.interface.to_owned();
        let metrique = route.metrique.formater();
        let note = route.note.formater();
        let duree_moyenne;
        match interface {
            Some(i) => duree_moyenne = i.duree_moyenne.formater(),
            None => duree_moyenne = String::new(),
        }
        let metrique_desiree = route.metrique_desiree.formater();
        let details = route.route.to_owned();

        dbg!(
            "Interface : '{}' Métrique : '{}' Note : '{}' Durée moyenne : '{}' Métrique désirée : '{}' Route : '{}' ",
            nom_interface, metrique, note, duree_moyenne,metrique_desiree,details
        );
    }
}
