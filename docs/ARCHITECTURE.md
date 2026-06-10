# Architecture

Application web découplée en **3 conteneurs**, sur le modèle des autres projets maison
(`frontend` + `backend` + `db`). Le code est séparé : le frontend ne parle au backend que
via l'API REST `/api/*`.

```
Navigateur ──> frontend (Nginx :80)
                 ├── sert le build React (statique, SPA)
                 └── proxy /api/  ──> backend (Rust/Axum :8080)
                                          └── DATABASE_URL ──> db (PostgreSQL :5432)
```

| Conteneur | Image | Rôle | Port (local) |
|---|---|---|---|
| `frontend` | Nginx + build React | sert le SPA, relaie `/api` | 8099 → 80 |
| `backend`  | Rust/Axum | API REST + moteurs de calcul | 8097 → 8080 |
| `db`       | postgres:16-alpine | persistance des jobs | 5436 → 5432 |

## Principe d'unités

Toutes les longueurs sont stockées et calculées en **micromètres entiers (`i64`)**. Conversions
exactes : `1 po = 25 400 µm`, `1 pi = 304 800 µm`, `1 mm = 1 000 µm`. Aucune erreur d'arrondi sur
les fractions impériales. Les unités d'affichage/saisie (pi/po ↔ mm) sont une affaire de **frontend** ;
l'API échange en µm.

## Backend (Rust)

```
backend/src/
├── main.rs            Serveur Axum : routes, état (pool PG), démarrage, migrations
├── lib.rs             Déclaration des modules
├── api.rs             DTOs requête/réponse (import, export, drilling, save job…)
├── db.rs              sqlx : connexion, migration, CRUD des jobs (table `jobs`)
├── drilling.rs        Répartition symétrique des trous de perçage (cahier d'origine)
├── export.rs          Génération CSV : modèle d'import, plan de coupe, plan de perçage
├── import.rs          Lecture .xlsx (calamine) / .csv → liste de coupe
├── parse.rs           Analyse de longueurs en texte (pi/po, fractions, mm) → µm
└── optimizer/
    ├── mod.rs         API du moteur + tests
    ├── model.rs       Types : Stock, Part, BarPlan, Solution, Summary…
    ├── solver.rs      Optimisation 1D (cutting stock) : heuristiques + recherche locale
    └── units.rs       Conversions µm ↔ mm/po/pi
```

### Schéma de données (PostgreSQL)

Un seul tableau, longueurs et résultats en JSONB :

```sql
CREATE TABLE jobs (
    id          UUID PRIMARY KEY,
    job_number  TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    status      TEXT NOT NULL DEFAULT 'enregistre',
    settings    JSONB NOT NULL,   -- kerf, seuil, paramètres de perçage
    stocks      JSONB NOT NULL,   -- longueurs de stock
    parts       JSONB NOT NULL,   -- liste de coupe
    result      JSONB             -- plan de coupe calculé
);
```

Le schéma est appliqué automatiquement au démarrage (`db::migrate`). Sans `DATABASE_URL`,
le backend démarre quand même : seuls les endpoints `/api/jobs` renvoient `503`.

## Frontend (React + Vite + TypeScript)

```
frontend/src/
├── main.tsx           Point d'entrée React
├── App.tsx            État global + mise en page + appels API
├── api.ts             Client HTTP (fetch) typé
├── types.ts           Types partagés avec l'API
├── units.ts           Conversions + parsing + formatage des longueurs
├── LengthInput.tsx    Champ longueur (accepte fractions) + unité
├── ResultView.tsx     Résumé + visualisation des barres (SVG) + export
├── DrillingView.tsx   Plan de perçage par pièce (trous + cotes) + export
└── History.tsx        Historique des jobs (chargement / suppression)
```

## Endpoints API

| Méthode | Route | Rôle |
|--------|-------|------|
| POST | `/api/optimize` | calcule un plan de coupe (`Problem` → `Solution`) |
| POST | `/api/import` | multipart `file` + `unit` → liste de coupe |
| GET | `/api/template.csv` | modèle d'import (CSV) |
| POST | `/api/export.csv` | export du plan de coupe |
| POST | `/api/drilling` | répartition des trous par pièce |
| POST | `/api/drilling.csv` | export des positions de trous |
| POST/GET | `/api/jobs` | enregistre / liste les jobs |
| GET/DELETE | `/api/jobs/:id` | lit / supprime un job |
| GET | `/api/health` | sonde |

## Déploiement

L'URL publique se branche sur le conteneur **frontend** (port 80) ; il sert le SPA et relaie
`/api`. Le **backend** reste interne au réseau Docker (pas besoin d'exposition publique).
Variables backend : `DATABASE_URL`, `PORT`.
