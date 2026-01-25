
# ♠️ Rust Blackjack Discord Bot

A Discord bot written in **Rust** that allows users to play **Blackjack** directly inside a Discord channel.

The bot is **feature-complete and functional**.  
There are a few rough edges (most notably a Discord UI warning), but gameplay works as expected and the core mechanics are fully implemented.

This project was built primarily as a learning project to explore Rust, async programming, and Discord interactions using Serenity.

---

## ✨ Features

- Multiplayer Blackjack in a Discord channel
- Reaction-based player registration
- Button-based gameplay (`Hit` / `Stand`)
- Dealer (system) logic
- Win / lose / tie resolution
- Supports multiple winners
- Game state stored safely across async tasks
- Prevents late joins once a game starts

---

## 🕹️ How It Works

1. A user starts a game with:
!blackjack


2. The bot sends a message asking users to react to join the game.

3. After a short delay, the game starts:
- Each player is dealt two cards
- The dealer draws one visible card

4. Players take turns using buttons:
- **Hit** → draw another card
- **Stand** → stop drawing

5. Once all players have acted:
- The dealer finishes drawing
- Winners are calculated and announced

---

## ⚠️ Known Issue

### “Interaction Failed” message
Discord sometimes displays **“Interaction Failed”** when a button is clicked.

Important:
- The interaction **does succeed**
- Game state updates correctly
- Player actions are applied
- Gameplay is not affected

This happens because some interactions are not explicitly acknowledged with a response.  
It’s a UI warning only and does **not** break the game.

---

## 🧠 Design Notes

- Cards are generated randomly per draw
- No deck exhaustion or card counting (by design)
- Game state is tracked per message
- Logic favors clarity and correctness over optimization

This is **not** a production-grade casino bot — it’s a functional, learning-focused project.

---

## 🧱 Tech Stack

- **Rust**
- **Serenity** (Discord API)
- **Tokio** (async runtime)
- **Rand** (random card generation)

---

## 🚀 Running the Bot

1. Set your Discord bot token:
```bash
export DISCORD_TOKEN=your_token_here
2. Run the bot 
cargo run 
3. Invite the bot with permissions for:

Sending messages

Reactions

Message content

Interactions / buttons
📌 Project Status

✅ Complete

⚠️ Minor known issues

🎮 Fully playable

🧠 Learning-focused
📝 Final Thoughts

This bot does what it’s supposed to do:
you can start a game, play Blackjack, and get correct results.

There are things that could be cleaned up or improved, but the core system works, and the project served its purpose.

If you’re reading this repo:
expect working code, some rough edges, and a lot of Rust learning along the way.
