# syscheck

Linux sistem sağlık kontrol aracı. `journalctl` loglarını parse ederek servisleri hata/uyarı sayısına göre renkli bir tabloda özetler.

## Kurulum

```bash
cargo build --release
# Binary: target/release/syscheck
```

## Kullanım

```bash
syscheck                              # Mevcut boot, ilk 20 servis
syscheck -n 10                        # İlk 10 servis
syscheck -n 0                         # Tüm servisler
syscheck -b -1                        # Tüm boot'ları dahil et
syscheck -v                           # Verbose: hatalı servislerin son loglarını göster

# Zaman filtresi
syscheck --since '1 hour ago'
syscheck --since 'yesterday'
syscheck --since '2024-01-15 10:00' --until '2024-01-15 12:00'

# Servis filtresi (substring, büyük/küçük harf duyarsız)
syscheck nginx
syscheck nginx.service

# Stdin (pipe) modu
journalctl -o json | syscheck --stdin
```

## Çıktı

```
SERVICE                               CRIT       ERR      WARN    NOTICE     TOTAL
nginx.service                            -         3         1        12        16
sshd.service                             1         -         -         4         5
systemd-resolved.service                 -         -         2         8        10
...

Özet:  47 servis tarandı, 3 sorunlu
```

Renk kodları:
- **Kırmızı kalın** — Critical hata var (priority 0–2)
- **Sarı kalın** — Error var (priority 3)
- **Sarı** — Warning var (priority 4)
- Normal — Sorunsuz (Ok)

Verbose modda (`-v` veya servis filtresi aktifken) her servisin son log satırları zaman damgasıyla birlikte gösterilir.

## Önkoşullar

- Linux (systemd/journald)
- Rust 1.85+

## Mimari

```
journalctl -o json  →  parse  →  group/sort  →  display
```

| Dosya | Sorumluluk |
|-------|-----------|
| `main.rs` | CLI (clap), journalctl çalıştırma, stdin okuma |
| `journal.rs` | `JournalEntry` struct, NDJSON parse |
| `summary.rs` | `ServiceSummary`, `HealthStatus`, gruplama ve sıralama |
| `display.rs` | Renkli tablo çıktısı |

## Bağımlılıklar

- [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json) — JSON parse
- [colored](https://crates.io/crates/colored) — Terminal renklendirme
- [clap](https://crates.io/crates/clap) — CLI arg parsing
- [anyhow](https://crates.io/crates/anyhow) — Hata yönetimi
