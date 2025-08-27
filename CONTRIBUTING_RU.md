# Участие в разработке Shadow Ghost

## Быстрый старт

```bash
# 1. Клонируйте репозиторий
git clone <repo-url>
cd shadowghost

# 2. Установите зависимости
flutter pub get

# 3. Запустите приложение (мост генерируется автоматически)
flutter run
```

Вот и всё! Мост Rust-Flutter генерируется автоматически при первом запуске.

## Технологический стек

- **Бэкенд**: Rust
- **Фронтенд**: Flutter

# Команды установки Flutter SDK

## Windows
```powershell
# Chocolatey
choco install flutter

# Scoop
scoop bucket add extras
scoop install flutter

# Git
git clone https://github.com/flutter/flutter.git -b stable
```

## macOS
```bash
# Homebrew
brew install --cask flutter

# Git
git clone https://github.com/flutter/flutter.git -b stable
export PATH="$PATH:`pwd`/flutter/bin"
```

## Linux
```bash
# Snap
sudo snap install flutter --classic

# Git
git clone https://github.com/flutter/flutter.git -b stable
export PATH="$PATH:`pwd`/flutter/bin"
```

## Настройка среды разработки

### Установка зависимостей

#### Rust (при необходимости)
с```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Android Studio
```bash
# Скачайте с https://developer.android.com/studio
# Установите Android SDK, Android SDK Command-line Tools, Android SDK Build-Tools
```

#### VS Code
```bash
# Расширения для Flutter
code --install-extension Dart-Code.flutter
code --install-extension Dart-Code.dart-code
```

### Проверка и настройка
```bash
# Проверьте установку и зависимости
flutter doctor

# Проверьте версию
flutter --version

# Примите лицензии Android SDK
flutter doctor --android-licenses

# Создайте новый тестовый проект
flutter create test_app
cd test_app
flutter run
```

### Настройка устройств
```bash
# Покажите доступные устройства
flutter devices

# Включите поддержку веб
flutter config --enable-web

# Включите поддержку десктопных приложений
flutter config --enable-windows-desktop
flutter config --enable-macos-desktop  
flutter config --enable-linux-desktop
```

### Расширенные команды

#### Ручная генерация моста
```bash
# Если автогенерация не работает
dart run build_runner build

# Режим наблюдения для разработки
dart run build_runner watch

# Очистка сгенерированных файлов
dart pub run build_runner clean
```

### Структура проекта
```
shadowghost/
├── lib/                       # Flutter/Dart код
│   └── bridge_generated/      # Автогенерированный код моста
├── rust/                      # Rust код
│   ├── api/                   # Экспортированные функции для Flutter
│   ├── src/
│   │   └── lib.rs             # Точка входа в библиотеку Rust
│   └── Cargo.toml
├── flutter_rust_bridge.yaml   # Конфигурация моста
└── pubspec.yaml
```

## Процесс создания Pull Request

1. **Создайте форк и ветку**
   ```bash
   git checkout -b feature/your-feature
   ```

2. **Внесите изменения**
   - Редактируйте Rust код в `rust/src/`
   - Код моста перегенерируется автоматически в режиме наблюдения
   - Редактируйте Flutter код в `lib/`

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
   git commit -m "feat: добавить голосовые звонки"
   git commit -m "fix: исправить таймаут соединения"
   git commit -m "docs: обновить документацию API"
   git commit -m "bridge: обновить экспорт Rust FFI"
   ```

5. **Чеклист для PR**
   - [ ] Код моста перегенерирован (автоматически)
   - [ ] Тесты Rust пройдены (`cargo test`)
   - [ ] Тесты Flutter пройдены (`flutter test`)
   - [ ] Код отформатирован (`cargo fmt` + `flutter format .`)
   - [ ] Сгенерированные файлы не редактировались вручную
   - [ ] Безопасность проверена (если применимо)

## Руководство по Flutter Rust Bridge

- ✅ Размещайте экспортированные функции в `rust/src/api.rs`
- ✅ Используйте режим наблюдения во время разработки
- ✅ Позвольте build_runner обрабатывать генерацию кода
- ✅ Следуйте соглашениям именования Rust для экспорта

### НЕ ДЕЛАЙТЕ
- ❌ Не редактируйте файлы в `lib/bridge_generated/` вручную
- ❌ Не коммитьте сгенерированные файлы, если они в `.gitignore`
- ❌ Не используйте `tool/build.dart` (устарело)
- ❌ Не запускайте `flutter_rust_bridge_codegen` вручную

### Добавление новых функций Rust
1. Добавьте функцию в `rust/src/api.rs`
2. Мост перегенерируется автоматически в режиме наблюдения
3. Используйте сгенерированный Dart код в `lib/bridge_generated/`

## Руководство по безопасности
- **Всегда** проверяйте внешние входные данные в Rust и Dart
- Используйте **безопасные настройки по умолчанию** в криптографических функциях
- **Тестируйте** границы безопасности моста
- Сообщайте о проблемах безопасности конфиденциально на: ~`security@shadowghost.dev`~

## Архитектурные заметки
Проект использует Flutter Rust Bridge v2 для бесшовной интеграции:
- **Автоматическая генерация кода** из Rust в Dart
- **Типобезопасность** через границы языков
- **Передача данных без копирования** где возможно
- **Асинхронная поддержка** для неблокирующих операций

## Лицензия
Участвуя в проекте, вы соглашаетесь с тем, что ваши вклады будут лицензированы под CC BY-NC-SA 4.0.
