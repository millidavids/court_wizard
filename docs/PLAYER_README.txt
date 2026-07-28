Court Wizard
============

Getting Started
---------------
Run court_wizard.exe (Windows), court_wizard (Linux), or double-click
Court Wizard.app (macOS) to play.

Save Data
---------
Your save data and settings are stored in:
  Windows:  %APPDATA%\court_wizard\
  Linux:    ~/.local/share/court_wizard/
  macOS:    ~/Library/Application Support/court_wizard/

Your active save is the file named "saves_v2.json" in that folder.

Clearing Progress
-----------------
Settings -> Clear Progress wipes your save and starts you fresh. Before the
wipe, the game keeps one rollback backup of your previous save next to the
active save:
  saves_v2.json          (your current, freshly-cleared save)
  saves_v2.json.cleared  (the save as it was the last time you cleared)

Only the most recent pre-clear backup is kept. Each Clear Progress overwrites
the previous .cleared file, so if you want to keep an older backup, copy it
somewhere safe before clearing again.

Restoring from a .cleared backup
--------------------------------
To roll back to the save you had before your last Clear Progress:
  1. Fully quit the game (so it doesn't overwrite the file on exit).
  2. Open your save folder (see "Save Data" above — on Windows, Win+R then
     type %APPDATA%\court_wizard and press Enter).
  3. Delete or rename "saves_v2.json" (the cleared/empty save).
  4. Rename "saves_v2.json.cleared" to "saves_v2.json".
  5. Launch the game — your previous progress is back.

Note: the .cleared backup goes away the next time you Clear Progress, so
restore before clearing again if you want it back.

Crash Reporting
---------------
If the game crashes, a file called "crash.log" is created in your save data
folder (see above). Please include this file when reporting bugs — it contains
information that helps us track down and fix the problem.

To find it quickly on Windows:
  1. Press Win+R
  2. Type: %APPDATA%\court_wizard
  3. Press Enter
  4. Look for crash.log
