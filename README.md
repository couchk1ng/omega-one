# Slot B

A Game Boy-centric frontend for Anbernic H700 handhelds running
[BaseOS](https://github.com/pvaibhav/BaseOS). It plays GBA, GBC and GB games, and SNES, PlayStation
and Genesis / Mega Drive games too, on a shelf of cartridges and discs that go down into the
handheld when you play them. See the [changelog](CHANGELOG.md) for what has changed.

## Supported devices

One build runs on all of them: Slot B reads the model from BaseOS at boot and lays itself out
for its panel.

| Device | Panel | Status |
|---|---|---|
| RG35XX SP | 640x480 | Main target, in daily use |
| RG SP | 720x480 | Supported, not yet retested on hardware |
| RG34XX SP, RG34XX | 720x480 | Untested |
| RG35XX Plus / 2024, H, Pro | 640x480 | Untested |
| RG40XX H, RG40XX V | 640x480 | Untested |
| RG CubeXX | 720x720 | Experimental |
| RG28XX | 480x640, turned | Experimental |

Own an untested one? Say how it goes: a report of what works on it is the most useful thing you can send.

## Highlights

- A shelf for each system, modelled low-poly and drawn the way a PlayStation drew 3D, with real
  label scans: Game Boy carts in their own plastics, SNES carts in the American shape for a game
  from the USA and the Super Famicom's for the rest, and PlayStation discs, a game on several
  discs standing as a stack of them. The one you are on floats and sways; A pulls it down out of
  the screen into the handheld, and the game comes on like a tube TV. At power-on, an opening in the
  style of Mega Man X5's boot terminal tells you the handheld's memory, chip and battery, then
  fades into the shelf.
- Six cores: mGBA and gpSP for the GBA and Snes9x and Supafaust for the SNES, chosen per cart;
  mGBA for the Game Boys; PCSX ReARMed for the PlayStation, with a BIOS of its own built in;
  Genesis Plus GX for the Genesis and Mega Drive. GBA fills a 640x480 screen and the PlayStation
  fills it at 4:3; the SNES and the Genesis are drawn at twice their size, and Game Boy stays at
  3x.
- A PlayStation game on several discs is one game: DISC, on SELECT+MENU in a game, puts another
  disc in, and the game resumes with the disc that was in the drive.
- Resume on boot, save states with a switcher, rewind (not on the PlayStation), fast-forward,
  and turbo buttons on the Game Boys.
- Link play between two handhelds over Wi-Fi, for Game Boy and GBA games: Pokémon trades,
  Advance Wars, Mario Kart and Tetris.
- Settings on START, in windows in the style of Mega Man Zero's menus that slide from screen to
  screen, over a background that keeps moving: CONFIGS (Fast Forward, Fast Forward Sound, Colour
  Correction, Rumble, Labels) and GENERAL (Date & Time, Network, Bluetooth, Theme, About).
- A tap of SELECT opens CONTROLS: what every button does, on the shelf and in a game, on one panel.
- Network: home Wi-Fi, network time, and SSH and SFTP, each ON or OFF. Y on one opens its panel:
  SSH's shows the login and root's own random password. Labels are fetched from ScreenScraper,
  one for each disc of a PlayStation game.
- Bluetooth, on handhelds that have it: pair a device and connect it, or forget it. Game sound and
  gamepads over Bluetooth are still to come.
- Theme: the background's pattern, its motion and its colour, which the menus and both bars take
  (Night is grey, Amber is Mega Man Zero's own), and the bottom bar's font, Bold or Thin.
- A pixel font, a top bar with the radios, clock and battery, and a bottom bar with the buttons.

## Quick start

1. Write the [BaseOS image](https://github.com/pvaibhav/BaseOS/releases/latest) for your model to a
   microSD card, put it in **SD1** and boot once.
2. Copy the contents of the latest release, from this repository's Releases page, to the root of
   that card.
3. Put your games in `Games/GBA`, `Games/GB`, `Games/GBC`, `Games/SNES`, `Games/PS1`,
   `Games/Genesis` and `Games/MD` (a Genesis game is a `.md`, `.gen`, `.smd` or `.bin`; the two
   folders are the same machine, kept apart so each has its own games and labels). A
   PlayStation game is a `.cue` with its `.bin`, a `.chd`, a `.pbp` or an `.m3u`; a real
   PlayStation BIOS, such as `scph5501.bin`, is used from `BIOS/` when it is there.
4. Boot. On the shelf, START opens Settings and a tap of SELECT shows every button.

## Credits and licence

Slot B is a fork of [omega-zero](https://github.com/oauramos/omega-zero) by
**Otávio ([@oauramos](https://github.com/oauramos))**, which is itself built on
[slot](https://github.com/BrandonKowalski/slot) by
[Brandon T. Kowalski](https://github.com/BrandonKowalski) (GPL-3.0). The code that hosts the
cores, with their saves, sound and input, is still largely his.

- [BaseOS](https://github.com/pvaibhav/BaseOS) by pvaibhav.
- Emulation by [mGBA](https://mgba.io) (MPL-2.0) and [gpSP](https://github.com/libretro/gpsp)
  (GPL-2.0) for the Game Boys; [Snes9x](https://github.com/libretro/snes9x) (free for
  non-commercial use only) and [Supafaust](https://github.com/libretro/supafaust), Mednafen's
  snes_faust (GPL-2.0), for the SNES;
  [PCSX ReARMed](https://github.com/libretro/pcsx_rearmed) (GPL-2.0) for the PlayStation; and
  [Genesis Plus GX](https://github.com/libretro/Genesis-Plus-GX) (free for non-commercial use
  only) for the Genesis and Mega Drive. Their
  licences and patches, and the source of all but mGBA, ship in `System/licenses/`.
- Type is [Pixel Code](https://github.com/qwerasd205/PixelCode) by Qwerasd (SIL OFL 1.1); every
  icon is drawn as pixel art. The LCD mask is derived from GigaHerz's LCD3x.
- Cart labels come from [ScreenScraper](https://www.screenscraper.fr), fetched with your own
  account.

Slot B is licensed under the GPL-3.0-or-later; see [LICENSE](LICENSE). It is a modified version of
omega-zero, and so of slot. It is written with Claude, and reviewed and tested by its
maintainer. Its screens are drawn with its UI kit, [`assets/ui`](assets/ui).

Game Boy, Game Boy Color, Game Boy Advance, Super Nintendo Entertainment System and Super Famicom
are trademarks of Nintendo. PlayStation is a trademark of Sony Interactive Entertainment. Sega,
Genesis and Mega Drive are trademarks of Sega Corporation. Anbernic is a trademark of its owner.
This project is not affiliated with any of them.
