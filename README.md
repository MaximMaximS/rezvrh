# rezvrh

Bakalari scraper, written in rust. Very early stage of development, but it seems to work.

## Usage

Create config file

```json
{
    "username": "username",
    "password": "password",
    "url": "https://bakalari.example.com"
}
```

Run the program

```bash
rezvrh -c config.json
```

This will create a `timetable.json` file in the current directory.

*This project is not affiliated with BAKALÁŘI software s.r.o.*
