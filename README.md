# Voice Assistant

A versatile voice assistant system written in Rust that provides voice interaction capabilities through various configurable services.

This project is part of a diploma thesis at HTL St. Pölten.

## Features

- Voice Recording (Local and Remote)
- Speech-to-Text (Local Whisper and Deepgram)
- Natural Language Understanding (Pattern Matching and Rasa)
- Text-to-Speech (ElevenLabs and Piper)
- LLM Integration (DeepSeek and Ollama)
- Weather Information (OpenWeatherMap)
- System Control
  - Volume Management
  - Window Management (KWin)
  - Timer Functionality
- WebSocket-based Communication
- Configurable Service Implementations

## Configuration

The system uses a TOML-based configuration system that allows for easy customization of service implementations and parameters. The configuration file is automatically created at first run in the XDG config directory.

A graphical frontend is being developed in parallel at [voice-frontend](https://github.com/eagely/voice-frontend), which provides a user-friendly interface for configuring and using the voice assistant.

## License

MIT - see [LICENSE](LICENSE)
