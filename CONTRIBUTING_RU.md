# Участие в разработке Shadow Ghost

## Быстрый старт

```bash
# 1. Клонируйте репозиторий
git clone <repo-url>
cd shadowghost

# 2. Установите flutter_rust_bridge_codegen глобально (обязательно!)
cargo install flutter_rust_bridge_codegen

# 3. Установите зависимости Flutter
flutter pub get

# 4. Сгенерируйте bridge код
flutter_rust_bridge_codegen generate

# 5. Запустите приложение
flutter run
```

## Предварительные требования

Перед началом разработки убедитесь, что у вас установлены:

- **Rust** (последняя стабильная версия)
- **Flutter SDK** (последняя стабильная версия)
- **flutter_rust_bridge_codegen** CLI инструмент (см. установку ниже)

### Установка flutter_rust_bridge_codegen

**Этот шаг обязателен перед использованием любых команд генерации bridge:**

```bash
# Установите CLI инструмент глобально
cargo install flutter_rust_bridge_codegen

# Проверьте установку
flutter_rust_bridge_codegen --version
```

## Технологический стек

- **Backend**: Rust
- **Frontend**: Flutter
- **Bridge**: Flutter Rust Bridge v2

## Структура проекта

```
shadowghost/
├── lib/                          # Flutter/Dart код
│   ├── bridge_generated/         # Сгенерированный bridge код (НЕ РЕДАКТИРОВАТЬ)
│   └── main.dart                # Точка входа Flutter приложения
├── rust/                        # Rust код
│   ├── src/
│   │   ├── lib.rs              # Точка входа Rust библиотеки
│   │   └── api/                # Экспортируемые функции для Flutter
│   └── Cargo.toml
├── flutter_rust_bridge.yaml    # Конфигурация bridge
└── pubspec.yaml
```

## Рабочий процесс разработки

### Генерация Bridge кода

```bash
# Сгенерировать bridge код (после любых изменений в Rust API)
flutter_rust_bridge_codegen generate

# Очистить и перегенерировать при необходимости
rm -rf lib/bridge_generated
flutter_rust_bridge_codegen generate

# Полный цикл разработки
flutter pub get
flutter_rust_bridge_codegen generate
flutter run
```

### Добавление новых Rust функций

1. Добавьте функцию в соответствующий файл `rust/src/api/*.rs` с аннотацией `#[frb]`
2. Экспортируйте модуль в `rust/src/lib.rs`
3. **Обязательно выполните** `flutter_rust_bridge_codegen generate`
4. Используйте сгенерированный Dart код из `lib/bridge_generated/`

Пример:

```rust
// rust/src/api/contacts.rs
use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn get_contacts() -> Result<Vec<String>, String> {
    // Ваша реализация
    Ok(vec!["Контакт 1".to_string()])
}
```

## Процесс создания Pull Request

1. **Создайте форк и ветку**

   ```bash
   git checkout -b feature/your-feature
   ```

2. **Внесите изменения**

   - Отредактируйте Rust код в `rust/src/`
   - **Обязательно выполните**: `flutter_rust_bridge_codegen generate`
   - Отредактируйте Flutter код в `lib/`

3. **Тестирование**

   ```bash
   # Тесты Rust
   cargo test

   # Тесты Flutter
   flutter test

   # Форматирование кода
   cargo fmt
   flutter format .
   ```

4. **Формат коммитов**

   ```bash
   git commit -m "feat: добавить голосовые вызовы"
   git commit -m "fix: исправить таймаут подключения"
   git commit -m "docs: обновить документацию API"
   ```

5. **Чеклист для PR**
   - [ ] CLI инструмент `flutter_rust_bridge_codegen` установлен
   - [ ] Bridge код перегенерирован (`flutter_rust_bridge_codegen generate`)
   - [ ] Тесты Rust прошли (`cargo test`)
   - [ ] Тесты Flutter прошли (`flutter test`)
   - [ ] Код отформатирован (`cargo fmt` + `flutter format .`)
   - [ ] Сгенерированные файлы не редактировались вручную

## Рекомендации по Flutter Rust Bridge

### ДЕЛАТЬ

- ✅ Установите `flutter_rust_bridge_codegen` глобально перед разработкой
- ✅ Размещайте экспортируемые функции в модулях `rust/src/api/`
- ✅ Используйте аннотации `#[frb]` для экспортируемых функций
- ✅ Используйте `#[frb(sync)]` для синхронных функций
- ✅ Используйте `Result<T, String>` для обработки ошибок
- ✅ **Всегда** перегенерируйте bridge код после изменений в Rust API

### НЕ ДЕЛАТЬ

- ❌ Не редактируйте файлы в `lib/bridge_generated/` вручную
- ❌ Не пропускайте перегенерацию bridge после изменений API
- ❌ Не делайте коммит без выполнения `flutter_rust_bridge_codegen generate`
- ❌ Не используйте команды генерации bridge без предварительной установки CLI инструмента

## Частые проблемы

### "flutter_rust_bridge_codegen: command not found"

```bash
# Решение: Установите CLI инструмент глобально
cargo install flutter_rust_bridge_codegen

# Проверьте установку
which flutter_rust_bridge_codegen
```

### Ошибки генерации Bridge

```bash
# Очистите и перегенерируйте
rm -rf lib/bridge_generated
flutter clean
flutter pub get
flutter_rust_bridge_codegen generate
```

### Отсутствующие типы в Dart

- Убедитесь, что функции Rust имеют аннотации `#[frb]`
- Проверьте, что модули правильно экспортированы в `rust/src/lib.rs`
- Проверьте конфигурацию `flutter_rust_bridge.yaml`
- Перегенерируйте bridge код

## Рекомендации по безопасности

- **Всегда** валидируйте внешние входные данные в Rust и Dart
- Используйте **безопасные настройки по умолчанию** в криптографических функциях
- **Тестируйте** границы безопасности bridge
- Сообщайте о проблемах безопасности конфиденциально

## Архитектурные заметки

Проект использует Flutter Rust Bridge v2 для бесшовной интеграции:

- **Ручная генерация кода** из Rust в Dart через CLI инструмент
- **Типобезопасность** через границы языков
- **Поддержка асинхронности** для неблокирующих операций
- **Обработка ошибок на основе Result** для надежного распространения ошибок

## Лицензия

Участвуя в проекте, вы соглашаетесь с тем, что ваши вклады будут лицензированы под CC BY-NC-SA 4.0.
