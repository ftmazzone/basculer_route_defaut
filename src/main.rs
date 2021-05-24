use std::{thread, time::Duration};

mod gestionnaire_de_routes;
static INTERFACE_PRIVILEGIEE: &str = "eth0";

fn main() {
    let mut n = 0;
    let mut interfaces = gestionnaire_de_routes::Interfaces::new();

    while n < 100000 {
        let mut routes = gestionnaire_de_routes::lister_routes();

        for (interface, _route) in &mut routes {
            gestionnaire_de_routes::tester_route(interface, &mut interfaces);
        }
        gestionnaire_de_routes::calculer_duree_moyenne(&mut interfaces);
        let routes_triees =
            gestionnaire_de_routes::trier_routes(INTERFACE_PRIVILEGIEE, routes, &mut interfaces);

        for route in routes_triees {
            let interface = interfaces.liste_interfaces.get(&route.interface);
            println!(
                "Interface : '{}' Métrique : '{:?}' Note : '{:?}' Route : '{}' Durée moyenne : '{:?}'",
                route.interface, route.metrique, route.note, route.route,interface.unwrap().duree_moyenne
            );
        }

        thread::sleep(Duration::from_secs(5));

        n = n + 1;
    }
}
