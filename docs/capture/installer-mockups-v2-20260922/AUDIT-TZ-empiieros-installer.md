# Аудит + ТЗ: EmpiierOS / EMPERY WORLD · Installer (полный поток 01–10)

**Продукт:** EmpiierOS (eliteew-ts-installer, egui) + отдельный LUKS console (`state-unlock-agent`)  
**Бренд:** EMPERY WORLD / **EmpiierOS** (единообразно; правки опечаток EmpierOS / EMPIERY)  
**Дата:** 2026-09-22  
**Исходники:** `refs/01-original-…` … `refs/10-original-…` (egui ~1920×1034 / console 1280×800)  
**Целевые макеты:** `refs/01-mockup-…-1920x1080.png` … `refs/10-mockup-…-1920x1080.png` (строго **1920×1080**)  
**HTML-источники макетов:** `mockup-html/*.html`  
**Стек реализации:** egui (native installer) + console agent — **не** React/Tailwind. Токены hex и layout-constraints — для egui-имплементаторов.

---

# Этап 1. Структурный и визуальный аудит

## 1.1 Информационная архитектура (полный поток)

| # | Шаг | Экран | Роль |
|---|-----|-------|------|
| 01 | Welcome | Приветствие | Старт, статус сети, CTA «Начать» |
| 02 | Regional | Язык, пояс, время | Локаль + ручное местное время |
| 03 | Network | Сеть | Авто / Вручную / Wi‑Fi / Пропустить |
| 04 | Disk | Диск | Выбор внутреннего диска + red warning |
| 05 | State-key | Ключ диска | Auto-issue vs свой ключ (≥12, не PIN) |
| 06 | Confirm | Подтверждение | Typed `ERASE …` + summary → Установить |
| 07 | Progress | Установка | 5 фаз записи + % |
| 08 | Done | Готово | PIN + ключ диска + typed ЗАПИСАЛ |
| 09 | Failed | Ошибка | Ветка с Progress; retry / к диску |
| 10 | Boot LUKS | Console | Не egui; ввод LUKS recovery |

**User goal:** поставить EmpiierOS на внутренний диск, сохранить **два разных** секрета (PIN ≠ ключ диска), перезагрузиться в защищённый том.

**Вердикт IA as-is:** последовательность шагов верная и достаточная. Проблемы не в порядке экранов, а в **подаче на FHD**: крошечный centered modal («тамагочи»), конкурирующий glitch-фон, слабая связность шагов (нет step rail), путаница трёх секретов (PIN / ключ диска / ключ сети) размазана по copy без единой модели.

**Пропуски ранних шагов в старом ТЗ:** 01–05 теперь зафиксированы оригиналами; IA continuity = один wizard chrome на 01–08, Failed как error-state того же shell, 10 — sibling family (badge + type).

---

## 1.2 Визуальный UI (общие проблемы 01–09)

| Аспект | Оценка | Комментарий |
|--------|--------|-------------|
| Canvas | Не FHD | Оригиналы ~1920×1034; console 1280×800 |
| Сетка | «Тамагочи» | Tiny floating modal в море фона |
| Фон | Шум | Крупные RORSX/ERROR glyphs конкурируют с red warnings |
| Бренд | Дрейф | EmpiierOS ок в ряде мест; риск EmpierOS/EMPIERY |
| Иерархия | Слабая на Done | PIN и ключ диска слишком близки по весу |
| Stepper Progress | Ок по смыслу | Но labels/«Финал» артефакт; нет global rail |
| Контраст CTA | Ок | Primary cyan / secondary outline — сохранить |
| Console 10 | Слишком sparse на FHD | Нужен scale type + badge family |

---

## 1.3 UX — узкие места по экранам

1. **01 Welcome** — center island; сеть есть, но нет карты пути (что будет дальше).  
2. **02 Regional** — важное правило «интернет не подстраивает время» тонет в мелком subtitle.  
3. **03 Network** — tabs Авто|Вручную|Wi‑Fi|Пропустить ок; empty states manual/Wi‑Fi не показаны (нужны в логике).  
4. **04 Disk** — red warning центрирован мелко; один диск без list-density; Обновить рядом с Назад без иерархии.  
5. **05 State-key** — критично: «не код входа» есть, но нет явной модели PIN vs ключ диска vs ключ сети до Done.  
6. **06 Confirm** — typed confirm корректен; summary сжата в одну простыню.  
7. **07 Progress** — 5 фаз ок; % + «Сейчас» ок; нет mini-log / ощущения «работы».  
8. **08 Done** — секреты недостаточно разделены визуально; reboot lock красный — хорошо.  
9. **09 Failed** — действий два (к диску / повторить) — ок; нет кода ошибки / фазы.  
10. **10 LUKS** — clarify «Не PIN…» ок; на FHD выглядит крошечным пятном.

---

## 1.4 Accessibility & cognitive load

- Destructive (04/06) и secrets (08) требуют **максимального** контраста без декоративного шума.  
- Typed confirms (`ERASE …`, `ЗАПИСАЛ`/`ZAPISAL`) — хороший friction; поля должны быть крупными, monospace.  
- Три секрета: без явной легенды пользователь вводит PIN в LUKS console.  
- Step rail снижает «где я?» на 8-шаговом wizard.  
- Focus / keyboard: egui — primary CTA и confirm input в логическом tab-order.

**Итог аудита:** не менять порядок шагов; **пересобрать chrome + плотность + разделение секретов**.

---

# Этап 2. Рефакторинг и концепция

## 2.1 Единый shell (01–09 egui)

```
┌─ Topbar 72px: [logo] EMPERY WORLD / EmpiierOS · установка     Шаг N из 8 ─┐
├─ Step rail 44px: 1…8 (done / current / todo)                               │
├─ Main: content column ~1040px (left) + aside ~360–400px (context)          │
│    └─ Panel flex-fill: title → body → (footer actions pinned bottom)       │
└─ Background: near-black + subtle grid; glitch glyphs MUTED/removed         ┘
```

**Выбор системы:** left-aligned dense column + aside (не крошечный centered modal). Один chrome на все wizard-шаги.

## 2.2 Концепция редизайна

- Adult dense installer chrome для 1920×1080.  
- **Palette v2 cyan/metal** (v1 matrix-green archived → `refs/archive-green-v1/`).  
- Accent electric cyan `#2ec4c6` / bright CTA `#3ad7d9` на brushed-metal near-black `#0a0c0e`; soft glow `rgba(46,196,198,.35)`.  
- Primary CTA solid cyan-bright; secondary outline. Success/online = cyan (не green).  
- Destructive: red callout primary visual (`#ff3b4a`).  
- Done: PIN huge mono cyan; ключ диска отдельная карточка (другой border/tint).  
- Progress: 5 labels в порядке Проверка→Разметка→Том данных→Система→Загрузчик; % + фаза.  
- LUKS console: terminal-minimal, cyan badge EmpiierOS, type scale FHD.  
- Copy: везде **EmpiierOS** / **EMPERY WORLD**.  
- Layout shell без изменений: topbar + 8-step rail + content ~1040px + aside; glitch glyphs muted/off.

## 2.3 Токены (для egui) — palette v2 cyan/metal

| Token | Hex | Use |
|-------|-----|-----|
| bg page | `#0a0c0e` | canvas |
| bg panel | `#12161a` | content card |
| bg elevated / input | `#0e1216` | inputs, chips, rail |
| border subtle | `#1e262e` | panel / dividers |
| border focus | `#2ec4c6` | focus ring |
| accent primary | `#2ec4c6` | brand, status online, focus; glow `rgba(46,196,198,.35)` |
| accent bright | `#3ad7d9` | CTA / current step |
| text primary | `#e8eef2` | titles, body |
| text secondary | `#8b97a3` | lead, meta |
| text muted | `#5c6772` | hints, aside |
| danger | `#ff3b4a` | wipe / fail / lock (не green) |
| logo mark | radial cyan (not green) | brand orb |
| mono | JetBrains Mono / Consolas | paths, keys, typed confirm |

**Запрещено:** любые green accents (`#39ff5a`, `#4ecf6a`, `#12c94a`, matrix-green и т.п.). Success/online — только cyan.

Layout: content column **1040px**, aside **≤400px**, gutters **48px**, panel radius **10px**. Shared CSS: `mockup-html/tokens.css` (linked optionally; tokens also inlined in each HTML).

---

# Этап 3. ТЗ на изменения

Ниже — по экранам. Каждый блок: **UI / UX·Logic / Content·Copy**.

---

## 01 · Welcome

### UI Changes
1. Stable shell (topbar + rail + panel 1040 + aside).  
2. Badge `EMPIIEROS` + H1 «Установка системы».  
3. Network status chip с cyan точкой (кабель · ifname).  
4. Primary «Начать» справа внизу панели.  
5. Убрать glitch-фон; subtle grid only.

### UX / Logic Changes
1. «Начать» → шаг 02.  
2. Статус сети read-only preview (полный конфиг на 03); если нет линка — muted «сеть не обнаружена» (не блокировать старт).  
3. Rail шаг 1 current.

### Content / Copy Changes
| Было | Стало |
|------|--------|
| EMPIIEROS / Установка системы | **EMPIIEROS** / **Установка системы** (сохранить) |
| длинный абзац | короткий: диск + сеть; PIN и ключ диска — один раз |
| Сеть: кабель подключён · enp0s3 | **Сеть: кабель подключён · enp0s3** |
| Начать | **Начать** |
| EmpierOS (если где-то) | **EmpiierOS** |

---

## 02 · Regional

### UI Changes
1. Тот же shell; rail шаг 2.  
2. Сегмент Русский | English.  
3. Select пояса; дата/время в 2 колонки.  
4. Hint формата под полями.  
5. Назад / Далее.

### UX / Logic Changes
1. Интернет **не** синхронизирует часы — UI обязан это сказать (lead, не footnote).  
2. Валидация ГГГГ-ММ-ДД / ЧЧ:ММ; Далее disabled при ошибке.  
3. Язык сразу переключает UI strings установщика.

### Content / Copy Changes
| Было | Стало |
|------|--------|
| Язык, пояс и время | **Язык, пояс и время** |
| Укажите местное время. Интернет его не подстраивает. | сохранить как **lead** |
| Формат даты: ГГГГ-ММ-ДД… | сохранить |
| Назад / Далее | **Назад** / **Далее** |

---

## 03 · Network

### UI Changes
1. Shell + rail шаг 3.  
2. Tabs: **Авто | Вручную | Wi‑Fi | Пропустить**.  
3. Статус кабеля + выбранный интерфейс (selected card).  
4. Назад / Далее.

### UX / Logic Changes
1. Авто: DHCP на выбранном if; Далее enabled при link up **или** явном «продолжить без линка» (продуктово: сейчас — при подключённом кабеле).  
2. Вручную: поля IP/mask/gw/DNS (отдельное состояние UI).  
3. Wi‑Fi: список SSID + пароль.  
4. Пропустить: offline; настройки не переносятся.  
5. Настройки переносятся на установленную систему (кроме Пропустить).

### Content / Copy Changes
| Было | Стало |
|------|--------|
| Сеть | **Сеть** |
| Настройки сети перенесутся… | сохранить |
| кабель подключён · enp0s3 | сохранить |
| Сетевой интерфейс / enp0s3 · Ethernet · подключён | сохранить |
| Авто / Вручную / Wi-Fi / Пропустить | **Авто / Вручную / Wi‑Fi / Пропустить** |

---

## 04 · Disk

### UI Changes
1. Shell + rail шаг 4.  
2. Список дисков: path mono + size + model; selected = cyan border.  
3. Red callout: новая разметка / старые разделы исчезнут.  
4. Кнопки: Обновить | Назад (sec) · Продолжить (pri).

### UX / Logic Changes
1. Только внутренние диски (исключить носитель установщика).  
2. Обновить = rescan.  
3. Продолжить disabled без selection.  
4. Фактический wipe — только после typed confirm на 06.

### Content / Copy Changes
| Было | Стало |
|------|--------|
| Диск | **Диск** |
| Выберите внутренний диск… | сохранить |
| /dev/nvme0n1 · 512 GB · Example NVMe | path + size в карточке; model secondary |
| Будет создана новая разметка… | **в red callout** |
| Обновить / Назад / Продолжить | сохранить |

---

## 05 · State-key (ключ диска)

### UI Changes
1. Shell + rail шаг 5.  
2. Primary option card: «Выдать ключ автоматически».  
3. Secondary link: «Задать свой ключ» + правила.  
4. Aside: легенда трёх секретов (PIN / ключ диска / ключ сети).  
5. Назад / Продолжить.

### UX / Logic Changes
1. Default mode = auto; Продолжить валиден после выбора auto **или** валидного custom.  
2. Custom: ≥12 символов; reject если выглядит как 4-digit PIN.  
3. Ключ **не** показывают на этом шаге при auto — только на 08.  
4. Explicit: это **не** PIN входа.

### Content / Copy Changes
| Было | Стало |
|------|--------|
| Ключ диска | **Ключ диска** |
| …Это не код входа в систему. | **…Это не код входа в систему (не PIN).** |
| Выдать ключ автоматически | сохранить |
| Ключ сгенерируем сами и покажем один раз… | «…на экране Готово перед перезагрузкой» |
| Задать свой ключ | сохранить |
| От 12 символов. Не используйте четырёхзначный код входа. | сохранить |

---

## 06 · Confirm erase

### UI Changes
1. Shell + rail шаг 6.  
2. H1 «Подтверждение очистки диска».  
3. Red callout с `/dev/…`.  
4. Summary как dl-grid (язык / пояс / сеть / ключ).  
5. Typed confirm mono input; Назад / Установить.  
6. Aside «Перед установкой».

### UX / Logic Changes
1. Установить enabled **только** при exact match `ERASE <diskname>` (напр. `ERASE nvme0n1`).  
2. После клика — необратимый старт фазы 07.  
3. Назад → 05.

### Content / Copy Changes
| Было | Стало |
|------|--------|
| Подтверждение | **Подтверждение очистки диска** |
| Будет полностью очищен диск … | сохранить в red callout |
| Язык/сеть/ключ одной простынёй | **grid labels** |
| Введите ERASE … | **Введите `ERASE nvme0n1`, чтобы продолжить** |
| Назад / Установить | сохранить |
| EmpierOS | **EmpiierOS** |

---

## 07 · Progress

### UI Changes
1. Shell + rail шаг 7.  
2. Horizontal stepper 5 фаз с check / current / todo.  
3. Крупный **%** + «Сейчас: &lt;фаза&gt;» + progress bar.  
4. Mini-log (последние события).  
5. Aside: расшифровка фаз.

### UX / Logic Changes
1. Порядок фаз фиксирован: Проверка → Разметка → Том данных → Система → Загрузчик.  
2. % и current phase из backend; UI не выдумывает порядок.  
3. Power-loss warning в lead.  
4. Success → 08; hard fail → 09.

### Content / Copy Changes
| Было | Стало |
|------|--------|
| Установка | **Установка** |
| Не выключайте питание… | + «не извлекайте носитель» |
| Проверка / Разметка / Том данных / Система / Загрузчик | **строго этот порядок** (убрать артефакт «Финал») |
| 35% Сейчас: Система | **35%** · **Сейчас: Система** |

---

## 08 · Done (PIN + recovery)

### UI Changes
1. Shell + rail шаг 8.  
2. Две карточки: PIN (huge mono cyan) vs Ключ диска (длинный mono, другой tint).  
3. Typed `ЗАПИСАЛ` / `ZAPISAL`.  
4. Red lock until confirmed.  
5. Aside: куда что вводить (login vs LUKS console).

### UX / Logic Changes
1. Reboot **закрыт**, пока confirm string не совпал (case rules: ЗАПИСАЛ или ZAPISAL).  
2. Секреты показываются **один раз**; после reboot UI установщика их не хранит на экране.  
3. Нет кнопки «Копировать в буфер» по умолчанию на air-gapped installs (опционально later); визуально copy-friendly sizes обязательны.

### Content / Copy Changes
| Было | Стало |
|------|--------|
| Запишите PIN и ключ диска | сохранить (accent title) |
| Это разные значения | **усилить** в lead |
| PIN / Ключ диска (авто) | **PIN · вход** / **Ключ диска · LUKS recovery** |
| Чтобы перезагрузить, наберите ЗАПИСАЛ… | сохранить + kbd |
| Перезагрузка закрыта… | сохранить red lock |

---

## 09 · Failed

### UI Changes
1. Тот же shell; rail показывает фазу обрыва (напр. 7) + meta «Установка прервана».  
2. Red H1 «Установка не завершена».  
3. Error callout + meta grid (цель / фаза / код).  
4. К выбору диска (sec) / Повторить (pri).  
5. Aside «Что сделать».

### UX / Logic Changes
1. «К выбору диска» → шаг 04 (rescan).  
2. «Повторить» → retry той же операции, если busy снят.  
3. Лог на носителе (`install-fail.log` или актуальный path).  
4. Диск после fail **не** считать чистым — copy это говорит.

### Content / Copy Changes
| Было | Стало |
|------|--------|
| Установка не завершена | сохранить |
| Не удалось записать разметку: диск занят… | сохранить + код `DISK_BUSY` в meta |
| Технический журнал сохранён… | + имя файла + caution о состоянии диска |
| К выбору диска / Повторить | сохранить |

---

## 10 · Boot LUKS console (не egui)

### UI Changes
1. Canvas 1920×1080; left content block с FHD type scale.  
2. Cyan badge **EmpiierOS** + title «ключ восстановления диска».  
3. Clarify line.  
4. `>` + masked dots + caret.  
5. Footer: `state-unlock-agent · EMPERY WORLD`.  
6. Без glitch; без egui chrome/rail.

### UX / Logic Changes
1. Принимает **только** LUKS recovery / state key — не PIN, не Wi‑Fi.  
2. Masked input; Enter submits.  
3. Ошибки — краткая console-line (без egui modal).

### Content / Copy Changes
| Было | Стало |
|------|--------|
| EmpiierOS (badge) | **EmpiierOS** |
| ключ восстановления диска | сохранить |
| Не PIN. Не ключ сети. Только LUKS state. | **Не PIN. Не ключ сети. Только LUKS state тома данных.** |

---

## Критерии приёмки

- [ ] Все целевые PNG = **1920×1080** (ffprobe)  
- [ ] Нет «тамагочи»: topbar + rail + panel≥~1040px на 01–09  
- [ ] Glitch RORSX/ERROR отсутствует или contrast ≤ subtle grid  
- [ ] Бренд: только **EmpiierOS** / **EMPERY WORLD**  
- [ ] Destructive: red callout + typed ERASE на 06  
- [ ] Progress labels order fixed; % + current phase  
- [ ] Done: PIN и ключ визуально различны; reboot lock  
- [ ] Failed: два CTA + log mention  
- [ ] LUKS: badge + clarify + masked prompt, same family  
- [ ] Palette v2 cyan/metal; zero green accents in HTML/mockups
- [ ] ТЗ actionable для egui (без React deps)

---

## История

| Дата | |
|------|--|
| 2026-09-22 | Полный аудит 01–10 + ТЗ + FHD mockups |
| 2026-09-22 | Palette v2 cyan/metal; green mockups → `refs/archive-green-v1/` |

## Визуальный стандарт

Не centered tiny modal. Целевой тон: **плотный native installer** (Anaconda/Calamares density) в cyan/metal skin EMPERY WORLD (palette v2). Aside — краткий context, не tutorial-простыни.
