// (c) 2026 GestEase Technologie Inc. - Licence MIT (voir LICENSE).
// Contenu d'aide par section (affiché dans une modale via SectionHelp).
import type { ReactNode } from 'react'

export interface HelpTopic {
  title: string
  body: ReactNode
}

export const HELP = {
  base: {
    title: 'Données de base',
    body: (
      <>
        <p>Les réglages généraux appliqués à tout le calcul.</p>
        <ul>
          <li>
            <strong>Numéro de job</strong> — identifiant du travail. Saisissez-le au clavier ou
            scannez un <strong>code-barres</strong> (le lecteur agit comme un clavier : le champ se
            remplit puis valide).
          </li>
          <li>
            <strong>Trait de scie (kerf)</strong> — épaisseur de matière retirée à chaque coupe (par
            défaut 1/8″). Elle est soustraite à chaque tronçon : plus le kerf est grand, plus on
            perd de matière.
          </li>
          <li>
            <strong>Seuil de chute réutilisable</strong> — une chute plus grande ou égale à ce seuil
            est considérée <strong>récupérable</strong> (retour en stock) et n'est donc pas comptée
            comme une perte. En dessous, c'est une perte réelle.
          </li>
        </ul>
      </>
    ),
  },
  stock: {
    title: 'Longueurs de stock disponibles',
    body: (
      <>
        <p>L'inventaire des barres brutes dans lesquelles on découpe.</p>
        <ul>
          <li>Ajoutez chaque longueur disponible (ex. 16 pi, 20 pi) avec une étiquette.</li>
          <li>
            Colonne <strong>Dispo</strong> : laissez <strong>vide</strong> pour une quantité
            illimitée, sinon indiquez le nombre de barres en stock.
          </li>
          <li>
            Avec plusieurs longueurs, l'optimiseur choisit automatiquement le{' '}
            <strong>meilleur mélange</strong> pour minimiser la perte.
          </li>
        </ul>
      </>
    ),
  },
  cutlist: {
    title: 'Liste de coupe',
    body: (
      <>
        <p>Les pièces à produire. Une ligne par type de pièce.</p>
        <ul>
          <li>
            <strong>partID</strong> : identifiant de la pièce · <strong>model</strong> : profil de
            référence · <strong>longueur</strong> · <strong>quantité</strong>.
          </li>
          <li>
            <strong>Importer Excel / CSV</strong> remplit le tableau automatiquement.{' '}
            <strong>Modèle d'import</strong> télécharge un gabarit prêt à remplir.
          </li>
          <li>
            Les longueurs acceptent les formats <code>6</code>, <code>5'</code>, <code>60"</code>,{' '}
            <code>3-1/2"</code>, <code>1524mm</code>.
          </li>
        </ul>
      </>
    ),
  },
  drilling: {
    title: 'Perçage — répartition des trous',
    body: (
      <>
        <p>
          Paramètres de perçage appliqués à <strong>toutes</strong> les pièces. Le nombre de trous
          s'ajuste automatiquement selon la longueur de chaque pièce.
        </p>
        <ul>
          <li>
            <strong>Entraxe</strong> — distance centre-à-centre entre deux trous consécutifs.
          </li>
          <li>
            <strong>Marge de bout min / max</strong> — distance autorisée entre l'extrémité et le
            centre du trou extrême (le « point de départ »). La répartition est symétrique : on
            retient le nombre de trous qui donne la plus grande marge valide.
          </li>
        </ul>
      </>
    ),
  },
  result: {
    title: 'Résultat — plan de coupe',
    body: (
      <>
        <p>Le plan optimal pour produire toutes les pièces avec le moins de matière.</p>
        <ul>
          <li>Chaque barre montre les pièces (couleurs) et la chute en bout.</li>
          <li>
            Chute <span style={{ color: 'var(--reuse)' }}>verte</span> = réutilisable ;{' '}
            <span style={{ color: 'var(--warn)' }}>beige</span> = perte réelle.
          </li>
          <li>
            <strong>Utilisation</strong> = part de matière réellement utilisée. <strong>Exporter</strong>{' '}
            génère un CSV pour l'atelier ; <strong>Enregistrer</strong> conserve le job dans
            l'historique.
          </li>
        </ul>
      </>
    ),
  },
  drillplan: {
    title: 'Plan de perçage',
    body: (
      <>
        <p>Pour chaque pièce coupée, la position de chaque trou.</p>
        <ul>
          <li>
            La barre affiche les trous ; le <strong>* / point vert</strong> = trou central.
          </li>
          <li>
            Les <strong>cotes</strong> indiquent la marge de début, le 1<sup>er</sup> entraxe et la
            marge de fin (les points de départ depuis chaque extrémité).
          </li>
          <li>Le tableau dépliable donne chaque trou mesuré depuis A et depuis B.</li>
        </ul>
      </>
    ),
  },
  history: {
    title: 'Historique des jobs',
    body: (
      <>
        <p>Les jobs enregistrés en base de données.</p>
        <ul>
          <li>
            <strong>Charger</strong> réinjecte un job (stock, liste, paramètres, résultat) dans
            l'écran.
          </li>
          <li>
            <strong>Supprimer</strong> le retire définitivement. <strong>Rafraîchir</strong> remet
            la liste à jour.
          </li>
        </ul>
      </>
    ),
  },
} satisfies Record<string, HelpTopic>
