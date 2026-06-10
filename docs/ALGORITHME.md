# Algorithmes

Deux calculs indépendants, tous deux en **micromètres entiers** : l'optimisation de coupe
(découpe des extrusions) et la répartition du perçage (trous par pièce).

---

## 1. Optimisation de coupe — *1D cutting stock*

### Problème

À partir de longueurs de barres brutes (plusieurs longueurs possibles, dispo limitée ou
illimitée) et d'une liste de coupe (pièces × quantités), produire un plan de coupe qui
**minimise la matière brute consommée** — donc la perte et le résiduel.

### Modèle du trait de scie (kerf)

Chaque coupe consomme une largeur de lame `kerf`. Pour `k` pièces sur une barre :

```
Σ(longueurs des pièces) + k × kerf  ≤  longueur_barre
chute = longueur_barre − Σ(longueurs) − k × kerf
```

Convention **conservatrice** : un trait compté **par pièce** (squaring du bout + tronçonnage).
Elle ne sous-estime jamais la matière nécessaire.

### Objectif (ordre lexicographique, à minimiser)

1. pièces non placées (le moins possible) ;
2. **matière brute totale consommée** (= résiduel le plus court) ;
3. nombre de barres ;
4. perte réelle (chutes `< seuil`) — maximise donc la chute **réutilisable** concentrée.

Le « seuil de chute réutilisable » distingue une chute récupérable (retour stock, non comptée
comme perte) d'une perte réelle.

### Méthode

Problème NP-difficile, mais très traitable aux tailles d'atelier. Approche pragmatique :

1. **Heuristiques gloutonnes** — 4 variantes : *First-Fit* / *Best-Fit Decreasing* × choix du
   stock (plus petit suffisant / plus grand).
2. **Recherche locale** sous budget de temps :
   - *élimination de barre* : redistribuer toutes les coupes de la barre la moins remplie ;
   - *réduction de stock* : ramener chaque barre au plus petit stock disponible qui convient.
3. On retient la meilleure solution au sens de l'objectif ci-dessus.

C'est une approche **best-effort** (pas une preuve d'optimalité), mais elle trouve l'optimum sur
les cas usuels. Exemple de référence — barre 16 pi, kerf 1/8″, liste `1×3pi 1×4pi 2×6pi 6×5pi` :
**4 barres** (optimum prouvé : 588 po de pièces ne tiennent pas en 3 barres = 576 po),
utilisation 76,6 %.

---

## 2. Perçage — répartition symétrique des trous

Port fidèle du cahier des charges d'origine (`docs/specs/`). Appliqué à **chaque pièce coupée** ;
le nombre de trous `N` s'ajuste à la longueur.

### Règles

- Répartition **symétrique** autour du centre (`centre = longueur / 2`).
- **Entraxe** centre-à-centre `pas` constant.
- **Marge de bout** (bord → centre du trou extrême) — *le point de départ* :

```
span  = (N − 1) × pas
marge = (longueur − span) / 2          valide si  marge_min ≤ marge ≤ marge_max
```

- On retient le `N` donnant la **plus grande marge valide** (centrage maximal, trou extrême le
  plus loin du bord sans dépasser `marge_max`).
- **N impair** → un trou au centre exact ; **N pair** → les trous encadrent le centre à
  `± pas/2, ± 3·pas/2, …`.
- Sortie : position du centre de chaque trou **depuis l'extrémité A et depuis l'extrémité B**,
  + indicateur de trou central. Symétrie garantie : `depuis_A(trou 1) = depuis_B(dernier trou)`.

### Cas sans solution

Pour certaines longueurs, aucun `N` ne respecte simultanément `marge_min` et `marge_max`
(zones mortes liées au pas fixe) : la pièce est signalée « perçage impossible ».

### Exemple de référence

Profil 2400 mm, entraxe 112 mm, marges [40, 112] mm → **21 trous**, marge de bout **80 mm**,
trou central exactement à **1200 mm** des deux extrémités.
