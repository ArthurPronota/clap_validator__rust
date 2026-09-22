# Механизм проверки значения аргументов CLAP

## Что такое validator в clap

**Validator** — это механизм **проверки** значения аргумента **после парсинга**, но **до** того, как оно попадёт в поле структуры. Если проверка не прошла — `clap` выдаёт **ошибку** и **завершает программу** (или возвращает `Err` через `try_parse`).

В `clap` 4 validator задаётся через **`value_parser`** — это функция или макрос, который:

1. **Парсит** строку в нужный тип.
2. **Проверяет** значение.
3. Возвращает `Result<T, E>`:
   - `Ok(value)` — успех;
   - `Err(message)` — ошибка с **понятным текстом**.

## Три способа задать validator в примере

### 1. `value_parser!(u8).range(0..=100)` — встроенный парсер + диапазон

```rust
#[arg(
    short = 't', 
    long = "threshold", 
    value_parser = value_parser!(u8).range(0..=100)
)]
threshold: u8,
```

- **`value_parser!(u8)`** — макрос, создающий **встроенный** парсер для `u8`.
- **`.range(0..=100)`** — **сужает** допустимый диапазон до `0..=100`.
- **Без** `.range` — принимал бы **весь** диапазон `u8` (`0..=255`).

**Что делает:**
- парсит строку в `u8`;
- проверяет, что значение **в диапазоне** `0..=100`;
- если **вне** диапазона — ошибка.

**Пример:**

```bash
$ cargo run -- -p abc.file -t 100 --config Cargo.toml -e abg@m.com
# ✅ Ok

$ cargo run -- -p abc.file -t 200 --config Cargo.toml -e abg@m.com
# ❌ error: invalid value '200' for '--threshold <THRESHOLD>':
#    200 is not in 0..=100

$ cargo run -- -p abc.file -t abc --config Cargo.toml -e abg@m.com
# ❌ error: invalid value 'abc' for '--threshold <THRESHOLD>':
#    invalid digit found in string
```

### 2. `parse_config` — кастомный validator для `PathBuf`

```rust
#[arg(short = 'c', long = "config", value_parser = parse_config)]
config: PathBuf,
```

```rust
fn parse_config(s: &str) -> Result<PathBuf, String> {
    match PathBuf::from_str(s) {
        Ok(p) => {
            if p.is_file() {
                Ok(p)                          // ✅ файл существует
            } else {
                Err(format!("Not found file {:?}", p))   // ❌ файла нет
            }
        },
        Err(err) => Err(format!("{}", err)),       // ❌ невалидный путь
    }
}
```

**Что делает:**

1. **Парсит** строку в `PathBuf` (`PathBuf::from_str`).
2. Проверяет, что **файл существует** (`p.is_file()`).
3. Если **нет** — возвращает `Err` с сообщением.

**Пример:**

```bash
$ cargo run -- -p abc.file -t 50 --config Cargo.toml -e abg@m.com
# ✅ Ok (Cargo.toml существует)

$ cargo run -- -p abc.file -t 50 --config missing.toml -e abg@m.com
# ❌ error: invalid value 'missing.toml' for '--config <CONFIG>':
#    Not found file "missing.toml"
```

**Особенности:**

- `PathBuf::from_str` **почти всегда** возвращает `Ok` — **любая** строка валидна как путь.
- **Реальная** проверка — `p.is_file()`.
- Ошибка — **строка**, которую `clap` покажет пользователю.

### 3. `parse_email` — кастомный validator для `String`

```rust
#[arg(short = 'e', long = "email", value_parser = parse_email)]
email: String,
```

```rust
fn parse_email(e: &str) -> Result<String, String> {
    match e.split_once('@') {
        Some((user, domain)) if !user.is_empty() && domain.contains('.') => Ok(e.to_string()),
        _ => Err(format!("Invalid email: {e}"))
    }
}
```

**Что делает:**

1. Разбивает строку по **первому** `@` (`split_once`).
2. Проверяет:
   - `user` **не пустой**;
   - `domain` содержит **точку**.
3. Если **всё ок** — возвращает строку.
4. Иначе — `Err`.

**Пример:**

```bash
$ cargo run -- -p abc.file -t 50 --config Cargo.toml -e abg@m.com
# ✅ Ok

$ cargo run -- -p abc.file -t 50 --config Cargo.toml -e invalid
# ❌ error: invalid value 'invalid' for '--email <EMAIL>':
#    Invalid email: invalid

$ cargo run -- -p abc.file -t 50 --config Cargo.toml -e @m.com
# ❌ error: invalid value '@m.com' for '--email <EMAIL>':
#    Invalid email: @m.com

$ cargo run -- -p abc.file -t 50 --config Cargo.toml -e user@localhost
# ❌ error: invalid value 'user@localhost' for '--email <EMAIL>':
#    Invalid email: user@localhost
```

## Сигнатура кастомного validator

```rust
fn validator(s: &str) -> Result<T, E>
where
    E: Into<Box<dyn Error + Send + Sync>>,
```

- **`s: &str`** — **сырая** строка из командной строки.
- **`Result<T, E>`** — либо успех, либо ошибка.
- **`E`** — **любой** тип, который можно преобразовать в `Box<dyn Error>`:
  - `String` — самый простой;
  - `&str`;
  - пользовательский тип с `impl Display`.

## Разбор полного примера

```rust
use clap::{Parser, value_parser};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Parser)]
struct CliArgs {
    /// Path to scan for cleanup
    #[arg(short = 'p', long = "path")]
    path: PathBuf,

    /// Control threshold
    #[arg(
        short = 't', 
        long = "threshold", 
        value_parser = value_parser!(u8).range(0..=100)
    )]
    threshold: u8,

    /// Config file
    #[arg(short = 'c', long = "config", value_parser = parse_config)]
    config: PathBuf,

    /// Email
    #[arg(short = 'e', long = "email", value_parser = parse_email)]
    email: String,
}
```

### Что происходит при запуске

```bash
$ cargo run -- -p abc.file -t 100 --config Cargo.toml -e abg@m.com
```

1. **`-p abc.file`** — `PathBuf`, **без** validator → любая строка валидна.
2. **`-t 100`** — `u8` + `range(0..=100)`:
   - парсится в `100`;
   - `100` в диапазоне `0..=100` → ✅.
3. **`--config Cargo.toml`** — `parse_config`:
   - `Cargo.toml` → `PathBuf`;
   - файл **существует** → ✅.
4. **`-e abg@m.com`** — `parse_email`:
   - `abg@m.com` → `split_once('@')` → `("abg", "m.com")`;
   - `user` не пустой, `domain` содержит `.` → ✅.

**Вывод:**

```
CliArgs {
    path: "abc.file",
    threshold: 100,
    config: "Cargo.toml",
    email: "abg@m.com",
}
```

## Ошибки при невалидных данных

### `-t 200` (вне диапазона)

```
error: invalid value '200' for '--threshold <THRESHOLD>':
  200 is not in 0..=100

For more information, try '--help'.
```

### `--config missing.toml` (файла нет)

```
error: invalid value 'missing.toml' for '--config <CONFIG>':
  Not found file "missing.toml"

For more information, try '--help'.
```

### `-e invalid` (не email)

```
error: invalid value 'invalid' for '--email <EMAIL>':
  Invalid email: invalid

For more information, try '--help'.
```

## Полезные встроенные парсеры

| Парсер | Что делает |
|---|---|
| `value_parser!(u8)` | Парсит `u8` (0..=255) |
| `value_parser!(i32)` | Парсит `i32` |
| `value_parser!(f64)` | Парсит `f64` |
| `value_parser!(bool)` | Парсит `true`/`false` |
| `value_parser!(PathBuf)` | Парсит путь |
| `value_parser!(IpAddr)` | Парсит IP-адрес |
| `value_parser!(SocketAddr)` | Парсит `host:port` |
| `value_parser!(u8).range(0..=100)` | `u8` + диапазон |
| `value_parser!(i32).range(1..)` | `i32` + диапазон |
| `value_parser!(f64).range(0.0..1.0)` | `f64` + диапазон |

## Кастомный validator с `FromStr`

Если у вас есть тип с `impl FromStr`:

```rust
#[derive(Debug, Clone)]
struct Email(String);

impl FromStr for Email {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('@') {
            Some((user, domain)) if !user.is_empty() && domain.contains('.') => {
                Ok(Email(s.to_string()))
            },
            _ => Err(format!("Invalid email: {s}")),
        }
    }
}

#[derive(Parser)]
struct CliArgs {
    #[arg(short = 'e', long = "email")]
    email: Email,   // ← clap сам вызовет FromStr
}
```

`clap` **автоматически** использует `FromStr`, если тип его реализует.

## Сводная таблица

| Способ | Синтаксис | Когда использовать |
|---|---|---|
| Встроенный парсер | `value_parser!(u8)` | Простые типы |
| Диапазон | `value_parser!(u8).range(0..=100)` | Числа с ограничением |
| Кастомная функция | `value_parser = parse_email` | Сложная логика |
| `FromStr` | `email: Email` | Тип с `impl FromStr` |

## Итог

- **Validator** в `clap` — это **`value_parser`**, который **парсит** и **проверяет** значение.
- **Три способа** задать: встроенный парсер, кастомная функция, `FromStr`.
- **Встроенный:** `value_parser!(u8).range(0..=100)` — парсит + проверяет диапазон.
- **Кастомный:** `fn parse_config(s: &str) -> Result<T, E>` — любая логика.
- **Возвращает** `Result<T, E>` — `Ok` или `Err` с **понятным** сообщением.
- **`clap`** сам выводит ошибку и **завершает** программу при `Err`.
- **В вашем примере:**
  - `-t` — `u8` в диапазоне `0..=100`;
  - `-c` — файл **должен существовать**;
  - `-e` — email **должен** содержать `@` и точку в домене.
- **Правило:** используйте **встроенные** парсеры, где возможно; **кастомные** — для специфичной логики.
