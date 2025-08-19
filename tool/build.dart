import 'dart:io';

Future<void> main(List<String> args) async {
  final command = args.isNotEmpty ? args[0] : 'help';

  switch (command) {
    case 'bridge':
      await generateBridge();
      break;
    case 'dev':
      await generateBridge();
      await runDev();
      break;
    case 'build':
      await generateBridge();
      await checkCode();
      await buildRelease();
      break;
    case 'clean':
      await clean();
      break;
    case 'install':
      await install();
      break;
    case 'check':
      await checkCode();
      break;
    case 'test':
      await runTests();
      break;
    case 'watch':
      await watchMode();
      break;
    default:
      printHelp();
  }
}

Future<void> generateBridge() async {
  print('Generating Flutter Rust Bridge...');
  final result = await Process.run('flutter_rust_bridge_codegen', ['generate']);
  if (result.exitCode != 0) {
    print('Error: ${result.stderr}');
    exit(1);
  }
  print('Bridge generated successfully');
}

Future<void> runDev() async {
  print('Starting Flutter in debug mode...');
  await Process.run('flutter', ['run', '--debug']);
}

Future<void> buildRelease() async {
  print('Building release APK...');
  final result = await Process.run('flutter', ['build', 'apk', '--release']);
  if (result.exitCode != 0) {
    print('Build failed: ${result.stderr}');
    exit(1);
  }
  print('Build completed successfully');
}

Future<void> clean() async {
  print('Cleaning...');
  await Process.run('flutter', ['clean']);
  await Process.run('cargo', ['clean'], workingDirectory: 'rust');
  print('Clean completed');
}

Future<void> install() async {
  print('Installing dependencies...');
  await Process.run('flutter', ['pub', 'get']);
  await Process.run('cargo', ['fetch'], workingDirectory: 'rust');
  print('Dependencies installed');
}

Future<void> checkCode() async {
  print('Checking Rust code...');
  final rustCheck = await Process.run('cargo', [
    'check',
  ], workingDirectory: 'rust');
  if (rustCheck.exitCode != 0) {
    print('Rust check failed: ${rustCheck.stderr}');
    exit(1);
  }

  print('Analyzing Flutter code...');
  final flutterCheck = await Process.run('flutter', ['analyze']);
  if (flutterCheck.exitCode != 0) {
    print('Flutter analyze failed: ${flutterCheck.stderr}');
    exit(1);
  }
  print('Code check passed');
}

Future<void> runTests() async {
  print('Running Rust tests...');
  await Process.run('cargo', ['test'], workingDirectory: 'rust');

  print('Running Flutter tests...');
  await Process.run('flutter', ['test']);
  print('Tests completed');
}

Future<void> watchMode() async {
  print('Starting watch mode...');
  final rustDir = Directory('rust/src');
  if (!rustDir.existsSync()) {
    print('Rust source directory not found');
    exit(1);
  }

  await generateBridge();

  rustDir.watch(recursive: true).listen((event) async {
    if (event.path.endsWith('.rs')) {
      print('Rust file changed: ${event.path}');
      await generateBridge();
    }
  });

  print('Watching for Rust file changes... Press Ctrl+C to stop');
  await Future.delayed(Duration(days: 365));
}

void printHelp() {
  print('''
Shadow Ghost Build Tool

Usage: dart run tool/build.dart <command>

Commands:
  bridge   - Generate Flutter Rust Bridge
  dev      - Generate bridge and run in debug mode
  build    - Generate bridge, check code, and build release
  clean    - Clean build artifacts
  install  - Install dependencies
  check    - Check Rust and Flutter code
  test     - Run tests
  watch    - Watch for Rust file changes and auto-generate bridge
  help     - Show this help message
''');
}
