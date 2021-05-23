``` bash
while inotifywait -r -e ./basculer_route_defaut/src; do
cargo run
done
```

``` bash
rsync -r ./basculer_route_defaut/src/ [utilisateur]@[nom]:~/basculer_route_defaut/src
```