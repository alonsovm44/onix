# FAQ 
I see Onix as a middle-ground protocol that prioritizes the developer's distribution velocity and the user's security posture.

Here are the answers to common FAQ and some additional ones that are likely to come up.

1) Why not just use winget or chocolatey in Windows?
While winget and chocolatey are excellent for the Windows ecosystem, Onix provides a different value proposition:

Cross-Platform Consistency: A developer writes one install.onix file. It works for a user on Fedora, macOS (Intel/M1), and Windows. You don't have to learn winget manifests, Homebrew formulae, and Debian packaging simultaneously.
Decentralization: Traditional package managers usually require a "Pull Request" or a submission process to a central repository. Onix is "bring your own host." You just drop the .onix file in your GitHub Release, and it's ready.

The Permission UX: winget often runs installers (MSIs/EXEs) opaquely. Onix's TUI explicitly tells the user: "This tool wants to touch your PATH and write to this specific folder." It’s about informed consent.

2) If Onix fetches precompiled binaries, why not just ask the users to download them and install them manually?
Manual installation is the "friction-filled path" where both security and usability go to die:

The Trust Gap: Users almost never verify SHA256 checksums manually. Onix makes verification mandatory and automatic.
The Setup Tax: Moving a binary to ~/.local/bin, running chmod +x, and manually editing a .zshrc or Windows Registry for the PATH is error-prone. Onix automates the "plumbing" so the tool "just works" immediately after download.

Discovery of Versioning: Onix can eventually handle the logic of "Is there a newer version?" which manual downloads cannot do easily.

3) Is this just Nix but simpler?

- Core difference (one sentence)
Nix: “Rebuild your entire environment deterministically”
Onix: “Install this CLI safely and transparently in seconds”

⚖️ Deep comparison

- Scope
Nix: 
+ OS-level package manager
+ system configurations
+ dev environments
+ full dependency graphs

- Onix
+ ONLY CLI tools / binaries
+ no system rebuilding
+ no environment modeling

Onix is narrow by design

4) other FAQ that might arise:

- "How do I uninstall things Onix installed?"

Answer: Since Onix is declarative, it can track exactly which files it created and which environment variables it modified, allowing for a "clean" uninstall that doesn't leave junk on the system.

- "How does Onix itself get installed? (The Bootstrap Problem)"

Answer: This is the classic chicken-and-egg problem. Onix would likely be a single-file static binary download. Once a user has onix, they never need to manually install a CLI tool again.

- "Does Onix support complex tools with shared library dependencies?"

Answer: In v1, Onix targets standalone binaries (statically linked). For complex tools, we would recommend developers use AppImage or flatpaks, or stick to OS package managers. Onix wins on the "small-to-medium utility" category.

- "Is the manifest safe from being tampered with?"

Answer: This is why the schema includes sha256 for the artifacts. In future versions, we could support GPG signing of the .onix manifest itself to ensure the instructions haven't been hijacked.
