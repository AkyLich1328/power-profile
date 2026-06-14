Умная система смены профилей производительности батареи на линукс

-----

При работе от зарядки - стандартно максимальная производительность

При работе от батареи - стандартно экономия энергии

Если температура процессора > 85 или процент батареи < 25 - экономия энергии

И еще отключен лимит зарядки, для того что бы его включить нужно в конфиге поменять
```
enable_charge_limit: true,
charge_limit: "Число при котором будет прекращаться зарядка, стандартно 85",
```

Конфиг файл находится в : ```/etc/power-profile/config.json```

"UPD"
Была добавленна возможность отключения TurboBoost для экономии заряда батареи
А так же смена яркости монитора в зависимости от профиля питания
Все настраивается в конфиг файле

<img width="1598" height="900" alt="image" src="https://github.com/user-attachments/assets/b8f745f1-f3cc-4eef-bccc-64abee753f70" />


Зависимости у программы:
```
sudo pacman -S power-profiles-daemon
sudo systemctl enable --now power-profiles-daemon
```

Для автозапуска с системой нужно:

перенести power-profile в /usr/local/bin/
```
mv power-profile /usr/local/bin/
```

даем права на запуск

```
sudo chmod +x /usr/local/bin/power-profile
```

Дальше создаем systemd сервис
```
sudo nano /etc/systemd/system/power-profile.service
```
Вставляем в конфиг 
```
[Unit]
Description=Power Profile Manager (Rust)
After=multi-user.target

[Service]
Type=simple
ExecStart=/usr/local/bin/power-profile auto
Restart=always
RestartSec=5

#root (для /sys и powerprofilesctl)
User=root

[Install]
WantedBy=multi-user.target
```

После этого включаем сервис
```
sudo systemctl daemon-reload

sudo systemctl enable power-profile.service

sudo systemctl start power-profile.service
```

Проверка статуса
```
systemctl status power-profile.service
```
