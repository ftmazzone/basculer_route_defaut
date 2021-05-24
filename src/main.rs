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
   
        }
        gestionnaire_de_routes::calculer_duree_moyenne(&mut interfaces);
        gestionnaire_de_routes::trier_routes(INTERFACE_PRIVILEGIEE,  routes, &mut interfaces);
  
        for (interface, route) in &mut routes {
            println!(
                "Interface : '{}' Durée : {:?} Note : {:?} Route : '{}'",
                interface,
                duree.unwrap_or(Duration::new(0, 0)),
                route.note
                route.route
            );
        }

        thread::sleep(Duration::from_secs(5));

        n = n + 1;
    }
}
