Умная система смены профилей производительности батареи на линукс
-----
При работе от зарядки - максимальная производительность
При работе от батареи - средняя производительность
Если температура процессора > 85 или процент батареи < 25 - экономия энергии

Конфиг файл находится в : /etc/power-profile/config.json

<img width="1594" height="900" alt="image" src="https://github.com/user-attachments/assets/4e459930-b65c-4df5-93a9-fbebcd2bf775" />

Зависимости у программы:

sudo pacman -S power-profiles-daemon

sudo systemctl enable --now power-profiles-daemon


Для автозапуска с системой нужно:
перенести power-profile в /usr/local/bin/

mv power-profile /usr/local/bin/

Дальше создаем systemd сервис

sudo nano /etc/systemd/system/power-profile.service

Вставляем в конфиг 
/////////////
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
/////////////

После этого включаем сервис

sudo systemctl daemon-reload

sudo systemctl enable power-profile.service

sudo systemctl start power-profile.service

Проверка статуса

systemctl status power-profile.service
