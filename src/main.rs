use std::{thread, time::Duration};

mod gestionnaire_de_routes;
static INTERFACE_PRIVILEGIEE: &str = "eth0";

fn main() {
    let mut n = 0;
    let mut interfaces = gestionnaire_de_routes::Interfaces::new();

    while n < 100000 {
        let mut routes = gestionnaire_de_routes::lister_routes();

        for (interface, route) in &mut routes {
            let duree = gestionnaire_de_routes::tester_route(interface, &mut interfaces);
            println!(
                "Interface : '{}' Durée : {:?} Route : '{}'",
                interface,
                duree.unwrap_or(Duration::new(0, 0)),
                route.route
            );

            for (_interface, details_interface) in &mut interfaces.liste_interfaces {
                for (date, duree) in &mut details_interface.durees {
                    println!(
                        "Durée : {} {:?} {:?}",
                        date,
                        duree.unwrap_or(Duration::from_micros(0)),
                        details_interface
                            .duree_moyenne
                            .unwrap_or(Duration::from_micros(0))
                    );
                }
            }
        }
        gestionnaire_de_routes::trier_routes(INTERFACE_PRIVILEGIEE,  routes, &mut interfaces);
        gestionnaire_de_routes::calculer_duree_moyenne(&mut interfaces);
        thread::sleep(Duration::from_secs(5));

        n = n + 1;
    }
}
