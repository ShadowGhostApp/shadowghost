import 'package:flutter/material.dart';

void main() {
  runApp(MyApp());
}

enum ContactStatus { online, offline, away }

class Contact {
  final String name;
  final String lastMessage;
  final ContactStatus status;
  final DateTime timestamp;

  Contact({
    required this.name,
    required this.lastMessage,
    required this.status,
    required this.timestamp,
  });
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

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
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final List<Contact> _contacts = [
    Contact(
      name: 'Alice',
      lastMessage: 'Hello there!',
      status: ContactStatus.online,
      timestamp: DateTime.now(),
    ),
    Contact(
      name: 'Bob',
      lastMessage: 'How are you?',
      status: ContactStatus.away,
      timestamp: DateTime.now().subtract(Duration(minutes: 30)),
    ),
  ];

  void _addContact() {
    setState(() {
      _contacts.add(
        Contact(
          name: 'User ${_contacts.length + 1}',
          lastMessage: 'New contact',
          status: ContactStatus.offline,
          timestamp: DateTime.now(),
        ),
      );
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Shadow Ghost'),
        backgroundColor: Colors.black,
      ),
      body: ListView.builder(
        itemCount: _contacts.length,
        itemBuilder: (context, index) {
          final contact = _contacts[index];
          return ListTile(
            leading: CircleAvatar(
              backgroundColor: contact.status == ContactStatus.online
                  ? Colors.green
                  : contact.status == ContactStatus.away
                  ? Colors.orange
                  : Colors.grey,
              child: Text(contact.name[0]),
            ),
            title: Text(contact.name),
            subtitle: Text(contact.lastMessage),
            trailing: Text(
              '${contact.timestamp.hour}:${contact.timestamp.minute.toString().padLeft(2, '0')}',
            ),
            onTap: () => _showMessageDialog(contact),
          );
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _addContact,
        child: Icon(Icons.add),
      ),
    );
  }

  void _showMessageDialog(Contact contact) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(contact.name),
        content: Text('Chat with ${contact.name}'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Close'),
          ),
        ],
      ),
    );
  }
}
