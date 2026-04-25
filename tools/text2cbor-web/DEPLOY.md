# Déploiement text2cbor SaaS en production

## Prérequis

- Rust 1.70+
- text2cbor (binaire dans PATH)
- nginx ou Caddy (proxy inverse + TLS)
- Accès SMTP (pour envoi factures — optionnel)

## 1. Build

```bash
cd tools/text2cbor-web
cargo build --release
sudo cp target/release/text2cbor-web /opt/text2cbor-saas/
```

## 2. Service systemd

```ini
# /etc/systemd/system/text2cbor-saas.service
[Unit]
Description=text2cbor SaaS — HTML to CBOR-Web conversion
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/text2cbor-saas
ExecStart=/opt/text2cbor-saas/text2cbor-web
Restart=always
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now text2cbor-saas
```

## 3. Reverse proxy nginx

```nginx
# /etc/nginx/sites-available/text2cbor
server {
    listen 443 ssl http2;
    server_name text2cbor.com;

    ssl_certificate     /etc/letsencrypt/live/text2cbor.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/text2cbor.com/privkey.pem;

    client_max_body_size 200M;

    location / {
        proxy_pass http://127.0.0.1:3003;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 4. TLS avec Let's Encrypt

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d text2cbor.com
```

## 5. Sendmail (emails de facture)

```bash
sudo apt install msmtp-mta
# Configurer /etc/msmtprc avec les credentials SMTP
```

## 6. Vérification

```bash
curl http://127.0.0.1:3003/api/status
# → {"status":"ok","version":"0.1.0","uptime_secs":120,"users":3}
```

## Maintenance

```bash
# Logs
journalctl -u text2cbor-saas -f

# Data
ls -la /opt/text2cbor-saas/data/
# users.json, invoices.json, invoices/*.html

# Backup
tar czf backup-saas-$(date +%Y%m%d).tar.gz /opt/text2cbor-saas/data/
```
