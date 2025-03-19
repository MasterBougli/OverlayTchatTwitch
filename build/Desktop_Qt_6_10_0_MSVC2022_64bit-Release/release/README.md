# Overlay Twitch Chat

Une application qui affiche le chat Twitch en overlay par-dessus n'importe quelle application Windows.

## Fonctionnalités

- Affichage du chat Twitch en overlay transparent
- Connexion directe via IRC au chat Twitch
- Clic à travers l'overlay (click-through) avec API Windows native
- Configuration de la chaîne Twitch à suivre
- Personnalisation du style (taille de police, opacité)
- Redimensionnement de la fenêtre de chat
- Déplacement facile de l'overlay
- Gestion du nombre maximal de messages affichés
- Support des couleurs et badges Twitch
- Reconnexion automatique configurable en cas de déconnexion
- Sauvegarde automatique de la position de l'overlay
- Icône dans le systray avec menu contextuel
- Sauvegarde des paramètres dans un fichier config.ini
- Bouton de réinitialisation des paramètres
- Application autonome en un seul fichier .exe (compilation statique)

## Développeur

Développé par BGS - Bougli Game Studio
© 2024 BGS - Bougli Game Studio. Tous droits réservés.

## Configuration requise

- Windows 10/11
- Qt 5.15 ou supérieur (version statique pour la compilation)
- Modules Qt Core, GUI, Network et Widgets

## Comment compiler

### Compilation standard
1. Assurez-vous d'avoir Qt installé avec les modules requis
2. Ouvrez le fichier .pro avec Qt Creator
3. Configurez le projet pour votre version de Qt
4. Compilez et exécutez

### Compilation statique (application autonome)
1. Installez une version statique de Qt (compilation manuelle nécessaire)
2. Configurez Qt Creator pour utiliser cette version statique
3. Assurez-vous que tous les plugins nécessaires sont disponibles
4. Compilez en mode Release
5. L'application sera générée en un seul fichier .exe

## Utilisation

1. Lancez l'application
2. Si aucune chaîne n'est configurée, la fenêtre d'options s'ouvre automatiquement
3. Configurez une chaîne Twitch
4. Le chat s'affiche en overlay
5. Clic droit sur l'icône du systray pour accéder aux options
6. Dans les options, configurez la chaîne Twitch et les paramètres d'affichage
7. Vous pouvez déplacer l'overlay lorsque la fenêtre d'options est ouverte
8. La position de l'overlay est automatiquement sauvegardée

## Fichier de configuration

Tous les paramètres sont sauvegardés dans un fichier config.ini situé dans le même dossier que l'application.
Ce fichier contient :
- Le nom de la chaîne Twitch à afficher
- Les paramètres d'affichage (taille de police, opacité du fond)
- Les dimensions et la position de la fenêtre
- Le nombre maximal de messages à conserver
- Les paramètres de reconnexion automatique

Vous pouvez réinitialiser tous les paramètres en utilisant le bouton "Réinitialiser" dans la fenêtre des options.

## Options de connexion

L'application offre des options avancées de connexion :
- Activation/désactivation de la reconnexion automatique
- Configuration de l'intervalle de reconnexion
- Affichage de messages d'état pour suivre l'état de la connexion

## Avantages de l'utilisation d'IRC

- Consommation de ressources plus faible qu'une solution basée sur WebView
- Plus de contrôle sur le format d'affichage
- Meilleure personnalisation
- Accès complet aux tags et badges Twitch
- Pas de dépendance à QtWebEngine

## Notes

- L'application utilise le protocole IRC pour se connecter directement aux serveurs Twitch
- Pour masquer/afficher l'overlay, cliquez sur l'icône du systray
- L'overlay reste toujours au premier plan des applications
- Pour permettre de cliquer à travers l'overlay, nous utilisons l'API Windows native
- Toute modification et redistribution de cette application est interdite sans autorisation explicite

## Licence

Copyright © 2024 BGS - Bougli Game Studio. Tous droits réservés.

Voir le fichier LICENSE pour plus de détails sur les restrictions de copyright. 