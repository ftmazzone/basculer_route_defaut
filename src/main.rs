use simple_signal::{self, Signal};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

mod gestionnaire_de_routes;
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

    let mut interfaces = gestionnaire_de_routes::Interfaces::new();

    //Tant que les signaux 'INT' et 'TERM' ne sont pas reçus
    while running.load(Ordering::SeqCst) {
        let routes = gestionnaire_de_routes::lister_routes();
        gestionnaire_de_routes::verifier_connectivite_interfaces(
            &running,
            &routes,
            &mut interfaces,
        );
        gestionnaire_de_routes::calculer_duree_moyenne(&mut interfaces);
        let routes_triees = gestionnaire_de_routes::trier_routes(
            interface_privilegiee.to_owned(),
            routes,
            &mut interfaces,
        );

        for route in &routes_triees {
            let interface = interfaces.liste_interfaces.get(&route.interface);
            dbg!(
                "Interface : '{}' Métrique : '{:?}' Note : '{:?}' Durée moyenne : '{:?}' Métrique désirée : '{:?}' Route : '{}' ",
                &route.interface, route.metrique, route.note, interface.unwrap().duree_moyenne,route.metrique_desiree,&route.route
            );
        }

        gestionnaire_de_routes::commuter_reseaux(&routes_triees);

        thread::sleep(Duration::from_secs(5));
    }
}
