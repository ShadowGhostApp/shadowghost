import 'package:flutter/material.dart';
import 'bridge_generated.dart';

void main() {
  runApp(MyApp());
}

class MyApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Shadow Ghost',
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
      final result = initCore();
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
      final contacts = getContacts();
      setState(() => _contacts = contacts);
    } catch (e) {
      setState(() => _status = 'Load contacts error: $e');
    }
  }

  Future<void> _addContact() async {
    if (!_isInitialized) return;
    try {
      final contact = await addContact(
        'Test Contact',
        'test_public_key_${DateTime.now().millisecondsSinceEpoch}',
      );
      setState(() => _contacts.add(contact));
    } catch (e) {
      setState(() => _status = 'Add contact error: $e');
    }
  }

  Future<void> _sendMessage(String contactId) async {
    if (!_isInitialized) return;
    try {
      final messageId = await sendMessage(contactId, 'Hello from Flutter!');
      setState(() => _status = 'Message sent: $messageId');
    } catch (e) {
      setState(() => _status = 'Send message error: $e');
    }
  }

  Future<void> _startDiscovery() async {
    if (!_isInitialized) return;
    try {
      final result = await startDiscovery();
      setState(() => _status = result);
    } catch (e) {
      setState(() => _status = 'Discovery error: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Shadow Ghost')),
      body: Column(
        children: [
          Card(
            child: Padding(
              padding: EdgeInsets.all(16),
              child: Text('Status: $_status'),
            ),
          ),
          if (_isInitialized) ...[
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                ElevatedButton(
                  onPressed: _addContact,
                  child: Text('Add Contact'),
                ),
                ElevatedButton(
                  onPressed: _startDiscovery,
                  child: Text('Start Discovery'),
                ),
              ],
            ),
            Expanded(
              child: ListView.builder(
                itemCount: _contacts.length,
                itemBuilder: (context, index) {
                  final contact = _contacts[index];
                  return Card(
                    child: ListTile(
                      title: Text(contact.name),
                      subtitle: Text('Status: ${contact.status}'),
                      trailing: IconButton(
                        icon: Icon(Icons.send),
                        onPressed: () => _sendMessage(contact.id),
                      ),
                    ),
                  );
                },
              ),
            ),
          ],
        ],
      ),
    );
  }
}
