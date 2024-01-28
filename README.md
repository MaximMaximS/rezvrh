# rezvrh

Scraper na rozvrh hodin z Bakalářů.
Asi nebude zatím moc funkční.

## Použití

Vytvořte soubor `config.json` s následujícím obsahem:

```json
{
    "username": "username",
    "password": "password",
    "url": "https://bakalari.example.com"
}
```

Pak spusťte:

```bash
rezvrh -c config.json
```

Tím se vytvoří soubor `rozvrh.json` s rozvrhem.

*Tento projekt není spojen s firmou BAKALÁŘI software s.r.o.*
