// ignore_for_file: avoid_print

import 'dart:io';

void main(List<String> args) {
  if (args.isEmpty) {
    print('Usage: dart update_version.dart <version>');
    print('Example: dart update_version.dart 1.0.0');
    exit(1);
  }

  final version = args[0];
  final versionRegex = RegExp(r'^\d+\.\d+\.\d+$');

  if (!versionRegex.hasMatch(version)) {
    print('Error: Version must be in format x.y.z');
    exit(1);
  }

  updatePubspecVersion('pubspec.yaml', version);
  updateCargoVersion('rust/Cargo.toml', version);

  print('Version updated to $version in all files');
}

void updatePubspecVersion(String filePath, String version) {
  final file = File(filePath);
  if (!file.existsSync()) {
    print('Warning: $filePath not found');
    return;
  }

  String content = file.readAsStringSync();
  content = content.replaceAll(
    RegExp(r'^version:.*$', multiLine: true),
    'version: $version',
  );
  file.writeAsStringSync(content);
  print('Updated $filePath');
}

void updateCargoVersion(String filePath, String version) {
  final file = File(filePath);
  if (!file.existsSync()) {
    print('Warning: $filePath not found');
    return;
  }

  String content = file.readAsStringSync();
  content = content.replaceAll(
    RegExp(r'^version = ".*"$', multiLine: true),
    'version = "$version"',
  );
  file.writeAsStringSync(content);
  print('Updated $filePath');
}
