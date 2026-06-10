# Optimiseur de coupe — extrusions

Outil web pour générer des **plans de coupe** à partir de longueurs de barres brutes
(plusieurs longueurs possibles) et d'une **liste de coupe**, en **minimisant la perte**.
Backend **Rust** (Axum), persistance **PostgreSQL**, frontend **React** (Vite + TypeScript).

## Fonctionnalités

- **Optimiseur 1D (cutting stock)** : minimise la matière brute consommée, donc le résiduel.
- **Multi-longueurs de stock** (dispo limitée ou illimitée) — choisit le meilleur mélange.
- **Trait de scie (kerf)** configurable (défaut 1/8″), pris en compte à chaque coupe.
- **Chutes réutilisables** : seuil configurable ; au-delà = retour stock (pas une perte).
- **Bascule unités** impérial (pi/po, fractions) ↔ métrique (mm).
- **Import Excel (.xlsx) / CSV** d'une liste de coupe + **modèle CSV** téléchargeable.
- **Numéro de job** au clavier ou au **lecteur code-barres** (USB/keyboard-wedge).
- **Export CSV** du plan de coupe et **historique** des jobs en base.
- **Perçage** : répartition symétrique des trous par pièce (entraxe + marges de bout), N ajusté selon la longueur — la spec d'origine `calcul-percage`, intégrée. Export CSV des positions.

Calculs en arithmétique **entière (micromètres)** → aucune erreur d'arrondi sur les fractions.

## Format d'import (modèle)

Colonnes (entêtes insensibles à la casse/accents) :

| jobID | partID | model | longueur | quantite *(opt.)* | unite *(opt.)* |
|-------|--------|-------|----------|-------------------|----------------|
| JOB-001 | P1 | Profil-U-100 | 6 | 2 | pi |

- `quantite` absente ⇒ chaque ligne = 1 pièce.
- `unite` absente ⇒ unité choisie dans l'interface (pi/po/mm). Valeurs : `pi`, `po`, `mm`.
- `longueur` accepte aussi des formats comme `5'`, `60"`, `3-1/2"`, `1524mm`.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — conteneurs, modules, schéma de données, API.
- [docs/ALGORITHME.md](docs/ALGORITHME.md) — optimisation de coupe (1D cutting stock) et perçage.
- [docs/specs/](docs/specs/) — cahier des charges d'origine du perçage (désormais intégré).
- [exemples/modele_liste_coupe.xlsx](exemples/modele_liste_coupe.xlsx) — modèle d'import prêt à l'emploi.

## Structure du projet

```
optimiseur-coupe/
├── backend/            API Rust (Axum) + moteurs de calcul        → docs/ARCHITECTURE.md
│   └── src/
│       ├── optimizer/  optimisation de coupe (model, solver, units)
│       ├── drilling.rs répartition des trous de perçage
│       ├── import.rs   lecture .xlsx / .csv
│       ├── export.rs   exports CSV
│       └── …
├── frontend/           SPA React (Vite + TS), servie par Nginx
├── docs/               documentation + specs d'origine
├── exemples/           modèle d'import Excel
└── docker-compose.yml  pile 3 conteneurs (frontend · backend · db)
```

## Lancer en local (Docker)

```bash
docker compose up --build
```

Puis ouvrir **http://localhost:8099**.

Ports : frontend `8099→80` (entrée navigateur), backend `8097→8080` (API/debug), PostgreSQL `5436→5432`.

## Développement

```bash
# Backend (port 8097)
cd backend && PORT=8097 DATABASE_URL=postgres://optimiseur:optimiseur@localhost:5436/optimiseur cargo run

# Frontend (proxy /api -> 8097)
cd frontend && npm install && npm run dev
```

Tests du moteur : `cd backend && cargo test`.

## API

| Méthode | Route | Rôle |
|--------|-------|------|
| POST | `/api/optimize` | calcule un plan (Problem → Solution) |
| POST | `/api/import` | multipart `file` + `unit` → items |
| GET | `/api/template.csv` | modèle d'import |
| POST | `/api/export.csv` | export du plan de coupe |
| POST | `/api/drilling` | répartition des trous par pièce |
| POST | `/api/drilling.csv` | export des positions de trous |
| POST/GET | `/api/jobs` | enregistre / liste les jobs |
| GET/DELETE | `/api/jobs/:id` | lit / supprime un job |
| GET | `/api/health` | sonde |

## Architecture (3 conteneurs)

- **frontend** — Nginx sert le build React et proxifie `/api/` vers le backend (port 80).
- **backend** — API Rust/Axum (port 8080), variable `DATABASE_URL`.
- **db** — PostgreSQL.

Le code est découplé : le frontend ne parle au backend que via l'API REST `/api/*`.

## Déploiement (devopsweb.dev)

Brancher l'URL dédiée sur le conteneur **frontend** (port 80) — il sert le React et relaie l'API.
Le **backend** n'a pas besoin d'être exposé publiquement (accès interne via le réseau Docker).
Variables backend : `DATABASE_URL`, `PORT`.
