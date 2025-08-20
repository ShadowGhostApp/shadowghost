# Участие в разработке Shadow Ghost

## Быстрый старт

```bash
# 1. Клонировать репозиторий
git clone <repo-url>
cd shadowghost

# 2. Установить зависимости
flutter pub get

# 3. Запустить приложение (мост генерируется автоматически)
flutter run
```

Всё! Мост Rust-Flutter генерируется автоматически при первом запуске.

## Технологический стек

- **Backend**: Rust
- **Frontend**: Flutter

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

#### Rust (если требуется)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Android Studio
```bash
# Скачать с https://developer.android.com/studio
# Установить Android SDK, Android SDK Command-line Tools, Android SDK Build-Tools
```

#### VS Code
```bash
# Расширения для Flutter
code --install-extension Dart-Code.flutter
code --install-extension Dart-Code.dart-code
```

### Проверка и настройка
```bash
# Проверка установки и зависимостей
flutter doctor

# Проверка версии
flutter --version

# Принятие лицензий Android SDK
flutter doctor --android-licenses

# Создание нового проекта для тестирования
flutter create test_app
cd test_app
flutter run
```

### Настройка устройств
```bash
# Список доступных устройств
flutter devices

# Включить веб-поддержку
flutter config --enable-web

# Включить поддержку настольных приложений
flutter config --enable-windows-desktop
flutter config --enable-macos-desktop  
flutter config --enable-linux-desktop
```

### Продвинутые команды

#### Ручная генерация моста

```bash
# Если автогенерация не сработала
dart run build_runner build

# Режим наблюдения для разработки
dart run build_runner watch

# Очистка сгенерированных файлов
dart pub run build_runner clean
```

### Структура проекта

```
shadowghost/
├── lib/                       # Код Flutter/Dart
│   └── bridge_generated/      # Автогенерируемый код моста
├── rust/                      # Код Rust
│   ├── api/                   # Экспортируемые функции для Flutter
│   ├── src/
│   │   └── lib.rs             # Точка входа библиотеки Rust
│   └── Cargo.toml
├── flutter_rust_bridge.yaml   # Конфигурация моста
└── pubspec.yaml
```

## Процесс создания Pull Request

1. **Форк и создание ветки**

   ```bash
   git checkout -b feature/ваша-фича
   ```

2. **Внесение изменений**
   - Редактируйте Rust код в `rust/src/`
   - Код моста регенерируется автоматически в режиме наблюдения
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
   git commit -m "feat: добавить голосовые вызовы"
   git commit -m "fix: исправить таймаут соединения"  
   git commit -m "docs: обновить API"
   git commit -m "bridge: обновить Rust FFI экспорты"
   ```

5. **Чеклист PR**
   - [ ] Код моста регенерирован (автоматически)
   - [ ] Тесты Rust пройдены (`cargo test`)
   - [ ] Тесты Flutter пройдены (`flutter test`)
   - [ ] Код отформатирован (`cargo fmt` + `flutter format .`)
   - [ ] Сгенерированные файлы не редактировались вручную
   - [ ] Безопасность проверена (если применимо)

## Рекомендации по Flutter Rust Bridge

- ✅ Размещайте экспортируемые функции в `rust/src/api.rs`
- ✅ Используйте режим наблюдения во время разработки
- ✅ Позвольте build_runner управлять генерацией кода
- ✅ Следуйте конвенциям именования Rust для экспортов

### НЕ

- ❌ Редактируйте файлы в `lib/bridge_generated/` вручную
- ❌ Коммитьте сгенерированные файлы, если они в gitignore
- ❌ Используйте `tool/build.dart` (устарел)
- ❌ Запускайте `flutter_rust_bridge_codegen` вручную

### Добавление новых Rust функций

1. Добавьте функцию в `rust/src/api.rs`
2. Мост регенерируется автоматически в режиме наблюдения
3. Используйте сгенерированный Dart код в `lib/bridge_generated/`

## Рекомендации по безопасности

- **Всегда** валидируйте внешние входные данные в Rust и Dart
- Используйте **безопасные настройки по умолчанию** в криптографических функциях
- **Тестируйте** границы безопасности моста
- Сообщайте о проблемах безопасности приватно на: ~`security@shadowghost.dev`~

## Архитектурные заметки

Проект использует Flutter Rust Bridge v2 для бесшовной интеграции:

- **Автоматическая генерация кода** из Rust в Dart
- **Типобезопасность** через границы языков
- **Передача данных без копирования** где возможно
- **Поддержка асинхронности** для неблокирующих операций

## Лицензия

Участвуя в разработке, вы соглашаетесь, что ваш вклад будет лицензирован под CC BY-NC-SA 4.0.
