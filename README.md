# Migrate iTunes to Rhythmbox

Small tool to help migrating iTunes library to Rhythmbox.

## Why another `migrate-itunes-to-rhythmbox`

You may have found that there is an existing
[`migrate-itunes-to-rhythmbox`](https://github.com/phauer/migrate-itunes-to-rhythmbox) project,
which was written in Python,
and has been verified by many people.

I found that tool silently drop certain items when migrating.
It might be because my library contains lots of songs with relatively complex names
(e.g. CJK characters, special symbols).

Since installation and development of that program seems to be quite involving,
I think it might be easier to just write a new one from scratch given it's not too long.

### Differences

* Less arguments and less commands to run.
  * Normally you only need to specify where the iTunes library file is.
  * It would migrate both database and playlists.
* Use metadata to match rather than file relative location.
  * This helps reducing the number of arguments.
  * It's also more robust under different filesystem path handling.
* More noise output.
  * It prints warning for any mismatching it found.
* Not migrate smart playlists.
  * Smart playlist information in iTunes library is rather cryptic,
    so I don't know how to decode it yet.
  * Also there are certain criteria iTunes supports but Rhythmbox doesn't.
  * A new option can be added to migrate the items regardless if desired.
* Doesn't migrate rating (for now).
  * Because I don't personally need.
  * But it can be added if anyone find it useful.
* Written in pure Rust.
  * No extra dynamic library dependencies.
  * Easy to install or build locally.

## Usage

### Preparation

You have to install Rhythmbox and import your Music folder before you use this tool.
Just place your music under `~/Music` and
Rhythmbox will automatically add your music files on start up.

You also need your iTunes Library in the XML format.
You can export the file in iTunes with `File > Library > Export Library...`.
You may also find the XML file under `<Music Folder>/iTunes/iTunes Music Library.xml`.

### Note

This tool is based on `(title, artist, album, track number, disc number)` tuple
for finding matched music from both libraries.

However, Rhythmbox prefers ID3v1 and APE tags over ID3v2 tags ([issue #1732][1]),
opposed to iTunes.
This can cause massive metadata mismatch for MP3 files if you ever change that in iTunes.

To mitigate this, it's recommended to run
```bash
ffmpeg -i input.mp3 -c copy -map_metadata 0 output.mp3
```
to strip ID3v1 and APE tags on all MP3 files before copying the files.

[1]: https://gitlab.gnome.org/GNOME/rhythmbox/issues/1732

### Migrate

Just run
```bash
migrate-itunes-to-rhythmbox "iTunes Music Library.xml"
```

### Backup

Rhythmbox database and playlists files are automatically backup to `.bak` file in the same directory,
and the tool would not proceed if such backup file already exists.

## License

Copyright (C) 2019 Xidorn Quan

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
