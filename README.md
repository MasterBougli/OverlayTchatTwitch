# Overlay Twitch Chat 2.0

Overlay de chat Twitch pour Windows 10 et 11, écrit en Rust. Téléchargez `overlay-tchat-twitch.exe` dans les [releases](https://github.com/MasterBougli/OverlayTchatTwitch/releases) et lancez-le depuis un dossier accessible en écriture. Entrez votre chaîne puis cliquez sur **Appliquer**.

Le clic gauche sur l’icône de notification affiche ou masque le chat. Le clic droit ouvre le menu des options. Dans les options, déplacez la fenêtre par la poignée supérieure et ajustez ses dimensions, les couleurs de fond par leur opacité, la taille du texte, la limite de messages et le délai de reconnexion. Cliquez sur **Appliquer** pour enregistrer. En mode lecture les clics traversent la fenêtre.

La V2 lit les réglages de `config.ini` de la V1 au premier lancement et écrit ensuite dans `config-v2.ini` à côté de l’exécutable. La connexion anonyme utilise IRC sur TLS. Les pseudos reprennent leur couleur Twitch et les badges apparaissent sous forme de libellés.

## Compilation Windows x64

Avec Rust stable pour MSVC et les outils C++ de Visual Studio, exécutez `cargo test` puis `cargo build --release`. L’exécutable se trouve dans `target/release`.

© BGS – Bougli Game Studio. Voir LICENSE.

[Soutenir Bougli sur Streamlabs](https://streamlabs.com/bouglitv) · [Patreon](https://www.patreon.com/cw/Bougli)
