# Brand assets (repo kit)

Only assets used by packaging and docs stay in-tree.

| File | Use |
|------|-----|
| `exports/app-icon-1024.png` | **Master** app icon → `AppIcon.icns` via `scripts/package-macos.sh` |
| `exports/app-icon-256.png` | Docs / previews |
| `exports/app-icon-128.png` | README hero (light) |

Generated (gitignored): `packaging/macos/AppIcon.icns` — rebuilt when the master PNG is newer.

Scratch / rejected concepts stay out of the repo.

## Final art

**Concept `5.jpg` as-is** — soft periwinkle plush cube inside a white browser window.
Do not recolor toward hard KMBlue, enlarge eyes, or add teary effects.

## Concept

**Kimini** is a lightweight browser shell for **Kimi Code**. The mark is a soft
periwinkle-blue plush “face cube” sitting in a browser window — mini shell,
mini buddy. Inspired by Kimi Code’s product face mark (blue tile + two eyes),
with Kimini’s own softer palette and window framing.

Not a letter-K monogram (that’s the main Kimi app mark).

## Colors

| Token | Hex | Notes |
|-------|-----|--------|
| Plush periwinkle | ~`#8DA3CE` | Character body (from final art) |
| Window mint→blue | `#C1E5CE` → `#4A90D9` | Background gradient |
| KMBlue | `#1A88FF` | Kimi Code product accent (UI links; not the icon body) |
| Accent blue | `#1783FF` | Kimi brand blue dot |
| Ink | `#1A1C1E` | Dark UI / copy |

## Regenerating derived icons

```sh
# package script rebuilds packaging/macos/AppIcon.icns from app-icon-1024.png
make app
# or force:
rm -f packaging/macos/AppIcon.icns && make app
```
