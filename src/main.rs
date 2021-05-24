use simple_signal::{self, Signal};
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

    let mut n = 0;
    let mut interfaces = gestionnaire_de_routes::Interfaces::new();

    //Tant que les signaux 'INT' et 'TERM' ne sont pas reçus
    while running.load(Ordering::SeqCst) {
        let mut routes = gestionnaire_de_routes::lister_routes();

        // Tester pendant deux minutes les routes avant de réévaluer la meilleure route
        let mut i = 0;
        while i < 25 && running.load(Ordering::SeqCst) {
            for (interface, route) in &mut routes {
                // Si la route actuelle n'est plus fonctionnelle : réévaluer la meilleure route sans attendre
                if None == gestionnaire_de_routes::tester_route(interface, &mut interfaces)
                    && route.metrique == Some(100)
                {
                    i = 25;
                    println!(
                        "L'interface par défaut principale n'est pas fonctionnelle. {:?}",
                        route
                    );
                } else {
                    i = i + 1;
                }
            }
            thread::sleep(Duration::from_secs(5));
        }

        gestionnaire_de_routes::calculer_duree_moyenne(&mut interfaces);
        let routes_triees =
            gestionnaire_de_routes::trier_routes(INTERFACE_PRIVILEGIEE, routes, &mut interfaces);

        // for route in &routes_triees {
        //     let interface = interfaces.liste_interfaces.get(&route.interface);
        //     println!(
        //         "Interface : '{}' Métrique : '{:?}' Note : '{:?}' Durée moyenne : '{:?}' Métrique désirée : '{:?}' Route : '{}' ",
        //         route.interface, route.metrique, route.note, interface.unwrap().duree_moyenne,route.metrique_desiree,route.route
        //     );
        // }

        gestionnaire_de_routes::commuter_reseaux(&routes_triees);

        thread::sleep(Duration::from_secs(5));

        n = n + 1;
    }
}
