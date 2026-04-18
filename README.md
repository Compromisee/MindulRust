
<div align="center">

# 🌳 Focus Forest

### A Beautiful Pomodoro Timer That Grows a Tree As You Focus

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Actix Web](https://img.shields.io/badge/Actix_Web-0055ff?style=for-the-badge&logo=rust&logoColor=white)
![HTML5](https://img.shields.io/badge/HTML5-E34F26?style=for-the-badge&logo=html5&logoColor=white)
![CSS3](https://img.shields.io/badge/CSS3-1572B6?style=for-the-badge&logo=css3&logoColor=white)
![JavaScript](https://img.shields.io/badge/JavaScript-F7DF1E?style=for-the-badge&logo=javascript&logoColor=black)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

<p align="center">
  <strong>Stay focused. Grow trees. Track progress. All in one beautiful app.</strong>
</p>

<p align="center">
  <em>A single-file Rust application — no npm, no bundlers, no external files. Just <code>cargo run</code> and go.</em>
</p>

---

**[Features](#-features) · [Screenshots](#-screenshots) · [Quick Start](#-quick-start) · [Usage](#-usage) · [API](#-api-reference) · [Tech Stack](#-tech-stack) · [Contributing](#-contributing)**

---

</div>

## ✨ Features

### 🎯 Core
| Feature | Description |
|---------|-------------|
| ⏱️ **Pomodoro Timer** | 15 / 25 / 45 / 60 / 90 / 120 min presets + custom duration |
| 🌳 **Growing Tree** | Dynamic fractal tree that evolves through **10 stages** as you focus |
| 😵 **Distraction Logger** | Log what broke your focus with timestamps |
| 📊 **Weekly Analytics** | 8 stat cards + 3 interactive bar charts |
| 📋 **Session History** | Full history with durations, times, and completion status |

### 🎵 Audio System
| Feature | Description |
|---------|-------------|
| 🌧️ **7 Ambient Sounds** | Rain, Forest, Ocean, Fire, Wind, Birds, Café — generated via Web Audio API |
| 🔗 **URL Playback** | Paste any `.mp3` / audio link to play |
| 📁 **MP3 Upload** | Upload your own audio files to the server |
| 🔊 **Volume Control** | Slider with dynamic icon state |

### 🎨 Visual Design
| Feature | Description |
|---------|-------------|
| ✨ **Particle System** | 80 animated particles with connecting lines |
| 🌈 **Animated Gradient** | Shifting gradient header with 4 colors |
| 💫 **Glow Effects** | Pulsing timer ring while focusing |
| 🌙 **Night Scene** | Moon, stars, fireflies in the tree canvas |
| 🌸 **Cherry Blossoms** | Appear on grand-level trees |
| 🔣 **Google Material Icons** | Consistent iconography throughout |
| 📱 **Fully Responsive** | Works on desktop, tablet, and mobile |

### 🌳 Tree Growth Stages

```
 Level       │ Minutes │ Visual
─────────────┼─────────┼──────────────────────────
 Seed 🌰     │    0    │ Small brown seed
 Sprout 🌱   │    5    │ Green stem + 2 leaves
 Seedling 🌿 │   20    │ Taller stem + more leaves
 Sapling 🪴  │   45    │ Thin trunk + small canopy
 Young 🌳    │   90    │ Branching tree
 Mature 🌲   │  150    │ Full canopy + fruits
 Grand 🌴    │  240    │ Large tree + blossoms
 Ancient 🏔  │  400    │ Massive trunk + fireflies
 Legendary ✨│  700    │ Glowing ancient tree
 World 🌍    │ 1000    │ Ultimate world tree
```

---

## 📸 Screenshots

<div align="center">

### Timer View
```
┌──────────────────────────────────────────────────┐
│  ⏱ Focus Timer          │  🌳 Your Tree          │
│                          │                        │
│     ┌──────────┐         │    ╱╲    ╱╲            │
│     │  25:00   │         │   ╱🌿╲  ╱🌿╲          │
│     │ Focusing │         │  ╱ 🌳  ╲╱    ╲         │
│     └──────────┘         │  │    ||    │          │
│                          │  │    ||    │          │
│  [▶ Start] [⟲ Reset]    │  ════════════          │
│                          │  Total: 45 min         │
├──────────────────────────┼────────────────────────┤
│  😵 Log Distraction      │  🎵 Sounds & Music     │
│  [Phone call      ] [+]  │  🌧 🌲 🌊 🔥 💨 🐦 ☕  │
│  • Phone call    14:23   │  🔊 ━━━━━━━━━●━━ 50%  │
│  • Email ping    14:31   │                        │
└──────────────────────────┴────────────────────────┘
```

### Analytics View
```
┌────────────────────────────────────────────────────┐
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐    │
│  │ 185  │ │  12  │ │  15  │ │   8  │ │ 420  │    │
│  │ min  │ │ sess │ │ avg  │ │ dist │ │ all  │    │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘    │
│                                                    │
│  📊 Daily Focus Minutes                            │
│  45 │  ██                                          │
│  30 │  ██  ██                     ██               │
│  15 │  ██  ██  ██  ██         ██  ██               │
│   0 │──██──██──██──██──██──██──██──                │
│     │  Mon  Tue  Wed  Thu  Fri  Sat  Sun           │
└────────────────────────────────────────────────────┘
```

</div>

---

## 🚀 Quick Start

### Prerequisites

- **Rust** (1.70+) — [Install Rust](https://rustup.rs/)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/focus-forest.git
cd focus-forest

# Build and run
cargo run --release
```

That's it. Open **http://localhost:8080** in your browser.

### One-Liner

```bash
git clone https://github.com/yourusername/focus-forest.git && cd focus-forest && cargo run --release
```

---

## 📖 Usage

### Starting a Focus Session

1. **Choose duration** — Click a preset (15/25/45/60/90/120) or enter custom minutes
2. **Press Start** — Timer begins, tree starts growing
3. **Stay focused** — Watch your tree grow in real-time
4. **Log distractions** — Type what distracted you (optional)
5. **Complete session** — Your tree permanently grows!

### Playing Ambient Sounds

| Method | How |
|--------|-----|
| **Built-in** | Click any sound button (Rain, Forest, Ocean, etc.) |
| **URL** | Switch to "Link" tab → paste audio URL → press ▶ |
| **Upload** | Switch to "Upload" tab → select MP3 file from your computer |

### Viewing Analytics

- Click the **📊 Analytics** tab
- View 8 stat cards: minutes, sessions, avg length, distractions, completion rate, streak
- 3 bar charts: daily focus minutes, sessions per day, distractions per day

---

## 📁 Project Structure

```
focus-forest/
├── Cargo.toml          # Rust dependencies
├── src/
│   └── main.rs         # Everything — backend + embedded frontend
├── README.md
└── LICENSE
```

> **Yes, it's really just one file.** The entire HTML/CSS/JS frontend is embedded
> in `main.rs` as a string constant. No build tools. No webpack. No node_modules.

---

## 🔌 API Reference

All endpoints return JSON.

### Sessions

| Method | Endpoint | Body | Description |
|--------|----------|------|-------------|
| `POST` | `/api/session/start` | `{ "duration_minutes": 25 }` | Start a new focus session |
| `POST` | `/api/session/end` | `{ "session_id": "...", "actual_minutes": 24.5, "completed": true }` | End a session |
| `POST` | `/api/session/distraction` | `{ "session_id": "...", "note": "Phone call" }` | Log a distraction |
| `GET` | `/api/sessions` | — | Get all sessions |
| `GET` | `/api/analytics` | — | Get weekly analytics |
| `GET` | `/api/total` | — | Get total focus minutes |

### Audio

| Method | Endpoint | Body | Description |
|--------|----------|------|-------------|
| `POST` | `/api/sound/upload` | `multipart/form-data` | Upload an audio file |
| `GET` | `/api/sound/list` | — | List uploaded sounds |
| `GET` | `/api/sound/{id}` | — | Stream an uploaded sound |

### Example

```bash
# Start a 25-minute session
curl -X POST http://localhost:8080/api/session/start \
  -H "Content-Type: application/json" \
  -d '{"duration_minutes": 25}'

# Get analytics
curl http://localhost:8080/api/analytics
```

---

## 🛠 Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Backend** | [Rust](https://www.rust-lang.org/) | Systems programming language |
| **Web Framework** | [Actix Web 4](https://actix.rs/) | High-performance async web server |
| **File Upload** | [actix-multipart](https://docs.rs/actix-multipart) | Handle MP3 uploads |
| **Serialization** | [Serde](https://serde.rs/) | JSON serialization/deserialization |
| **Date/Time** | [Chrono](https://docs.rs/chrono) | Date handling for analytics |
| **IDs** | [UUID](https://docs.rs/uuid) | Unique session identifiers |
| **Frontend** | Vanilla HTML/CSS/JS | No framework, no build step |
| **Icons** | [Material Symbols](https://fonts.google.com/icons) | Google's icon library |
| **Font** | [Inter](https://fonts.google.com/specimen/Inter) | Clean variable font |
| **Audio** | [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API) | Procedural ambient sounds |
| **Graphics** | [Canvas API](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API) | Tree rendering + particles |

---

## 🧩 Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Browser                           │
│                                                     │
│  ┌─────────┐  ┌──────────┐  ┌────────────────────┐ │
│  │ Particle │  │  Timer   │  │   Tree Canvas      │ │
│  │ Canvas   │  │  Ring    │  │   (fractal render) │ │
│  └─────────┘  └──────────┘  └────────────────────┘ │
│  ┌─────────────────┐  ┌──────────────────────────┐ │
│  │  Audio Engine    │  │  Analytics Charts        │ │
│  │  (Web Audio API) │  │  (dynamic bar charts)    │ │
│  └─────────────────┘  └──────────────────────────┘ │
│                     │                               │
│              fetch() API calls                      │
│                     │                               │
└─────────────────────┼───────────────────────────────┘
                      │
              HTTP (port 8080)
                      │
┌─────────────────────┼───────────────────────────────┐
│              Actix-Web Server                        │
│                     │                               │
│  ┌──────────────────┴────────────────────────────┐  │
│  │              Route Handlers                    │  │
│  │  /api/session/*  /api/analytics  /api/sound/*  │  │
│  └──────────────────┬────────────────────────────┘  │
│                     │                               │
│  ┌──────────────────┴────────────────────────────┐  │
│  │           Mutex<AppState>                      │  │
│  │  ┌────────────┐ ┌──────┐ ┌─────────────────┐  │  │
│  │  │  Sessions  │ │ Stats│ │ Uploaded Sounds  │  │  │
│  │  │  Vec<...>  │ │  f64 │ │ HashMap<id,bytes>│  │  │
│  │  └────────────┘ └──────┘ └─────────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

---

## ⚙️ Configuration

The app runs with sensible defaults. To change the port, edit `main.rs`:

```rust
// Change this line in main()
.bind("127.0.0.1:8080")?    // ← change port here
```

---

## 🤝 Contributing

Contributions are welcome! Here are some ideas:

### Good First Issues
- [ ] Add dark/light theme toggle
- [ ] Persist sessions to a SQLite database
- [ ] Add keyboard shortcuts (Space = start/pause)
- [ ] Add a "daily goal" setting

### Feature Ideas
- [ ] Multiple tree species to unlock
- [ ] Export analytics as CSV
- [ ] Desktop notifications when timer completes
- [ ] Multiplayer — see friends' forests
- [ ] Spotify / YouTube Music integration

### How to Contribute

```bash
# Fork the repo, then:
git clone https://github.com/yourusername/focus-forest.git
cd focus-forest
git checkout -b feature/your-feature

# Make changes to src/main.rs
# Test it
cargo run

# Commit and push
git add .
git commit -m "feat: add your feature"
git push origin feature/your-feature

# Open a Pull Request
```

---

## 📝 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file.

```
MIT License

Copyright (c) 2024 Focus Forest Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software...
```

---

## 💖 Acknowledgements

- [Actix Web](https://actix.rs/) — Blazing fast Rust web framework
- [Google Material Symbols](https://fonts.google.com/icons) — Beautiful icons
- [Inter Typeface](https://rsms.me/inter/) — Clean UI font
- Forest City concept inspired by [Forest App](https://www.forestapp.cc/)

---

<div align="center">

### ⭐ Star this repo if Focus Forest helped you stay focused!

**Made with 🦀 Rust and 💚 love**

<br>

[Report Bug](https://github.com/yourusername/focus-forest/issues) ·
[Request Feature](https://github.com/yourusername/focus-forest/issues) ·
[Discussions](https://github.com/yourusername/focus-forest/discussions)

</div>
```
