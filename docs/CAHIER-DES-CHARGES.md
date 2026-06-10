# Cahier des charges — Optimiseur de coupe et de perçage d'extrusions

**GestEase Technologie Inc.** · Version 2.0 · Juin 2026
Outil interne · atelier / fabrication

---

## 1. Objectif

Fournir une application web permettant de générer des **plans de coupe optimaux** pour des
extrusions, à partir d'un inventaire de barres brutes et d'une liste de coupe, en **minimisant la
perte de matière**. L'outil calcule également, pour chaque pièce coupée, la **répartition
symétrique des trous de perçage** (entraxe et marges de bout). Il gère l'import Excel/CSV des
listes de coupe, l'export des plans, et conserve l'**historique** des travaux (jobs) en base de
données.

## 2. Contexte et périmètre

L'outil répond à deux besoins d'atelier complémentaires :

- **Découpe (1D cutting stock)** : à partir de plusieurs longueurs de stock (dispo limitée ou
  illimitée), déterminer le nombre de barres nécessaires et le plan de coupe minimisant la perte,
  en tenant compte du trait de scie (kerf) et d'un seuil de chute réutilisable.
- **Perçage** : répartir symétriquement les trous sur chaque pièce, le nombre de trous s'ajustant
  à la longueur, dans le respect des marges de bout.

### 2.1 Hors périmètre (version 2.0)

- Génération de fichiers CNC ou de plans techniques détaillés.
- Gestion d'inventaire matière en temps réel (entrées/sorties de stock).
- Dimensionnement de l'outil de coupe ou du carré de perçage (donnée indicative).

## 3. Architecture technique

Application **découplée en trois conteneurs** (le frontend ne communique avec le backend que par
l'API REST `/api/*`) :

- **frontend** — Nginx sert l'interface React (Vite + TypeScript) et relaie `/api` vers le backend.
- **backend** — API Rust (Axum) : moteurs de calcul (coupe, perçage), import/export, persistance.
- **db** — PostgreSQL : enregistrement et historique des jobs.

Ports retenus pour l'exécution locale (Docker) :

| Service | Hôte → Conteneur | Rôle |
|---|---|---|
| frontend | 8099 → 80 | Entrée navigateur (SPA + proxy API) |
| backend | 8097 → 8080 | API REST (accès direct / debug) |
| postgres | 5436 → 5432 | Base de données |

## 4. Données d'entrée

### 4.1 Données de base

- **Numéro de job** : saisie clavier ou lecteur de **code-barres** (keyboard-wedge).
- **Trait de scie (kerf)** : épaisseur retirée à chaque coupe (défaut 1/8″), configurable.
- **Seuil de chute réutilisable** : une chute ≥ ce seuil est récupérable (non comptée comme perte).

### 4.2 Longueurs de stock

- Inventaire des barres brutes (étiquette + longueur). La disponibilité vide = illimitée.
- Avec plusieurs longueurs, l'optimiseur choisit le **meilleur mélange**.

### 4.3 Liste de coupe

- Une ligne par type de pièce, saisie manuelle ou par **import Excel (.xlsx) / CSV**.
- Un **modèle CSV** téléchargeable facilite la saisie. Colonnes reconnues (insensibles à la
  casse/accents) :

| Colonne | Obligatoire | Description |
|---|---|---|
| `jobID` | non | Numéro de job (sinon saisi dans l'interface) |
| `partID` | non | Identifiant de la pièce (sinon numéroté automatiquement) |
| `model` | non | Modèle / profil de référence |
| `longueur` | **oui** | Longueur de la pièce (accepte `6`, `5'`, `60"`, `3-1/2"`, `1524mm`) |
| `quantite` | non | Nombre de pièces (vide = 1 par ligne) |
| `unite` | non | `pi`, `po` ou `mm` (vide = unité de l'interface) |

### 4.4 Perçage

- **Entraxe** centre-à-centre entre deux trous consécutifs.
- **Marge de bout** minimale et maximale (distance bord → centre du trou extrême). Paramètres
  **globaux** appliqués à toutes les pièces.

## 5. Données de sortie

- **Plan de coupe** : nombre de barres, contenu de chaque barre, chute (réutilisable ou perte),
  taux d'utilisation, détail par longueur.
- **Plan de perçage** : par pièce, nombre de trous, marge de bout, position de chaque trou depuis
  chaque extrémité, cotes (début, 1ᵉʳ entraxe, fin).
- **Exports CSV** : plan de coupe et positions de perçage (lisibles dans Excel).
- **Historique** : jobs enregistrés en base, rechargeables ou supprimables.

## 6. Règles de calcul

### 6.1 Unités internes

Toutes les longueurs sont calculées en **micromètres entiers** (`1 po = 25 400 µm`,
`1 pi = 304 800 µm`, `1 mm = 1 000 µm`) afin d'éliminer toute erreur d'arrondi sur les fractions.

### 6.2 Optimisation de coupe

Modèle du trait de scie : pour `k` pièces sur une barre,
`Σ(longueurs) + k × kerf ≤ longueur_barre` ; `chute = longueur − Σ(longueurs) − k × kerf`
(une coupe comptée par pièce, convention conservatrice).

Objectif (par ordre lexicographique) :

1. placer le maximum de pièces ;
2. minimiser la **matière brute consommée** (résiduel le plus court) ;
3. moins de barres ;
4. concentrer la perte (chute réutilisable maximale).

Méthode : heuristiques gloutonnes (First/Best-Fit Decreasing × choix du stock) puis recherche
locale (élimination de barre, réduction de stock) sous budget de temps. Approche *best-effort*,
optimale sur les cas usuels.

### 6.3 Perçage

- Répartition **symétrique** autour du centre (`centre = longueur / 2`).
- `span = (N − 1) × entraxe` ; `marge = (longueur − span) / 2` ; valide si
  `marge_min ≤ marge ≤ marge_max`.
- On retient le `N` donnant la **plus grande marge valide** (centrage maximal).
- `N` impair → un trou au centre ; `N` pair → les trous encadrent le centre.
- Cas sans solution (zones mortes) : la pièce est signalée « perçage impossible ».

> **Exemple de référence** : profil 2400 mm, entraxe 112 mm, marges [40, 112] mm → **21 trous**,
> marge de bout **80 mm**, trou central à **1200 mm** des deux extrémités.

## 7. Exigences techniques

- Backend **Rust** (Axum) ; frontend **React** (Vite + TypeScript) ; base **PostgreSQL** (sqlx).
- Déploiement par **Docker** (3 conteneurs) ; image frontend = point d'entrée public.
- Calculs en **arithmétique entière (µm)** ; aucune erreur d'arrondi.
- Icônes **Material en SVG inline** (aucune dépendance réseau, fonctionnement hors-ligne).
- Thème **clair / sombre** manuel et palette officielle GestEase.
- Licence **MIT** ; détenteur des droits : GestEase Technologie Inc.

## 8. Interface et expérience

- Sélecteur d'unités **Impérial (pi/po) ⇄ Métrique (mm)** en haut à droite.
- Sélecteur de **thème clair / sombre** (mémorisé) à côté du sélecteur d'unités.
- Bouton d'**aide « ? » par section** ouvrant une fenêtre modale expliquant son rôle.
- Saisie de longueur tolérante (fractions, pi/po, mm) ; champ numéro de job compatible code-barres.
- Visualisations : barres de coupe colorées (chute réutilisable vs perte) ; barre de perçage avec
  trous et cotes.
- Accessibilité : anneau de focus visible, libellés ARIA sur les boutons-icônes.

## 9. Évolutions possibles

- Export Excel (.xlsx) mis en forme et bon de coupe imprimable (PDF).
- Cotes sur les barres de coupe ; schéma SVG du profil à l'échelle.
- Paramètres de perçage par modèle ; gestion des chutes réutilisables en inventaire.
- Convention de kerf alternative (`n − 1` traits) configurable.

## 10. Critères d'acceptation

- Pour une liste réalisable, toutes les pièces sont placées et le nombre de barres respecte la
  borne basse théorique.
- La somme `(pièces + traits de scie + chute)` égale la longueur de chaque barre utilisée.
- Le perçage est symétrique : distance du 1ᵉʳ trou depuis A = distance du dernier depuis B ; trou
  central (si `N` impair) au milieu exact.
- Les longueurs importées sont converties exactement (ex. `6 pi = 1828,8 mm`) quelle que soit
  l'unité du fichier.
- Aucune valeur négative ni hors barre n'est produite ; un cas impossible est signalé explicitement.
