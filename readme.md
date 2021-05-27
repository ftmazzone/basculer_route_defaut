```bash
#Pour créer un service
{
    echo '#/etc/systemd/system/basculer-route-defaut.service'
    echo '[Unit]'
    echo 'Description=Basculer automatiquement les routes par défaut en cas de panne'
    echo 'Documentation=https://127.0.0.1'
    echo 'After=network-online.target'
    echo 'Requires=network-online.target'
    echo ''
    echo '[Service]'
    echo 'Environment="INTERFACE_PRIVILEGIEE=eth1"'
    echo 'Type=simple'
    echo '#User='
    echo 'ExecStart=/usr/local/sbin/basculer_route_defaut'
    echo 'Restart=on-failure'
    echo 'RestartSec=60'
    echo 'StartLimitInterval=200'
    echo 'StartLimitBurst=3'
    echo 'SyslogIdentifier=basculer-route-defaut'
    echo ''
    echo '[Install]'
    echo 'WantedBy=multi-user.target'
 } > /etc/systemd/system/basculer-route-defaut.service'

systemctl enable basculer-route-defaut.service

```


``` bash
while inotifywait -r -e ./basculer_route_defaut/src; do
cargo run
done
```

``` bash
rsync -r ./basculer_route_defaut/src/ [utilisateur]@[nom]:~/basculer_route_defaut/src
```