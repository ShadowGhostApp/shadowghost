import 'package:flutter/material.dart';
import 'bridge_generated.dart';

void main() {
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'ShadowGhost',
      theme: ThemeData.dark(),
      home: HomeScreen(),
    );
  }
}

class HomeScreen extends StatefulWidget {
  @override
  _HomeScreenState createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  String _status = 'Not initialized';
  List<Contact> _contacts = [];
  bool _isInitialized = false;

  @override
  void initState() {
    super.initState();
    _initializeCore();
  }

  Future<void> _initializeCore() async {
    try {
      final result = await initializeCore();
      setState(() {
        _status = result;
        _isInitialized = true;
      });
      await _loadContacts();
    } catch (e) {
      setState(() => _status = 'Init error: $e');
    }
  }

  Future<void> _loadContacts() async {
    if (!_isInitialized) return;
    try {
      final contacts = await getContacts();
      setState(() => _contacts = contacts);
    } catch (e) {
      setState(() => _status = 'Load contacts error: $e');
    }
  }

  Future<void> _generateLink() async {
    if (!_isInitialized) return;
    try {
      final link = await generateMyLink();
      setState(() => _status = 'My link: $link');
    } catch (e) {
      setState(() => _status = 'Generate link error: $e');
    }
  }

  Future<void> _startServer() async {
    if (!_isInitialized) return;
    try {
      final result = await startServer();
      setState(() => _status = result);
    } catch (e) {
      setState(() => _status = 'Start server error: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('ShadowGhost')),
      body: Column(
        children: [
          Padding(padding: EdgeInsets.all(16), child: Text(_status)),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              ElevatedButton(
                onPressed: _generateLink,
                child: Text('Generate Link'),
              ),
              ElevatedButton(
                onPressed: _startServer,
                child: Text('Start Server'),
              ),
              ElevatedButton(onPressed: _loadContacts, child: Text('Refresh')),
            ],
          ),
          Expanded(
            child: ListView.builder(
              itemCount: _contacts.length,
              itemBuilder: (context, index) {
                final contact = _contacts[index];
                return ListTile(
                  title: Text(contact.name),
                  subtitle: Text(contact.address),
                  trailing: Text(contact.status.toString()),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
