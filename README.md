# Overlay Twitch Chat 2.0

Overlay de chat Twitch pour Windows 10 et 11, écrit en Rust. Téléchargez `overlay-tchat-twitch.exe` dans les [releases](https://github.com/MasterBougli/OverlayTchatTwitch/releases) et lancez-le depuis un dossier accessible en écriture. Entrez votre chaîne puis cliquez sur **OK**.

Le clic gauche sur l’icône de notification affiche ou masque le chat. Le clic droit ouvre le menu des options. Lorsque les options sont ouvertes, déplacez directement la fenêtre du chat avec le clic gauche. Cliquez sur **OK** pour enregistrer. En mode lecture les clics traversent la fenêtre.

Les réglages sont enregistrés dans `config.ini` au format de la V1. La connexion anonyme utilise IRC sur TLS. Le pseudo IRC reprend sa couleur Twitch et un point vert identifie les modérateurs.

## Compilation Windows x64

Avec Rust stable pour MSVC et les outils C++ de Visual Studio, exécutez `cargo test` puis `cargo build --release`. L’exécutable se trouve dans `target/release`.

© BGS – Bougli Game Studio. Voir LICENSE.

[Soutenir Bougli sur Streamlabs](https://streamlabs.com/bouglitv) · [Patreon](https://www.patreon.com/cw/Bougli)
