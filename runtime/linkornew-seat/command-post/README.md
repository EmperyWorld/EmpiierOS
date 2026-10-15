# LinkornEW Command Post units

Staff image only. FoFline packager does not install these.

## Статус (честно)

| Слой | Состояние |
|------|-----------|
| `plant-cpython.sh` | Сажает CPython 3.14 в `/usr/lib/linkornew/cpython` (Buildroot образа без python) |
| `cp-deck-serve` | Loopback `:4191`, static + proxy `/api`→`:8088` |
| `plant-deck/controller/sentinel` | Сажают дерево + site-packages на planted CPython |
| `cp-*-run.sh` | Реальный uvicorn, не `cp-guard` oneshot |
| Lab E2E (`/tmp` stage) | Deck 200, profile 200, sentinel register `sentinel-linkornew` healthy |
| Metal / staff USB | **Not MATCH** — нужен overlay с `LINKORNEW_COMMAND_POST=…` + токены |
| `cp-deck-stub.c` | Исторический placeholder, в Makefile; serve — рабочий путь |

Bind `0.0.0.0`/`::`/`*` запрещён. `generation_only=true` — режим продукта.

## Plant

```sh
# stage
bash /path/to/command-post/scripts/plant-linkornew-deck.sh /tmp/linkornew-stage

# или при assemble rootfs:
LINKORNEW_COMMAND_POST=/path/to/command-post \
  ./runtime/linkornew-seat/assemble-linkornew-rootfs.sh --overlay /path/to/root
```

Токены только в `var/lib/linkornew/command-post/{controller,sentinel}.env`.
