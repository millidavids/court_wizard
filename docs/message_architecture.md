# Message-Based Architecture

This document describes the message-based communication patterns used to decouple plugins and systems in the game.

## Philosophy

**Messages (Events) are used for:**
- Cross-plugin communication where plugins should remain decoupled
- Command patterns where one module requests another to take action
- Broadcasting state changes that multiple systems need to react to

**Direct component/resource access is acceptable for:**
- Read-only queries for display purposes (e.g., UI reading game state)
- Within the same logical module/plugin
- Performance-critical paths where message overhead is unacceptable

## Implemented Message Patterns

### 1. BlockSpellInput

**Purpose:** UI systems notify spell systems to ignore input for one frame when UI buttons are clicked.

**Flow:**
```
UI Button Click → MessageWriter<BlockSpellInput> → Spell Systems (MessageReader)
```

**Location:**
- Defined in: `src/game/input/events.rs`
- Registered in: `InputPlugin`
- Sent by: `ui::in_game::systems::hud_button_action()`
- Consumed by: `guardian_circle::systems::handle_guardian_circle_casting()`, etc.

**Benefits:**
- UI module doesn't need to export any types
- Spell systems don't depend on UI module
- Clean separation of concerns

### 2. PrimeSpellMessage

**Purpose:** UI requests a spell change without directly accessing the wizard's PrimedSpell component.

**Flow:**
```
Spell Book Button Click → MessageWriter<PrimeSpellMessage> → Wizard System (MessageReader)
```

**Location:**
- Defined in: `src/game/units/wizard/components.rs`
- Registered in: `WizardPlugin`
- Sent by: `ui::spell_book::systems::button_action()`
- Consumed by: `wizard::systems::handle_prime_spell_messages()`

**Benefits:**
- UI doesn't directly query or modify game components
- Wizard module owns its own component mutations
- Easy to add validation, logging, or side effects to spell changes

## Input Event Messages

All input events are messages to decouple input detection from input consumption:

- `MouseLeftPressed`, `MouseLeftHeld`, `MouseLeftReleased`
- `MouseRightPressed`, `MouseRightHeld`, `MouseRightReleased`
- `SpacebarPressed`, `SpacebarHeld`, `SpacebarReleased`

**Flow:**
```
Input Detection System → MessageWriter<Input> → Game Systems (MessageReader)
```

This allows:
- Multiple systems to react to the same input
- Input detection to be mocked/replaced for testing
- Input systems to run independently of gameplay systems

## Guidelines for Adding New Messages

### When to use Messages:

1. **Cross-plugin boundaries** - When plugin A needs plugin B to do something
2. **Command pattern** - "Please do X" rather than "I'm doing X directly"
3. **Multiple consumers** - When several systems need to react to an event
4. **Decoupling** - When you want to break a direct dependency

### When NOT to use Messages:

1. **Read-only display** - UI reading game state for visualization
2. **Same module** - Systems within the same logical plugin
3. **Performance critical** - Very frequent operations (e.g., per-entity per-frame)
4. **Simple data flow** - When components/resources are clearer

### How to add a Message:

1. **Define the message type:**
   ```rust
   #[derive(Message, Debug, Clone, Copy)]
   pub struct MyMessage {
       pub data: SomeData,
   }
   ```

2. **Register it in a plugin:**
   ```rust
   app.add_message::<MyMessage>()
   ```

3. **Send the message:**
   ```rust
   fn sender_system(mut writer: MessageWriter<MyMessage>) {
       writer.write(MyMessage { data: value });
   }
   ```

4. **Consume the message:**
   ```rust
   fn receiver_system(mut reader: MessageReader<MyMessage>) {
       for msg in reader.read() {
           // Handle message
       }
   }
   ```

## Module Boundaries

Current clean module boundaries:

```
ui/
  ├─ Only exports: UiPlugin
  ├─ Sends messages: BlockSpellInput, PrimeSpellMessage
  └─ Reads (display only): Mana, CastingState, PrimedSpell

game/
  ├─ input/
  │   └─ Defines & sends: All input messages, BlockSpellInput
  └─ units/wizard/
      ├─ Defines: PrimeSpellMessage
      └─ Handles: PrimeSpellMessage
```

No module exports types consumed by other top-level modules, maintaining clean separation.
