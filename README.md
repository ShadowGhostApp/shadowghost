# 🌙 Shadow Ghost

> Decentralized P2P messenger focused on privacy and anonymity

---

<p align="center">
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  </a>
  <a href="https://flutter.dev">
    <img src="https://img.shields.io/badge/Flutter-%2302569B.svg?style=for-the-badge&logo=Flutter&logoColor=white" alt="Flutter">
  </a>
</p>
<p align="center">
  <a href="http://creativecommons.org/licenses/by-nc-sa/4.0/">
    <img src="https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg" alt="CC BY-NC-SA 4.0">
  </a>
</p>

---

##### [**README на русском**](README_RU.md)

## 📱 Description

**Shadow Ghost** is a modern P2P messenger that ensures complete communication privacy without using central servers. All data is transmitted directly between devices with end-to-end encryption.

### ✨ Key Features

- 🔒 **Complete Privacy** - no servers, no logs, no surveillance
- 🌐 **P2P Connections** - direct communication between devices
- 💬 **Text Messaging** - fast message exchange
- 📁 **File Sharing** - secure document exchange
- 🎤 **Voice Calls** - encrypted audio calls
- 🔐 **End-to-End Encryption** - protection of all data
- 🚀 **Cross-Platform** - Android, Windows, Linux

## 🏗️ Project Architecture

```mermaid
flowchart TD
    Start([📱 User opens ShadowGhost])
    
    IP[☁️ Discover my public IP<br/>via STUN server]
    
    Join[🚀 Join P2P network<br/>via Bootstrap node]
    
    Find[🔍 Find friend's address<br/>in network database]
    
    Connect{🎯 Try direct connection}
    
    Direct[✅ Direct P2P chat<br/>Fast & Private]
    
    Relay[🔄 Route via relay server<br/>Still works!]
    
    Chat([💬 Encrypted messaging])
    
    Start --> IP
    IP --> Join
    Join --> Find
    Find --> Connect
    Connect -->|Success| Direct
    Connect -->|Blocked| Relay
    Direct --> Chat
    Relay --> Chat
    
    classDef process fill:#4CAF50,stroke:#2E7D32,color:#fff
    classDef decision fill:#FF9800,stroke:#F57C00,color:#fff
    classDef result fill:#2196F3,stroke:#1565C0,color:#fff
    
    class Start,IP,Join,Find,Chat process
    class Connect decision
    class Direct,Relay result
```

## 🔒 Security

Shadow Ghost uses modern cryptographic algorithms:

- **AES-256** for message encryption  
- **RSA-4096** for key exchange  
- **SHA-256** for hashing  
- **QUIC protocol** for secure transmission  

---

## 🎯 Supported Platforms

| Platform  | Status   |
|-----------|----------|
| 🤖 Android | 🚧 Planned |
| 🪟 Windows | 🚧 Planned |
| 🐧 Linux   | 🚧 Planned |

---

## 🤝 Contributing

We welcome any contribution to the project development!

- 📋 Technical Documentation: [**`CONTRIBUTING.md`**](CONTRIBUTING.md)
- 🐛 Report Bug: [**Issues**](../../issues)
- 💡 Suggest Enhancement: [**Discussions**](../../discussions)

---

## 📄 License

This project is licensed under the [**Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License**](LICENSE).

---

## 🌟 Support the Project

If you like **Shadow Ghost**, please give it a ⭐ star!  

<p align="center">
  <img src="https://readme-typing-svg.demolab.com/?font=Fira+Code&size=20&pause=1000&color=FF2E2E&center=true&vCenter=true&width=800&lines=Made+with+%E2%9D%A4+for+privacy+and+freedom+of+communication.">
</p>

