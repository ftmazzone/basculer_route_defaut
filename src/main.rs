use std::{thread, time};

mod gestionnaire_de_routes;

fn main() {
    let mut n = 0;
    let mut interfaces = gestionnaire_de_routes::Interfaces::new();

    while n < 100 {
            let mut routes = gestionnaire_de_routes::lister_routes();

        for (interface, route) in &mut routes {
            route.duree = Some(gestionnaire_de_routes::tester_route(
                interface,
                &mut interfaces,
            ));
            println!(
                "Interface : '{}' Durée : {:?} Route : '{}'",
                interface,
                route.duree.unwrap(),
                route.route
            );

            let details_interface = interfaces
            .liste_interfaces.entry(interface.to_owned())
            .or_insert(gestionnaire_de_routes::Interface::new(interface.to_owned()));

                for (date,duree) in &mut details_interface.durees{
                    println!("Durée : {} {}", date,duree.as_millis());
                }
            
        }
        thread::sleep(time::Duration::from_secs(5));

        n = n + 1;
    }
}
