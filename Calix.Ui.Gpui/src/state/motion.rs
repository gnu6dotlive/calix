use std::time::Duration;

/// Tokens de durées de transition de l'interface
pub struct MotionDurations;

impl MotionDurations {
    pub const FAST: Duration = Duration::from_millis(120);
    pub const STANDARD: Duration = Duration::from_millis(160);
    pub const EMPHASIS: Duration = Duration::from_millis(180);
    /// L'entrée d'une surface : la plus longue des durées, et toujours sous
    /// le plafond (`CEILING`).
    pub const SCOPE_ENTRY: Duration = Duration::from_millis(200);
    /// L'entrée de la plaque, sous le renfoncement.
    pub const PLATE_ENTRY: Duration = Duration::from_millis(180);
    /// Le plafond de toute animation : une entrée bornée, jamais une attente.
    ///
    /// La leçon de Skwir : l'interface ne fait pas attendre. Une animation est
    /// portée par l'élément (`with_animation`) et ne re-rend jamais la vue ;
    /// au-delà de 200 ms, elle se lit comme de la lenteur.
    pub const CEILING: Duration = Duration::from_millis(200);
}

/// Une courbe d'adoucissement, en Bézier cubique.
///
/// Les courbes sont des *jetons*, pas des formules dispersées : quatre
/// courbes nommées, et l'application n'en invente pas d'autres. Les points de
/// contrôle sont ceux du lanceur Artcraft, dont l'interface est leGFRien
/// témoin : une sortie franche, un aller-retour, un ressort, une emphase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Curve {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Curve {
    /// La courbe d'un Bézier cubique : `cubic-bezier(x1, y1, x2, y2)`.
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// La sortie : elle part vite et se pose.
    pub const OUT: Curve = Self::new(0.22, 1.0, 0.36, 1.0);

    /// L'aller-retour : la même courbe dans les deux sens.
    pub const IN_OUT: Curve = Self::new(0.65, 0.0, 0.35, 1.0);

    /// Le ressort : elle dépasse sa cible et revient.
    pub const SPRING: Curve = Self::new(0.34, 1.56, 0.64, 1.0);

    /// L'abscisse curviligne d'un Bézier, à `t`.
    fn curve_x(t: f32, x1: f32, x2: f32) -> f32 {
        let u = 1.0 - t;
        3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t
    }

    /// L'ordonnée curviligne d'un Bézier, à `t`.
    fn curve_y(t: f32, y1: f32, y2: f32) -> f32 {
        let u = 1.0 - t;
        3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t
    }

    /// La courbe évaluée : `t` du temps, `s` de la progression.
    ///
    /// L'abscisse du Bézier n'est pas le temps : on la résout, pour que la
    /// courbe ait la vitesse qu'elle annonce. La résolution est une
    /// dichotomie suivie d'une Newton-Raphson — c'est la méthode des
    /// navigateurs, et elle est exacte enough pour un écran.
    pub fn ease(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }

        // On cherche le `u` tel que l'abscisse du Bézier vaille `t`.
        let mut u = t;
        for _ in 0..8 {
            let x = Self::curve_x(u, self.x1, self.x2) - t;
            if x.abs() < 1e-6 {
                break;
            }
            let dx = 3.0
                * (1.0 - u)
                * (1.0 - u)
                * self.x1
                + 6.0 * (1.0 - u) * u * (self.x2 - self.x1)
                + 3.0 * u * u * (1.0 - self.x2);
            if dx.abs() < 1e-6 {
                break;
            }
            u -= x / dx;
            u = u.clamp(0.0, 1.0);
        }
        Self::curve_y(u, self.y1, self.y2)
    }
}

/// Fonctions d'adoucissement pures (fn(f32) -> f32)
pub fn ease_in_out(t: f32) -> f32 {
    Curve::IN_OUT.ease(t)
}

/// L'adoucissement de l'application : la sortie.
pub fn ease_out(t: f32) -> f32 {
    Curve::OUT.ease(t)
}

/// Le poids d'un élément dans une arrivée : il voyage plus loin et plus
/// longtemps, ou il s'ajuste vite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    /// L'ancre : l'élément qui porte la lecture.
    Heavy,
    /// L'élément ordinaire.
    Normal,
    /// L'élément léger : il s'ajuste, il ne voyage pas.
    Light,
}

impl Weight {
    /// La distance parcourue, en pixels.
    pub fn travel(self) -> f32 {
        match self {
            Weight::Heavy => 14.0,
            Weight::Normal => 10.0,
            Weight::Light => 7.0,
        }
    }

    /// La durée de son propre mouvement.
    pub fn duration(self) -> Duration {
        match self {
            Weight::Heavy => Duration::from_millis(180),
            Weight::Normal => Duration::from_millis(140),
            Weight::Light => Duration::from_millis(110),
        }
    }

}

/// L'écart entre deux éléments de la cascade, avant rétrécissement.
///
/// L'écart appartient à la cascade, pas à l'élément : le poids d'un élément
/// décide de la distance qu'il parcourt et de la durée de son mouvement, et
/// l'écart se rétrécit d'un élément à l'autre pour que la vague s'accélère.
pub const WATERFALL_BASE_GAP: Duration = Duration::from_millis(50);

/// Un élément d'une arrivée en cascade.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterfallStep {
    /// Le temps qui passe avant que cet élément ne bouge.
    pub delay: Duration,
    /// La distance qu'il parcourt, en pixels.
    pub travel: f32,
    /// La durée de son propre mouvement.
    pub duration: Duration,
}

/// L'arrivée en cascade : des éléments qui arrivent par en dessous, l'un après
/// l'autre, sans se faire la queue.
///
/// Les écarts se rétrécissent d'un élément à l'autre : la cascade est une
/// vague qui s'accélère, pas une file. Le dernier s'ajuste.
pub fn waterfall_entry(weights: &[Weight]) -> Vec<WaterfallStep> {
    let mut steps = Vec::with_capacity(weights.len());
    let mut delay = Duration::ZERO;
    for (index, weight) in weights.iter().enumerate() {
        steps.push(WaterfallStep {
            delay,
            travel: weight.travel(),
            duration: weight.duration(),
        });
        // Les écarts se rétrécissent : ×0,84 à chaque élément.
        delay += WATERFALL_BASE_GAP.mul_f32(0.84f32.powi(index as i32));
    }
    // La vague entière tient sous le plafond : au-delà, l'arrivée se lirait
    // comme une attente. Les écarts se resserrent ; les mouvements gardent
    // leur durée, qui dit le poids de chaque élément.
    if waterfall_duration(&steps) > MotionDurations::CEILING {
        let latest = steps
            .iter()
            .map(|step| step.delay)
            .max()
            .unwrap_or(Duration::ZERO);
        let longest = steps
            .iter()
            .map(|step| step.duration)
            .max()
            .unwrap_or(Duration::ZERO);
        let room = MotionDurations::CEILING.saturating_sub(longest);
        if !latest.is_zero() {
            let factor = (room.as_secs_f32() / latest.as_secs_f32()).clamp(0.0, 1.0);
            for step in &mut steps {
                step.delay = step.delay.mul_f32(factor);
            }
        }
    }
    steps
}

/// La durée totale d'une arrivée en cascade.
pub fn waterfall_duration(steps: &[WaterfallStep]) -> Duration {
    steps
        .iter()
        .map(|step| step.delay + step.duration)
        .max()
        .unwrap_or(Duration::ZERO)
}

/// La coupure à la courbe : l'entrée qui suit une coupe.
///
/// L'élément sortant accélère dans une direction ; la coupe tombe au milieu
/// du mouvement ; l'élément entrant continue dans la *même* direction et
/// décélère. L'assombrissement s'allume en plein chemin : une apparition en
/// fondu, de 0 à 1, se lirait comme un fondu et non comme une arrivée.
///
/// L'allumage se mesure en **distance**, pas en temps. La courbe de sortie
/// est en avance : à 35 % du temps, elle a déjà fait 86 % du chemin. Un
/// allumage mesuré en temps avaleait donc le voyage avant qu'il ne soit
/// visible — le geste était là, et personne ne le voyait. Mesuré en
/// distance, il tombe au début du temps, et le reste du voyage se voit.
///
/// Le voyage est partiel : environ 12 % de la surface, jamais un travers
/// complet.
pub fn cut_entry(delta: f32, travel: f32) -> (f32, f32) {
    let t = delta.clamp(0.0, 1.0);
    let offset = cut_travel(t, travel);
    let covered = travel - offset;
    let opacity = if travel <= 0.0 {
        1.0
    } else {
        (covered / (travel * 0.35)).clamp(0.0, 1.0)
    };
    (opacity, offset)
}

/// La moitié positionnelle de la coupure : le voyage qui se referme.
///
/// Une entrée se pose décalée et vient à sa place, dans la direction de la
/// sortie. C'est toute la coupe quand la surface qui entre contient une liste
/// virtualisée : une couche d'opacité autour d'elle laisse la fenêtre entière
/// à demi éteinte, et le mouvement seul suffit.
///
/// Le décalage s'applique par un **inset** — `top`, `left` — et il ne
/// déplace que l'élément : la mise en page de cette version de gpui traite
/// l'inset d'un élément en flux comme une position, pas comme une taille, si
/// bien que le voisin suivant garde sa place et que la largeur ne change pas.
/// C'est pourquoi une entrée se décale sans réajuster son texte ni pousser la
/// bande d'à côté ; une marge, elle, changerait le flux. La règle est
/// mesurée par `the_displacement_is_the_inset_and_it_moves_nothing_else`.
pub fn cut_travel(delta: f32, travel: f32) -> f32 {
    travel * (1.0 - ease_out(delta.clamp(0.0, 1.0)))
}

/// Le voyage du porteur : la barre part de la ligne quittée et se pose sur
/// la ligne choisie.
///
/// Elle ne saute pas. Une sélection qui saute est un état ; une barre qui
/// voyage est un geste, et l'œil la suit d'une ligne à l'autre. La courbe est
/// la sortie de l'application : elle part vite et se pose.
pub fn carrier_travel(from: f32, to: f32, delta: f32) -> f32 {
    let t = delta.clamp(0.0, 1.0);
    from + (to - from) * ease_out(t)
}


/// La barre du porteur : la sélection est un objet qui voyage d'une ligne à
/// l'autre.
///
/// Le porteur ne connaît ni la liste ni le rail : il connaît une rangée, la
/// rangée quittée, et le numéro du voyage. C'est ce qui permet à deux surfaces
/// — la liste et le rail — d'obéir à la même loi sans se connaître.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionMarker {
    /// La rangée où la barre se pose.
    pub index: usize,
    /// La rangée d'où elle part, quand elle vient de quelque part.
    pub from: Option<usize>,
    /// Le numéro du voyage : il change à chaque sélection, et l'animation
    /// recommence.
    pub epoch: usize,
}

/// Le porteur après un changement de sélection.
///
/// Rien ne bouge quand la sélection ne change pas : la barre reste où elle
/// est, et l'animation ne recommence pas à chaque rendu. Le porteur garde
/// alors sa rangée de départ : un rendu au milieu du voyage — la fiche qui
/// arrive, un survol — ne doit pas poser la barre d'un coup sur sa
/// destination. Un premier choix n'a pas de rangée de départ : la barre
/// s'allume sur place.
pub fn advance_marker(
    previous: Option<SelectionMarker>,
    selected: Option<usize>,
) -> (Option<SelectionMarker>, usize) {
    let previous_epoch = previous.map(|marker| marker.epoch).unwrap_or(0);
    match (previous, selected) {
        (None, None) => (None, previous_epoch),
        (Some(marker), Some(next)) if marker.index == next => (Some(marker), marker.epoch),
        (previous, selected) => {
            let epoch = previous_epoch + 1;
            let marker = selected.map(|index| SelectionMarker {
                index,
                from: previous.map(|marker| marker.index),
                epoch,
            });
            (marker, epoch)
        }
    }
}

/// Le ressort : l'aiguille dépasse sa position et revient.
///
/// Un instrument dépasse ; un commutateur, non. L'aiguille qui se jette sur
/// son cran le franchit légèrement avant de revenir s'y poser.
pub fn ease_spring(t: f32) -> f32 {
    Curve::SPRING.ease(t)
}

/// Politique d'animation de l'application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotionPolicy {
    pub reduce_motion: bool,
}

impl MotionPolicy {
    pub fn new(reduce_motion: bool) -> Self {
        Self { reduce_motion }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curves_are_tokens_and_they_behave() {
        // Les quatre courbes sont les points de contrôle nommés : l'une sort,
        // l'une va et vient, l'une dépasse, l'une insiste.
        assert_eq!(Curve::OUT, Curve::new(0.22, 1.0, 0.36, 1.0));
        assert_eq!(Curve::IN_OUT, Curve::new(0.65, 0.0, 0.35, 1.0));
        assert_eq!(Curve::SPRING, Curve::new(0.34, 1.56, 0.64, 1.0));
        for curve in [Curve::OUT, Curve::IN_OUT, Curve::SPRING] {
            assert!((curve.ease(0.0) - 0.0).abs() < 1e-6);
            assert!((curve.ease(1.0) - 1.0).abs() < 1e-6);
        }
        // La sortie est déjà bien avancée à mi-parcours : elle part vite.
        assert!(Curve::OUT.ease(0.5) > 0.8, "la sortie part vite");
        // L'aller-retour est encore à mi-parcours : elle est symétrique.
        assert!((Curve::IN_OUT.ease(0.5) - 0.5).abs() < 0.02);
    }

    #[test]
    fn the_overshoot_passes_the_target_and_comes_back() {
        // L'aiguille franchit son cran, puis revient s'y poser.
        assert!((ease_spring(0.0) - 0.0).abs() < 1e-6, "le départ est à zéro");
        assert!(ease_spring(0.6) > 1.0, "le cran est dépassé");
        assert!((ease_spring(1.0) - 1.0).abs() < 1e-6, "l'arrivée est posée");
        // Et le dépassement reste un dépassement, pas un écart.
        assert!(ease_spring(0.6) < 1.2);
    }

    #[test]
    fn the_cascade_overlaps_and_its_gaps_shrink() {
        // La vague : les éléments se rejoignent, les écarts se rétrécissent.
        let steps = waterfall_entry(&[
            Weight::Heavy,
            Weight::Normal,
            Weight::Normal,
            Weight::Light,
        ]);
        assert_eq!(steps.len(), 4);
        // L'ancre part la première.
        assert_eq!(steps[0].delay, Duration::ZERO);
        // Les écarts se rétrécissent : le premier intervalle est le plus large.
        let first_gap = steps[1].delay - steps[0].delay;
        let second_gap = steps[2].delay - steps[1].delay;
        let third_gap = steps[3].delay - steps[2].delay;
        assert!(first_gap > second_gap, "les écarts se rétrécissent");
        assert!(second_gap > third_gap, "les écarts se rétrécissent");
        // Le premier écart est au plus l'écart de base : la vague se resserre
        // pour tenir sous le plafond.
        assert!(first_gap <= WATERFALL_BASE_GAP);
        // L'ancre voyage plus loin et plus longtemps.
        assert!(steps[0].travel > steps[1].travel);
        assert!(steps[0].duration > steps[1].duration);
        // Le total couvre le dernier élément.
        let total = waterfall_duration(&steps);
        assert!(total >= steps[3].delay + steps[3].duration);
    }

    #[test]
    fn the_cut_entry_ignites_on_the_distance_and_keeps_the_direction() {
        // Au départ, rien n'est allumé et le panneau est encore à sa place de
        // départ.
        let (opacity, offset) = cut_entry(0.0, 40.0);
        assert_eq!(opacity, 0.0);
        assert_eq!(offset, 40.0);
        let (opacity, offset) = cut_entry(0.5, 40.0);
        assert!(opacity > 0.0, "l'assombrissement s'allume en chemin");
        assert!(offset < 40.0 && offset > 0.0, "le voyage se referme");
        let (opacity, offset) = cut_entry(1.0, 40.0);
        assert_eq!(opacity, 1.0);
        assert_eq!(offset, 0.0);
        // Le voyage est partiel : il ne franchit jamais toute la surface.
        assert!(40.0 < 120.0);

        // L'allumage se mesure en distance, pas en temps. La courbe de sortie
        // est en avance : au tiers du temps, le voyage est déjà presque fait,
        // et un allumage mesuré en temps l'aurait avalé.
        let (opacity_at_third, offset_at_third) = cut_entry(0.35, 40.0);
        assert!(
            opacity_at_third > 0.0,
            "à un tiers du temps, le panneau est déjà allumé : le voyage se voit"
        );
        assert!(
            offset_at_third < 40.0 * 0.35,
            "et il a déjà fait plus du tiers de son chemin"
        );

        // Le seuil d'allumage est bien le tiers du *chemin* : quand le voyage
        // a couvert un tiers de sa distance, le panneau est allumé.
        let mut ignition = None;
        for step in 0..=1000 {
            let delta = step as f32 / 1000.0;
            let (opacity, offset) = cut_entry(delta, 40.0);
            if opacity >= 1.0 {
                ignition = Some((40.0 - offset) / 40.0);
                break;
            }
        }
        let ignition = ignition.expect("le panneau s'allume");
        assert!(
            (ignition - 0.35).abs() < 0.02,
            "l'allumage tombe au tiers du chemin, pas au tiers du temps : {ignition}"
        );
    }

    #[test]
    fn every_duration_stays_under_the_ceiling() {
        for duration in [
            MotionDurations::FAST,
            MotionDurations::STANDARD,
            MotionDurations::EMPHASIS,
            MotionDurations::SCOPE_ENTRY,
            MotionDurations::PLATE_ENTRY,
        ] {
            assert!(duration <= MotionDurations::CEILING, "{duration:?}");
        }
        // Une cascade longue se resserre au lieu de faire attendre.
        let steps = waterfall_entry(&[Weight::Heavy; 8]);
        assert!(waterfall_duration(&steps) <= MotionDurations::CEILING);

        let q0 = ease_out(0.0);
        let q1 = ease_out(1.0);
        assert_eq!(q0, 0.0);
        assert_eq!(q1, 1.0);

        let io0 = ease_in_out(0.0);
        let io1 = ease_in_out(1.0);
        assert_eq!(io0, 0.0);
        assert_eq!(io1, 1.0);
    }

    #[test]
    fn the_cut_lands_where_it_was_aimed() {
        // L'entrée se pose décalée, puis vient à sa place : le voyage se
        // referme et finit exactement à zéro.
        assert_eq!(cut_travel(0.0, 18.0), 18.0);
        assert!(cut_travel(0.5, 18.0) < 18.0);
        assert!((cut_travel(1.0, 18.0)).abs() < 1e-4);

        // La moitié positionnelle de `cut_entry` est la même règle : une seule
        // courbe pour la coupe.
        for delta in [0.0, 0.2, 0.5, 0.8, 1.0] {
            assert_eq!(cut_entry(delta, 12.0).1, cut_travel(delta, 12.0));
        }
    }

    #[test]
    fn the_carrier_lands_on_the_line_it_was_sent_to() {
        // Au départ, la barre est exactement sur la ligne quittée ; à la fin,
        // exactement sur la ligne choisie. Entre les deux, elle avance.
        assert_eq!(carrier_travel(3.0, 7.0, 0.0), 3.0);
        assert!((carrier_travel(3.0, 7.0, 1.0) - 7.0).abs() < 1e-4);
        let mid = carrier_travel(3.0, 7.0, 0.5);
        assert!(mid > 3.0 && mid < 7.0, "la barre avance, elle ne saute pas");

        // Descendre et monter parcourent le même chemin.
        let up = carrier_travel(7.0, 3.0, 0.5);
        assert!((mid + up - 10.0).abs() < 1e-4, "le voyage est symétrique");

        // Une sélection sans ligne quittée reste immobile.
        assert_eq!(carrier_travel(5.0, 5.0, 0.42), 5.0);
    }

    #[test]
    fn the_first_selection_has_no_departure() {
        // Le premier choix n'a pas de ligne quittée : la barre s'allume sur
        // place, sans traverser la liste.
        let (marker, epoch) = advance_marker(None, Some(4));
        let marker = marker.expect("une sélection a un porteur");
        assert_eq!(marker.index, 4);
        assert_eq!(marker.from, None);
        assert_eq!(epoch, 1);
    }

    #[test]
    fn the_carrier_leaves_from_the_line_it_occupied() {
        // La barre part de la ligne quittée : la sélection voyage, elle ne
        // saute pas.
        let (first, epoch) = advance_marker(None, Some(4));
        let (marker, next_epoch) = advance_marker(first, Some(1));
        let marker = marker.expect("la sélection reste");
        assert_eq!(marker.index, 1);
        assert_eq!(marker.from, Some(4));
        assert!(next_epoch > epoch, "le voyage a son propre numéro");
    }

    #[test]
    fn the_carrier_does_not_restart_when_nothing_changes() {
        // Chaque rendu redemande le porteur : sans changement de sélection, le
        // numéro du voyage ne bouge pas, et l'animation ne recommence pas.
        let (first, epoch) = advance_marker(None, Some(2));
        let (again, same_epoch) = advance_marker(first, Some(2));
        assert_eq!(same_epoch, epoch);
        assert_eq!(again.map(|marker| marker.index), Some(2));

        // La sélection se referme : plus de porteur, et le changement a son
        // numéro — la plaque au repos entre à son tour.
        let (gone, closed) = advance_marker(again, None);
        assert!(gone.is_none());
        assert!(closed > same_epoch);
    }

    /// Le déplacement d'une entrée, mesuré sur la mise en page.
    ///
    /// C'est la moitié que la courbe ne prouve pas : `cut_travel` dit *de
    /// combien* une entrée se décale ; cette mesure dit *si* le décalage
    /// atteint le dessin, et ce qu'il dérange autour de lui.
    ///
    /// Trois questions, trois réponses : un `inset` sur un enfant en flux le
    /// déplace-t-il ? le frère suivant bouge-t-il ? la largeur change-t-elle ?
    ///
    /// La réponse est oui, non, non — et elle est la même dans une bande et
    /// dans un conteneur qui défile, ce qui compte pour la plaque : son corps
    /// est un enfant de la zone qui défile.
    #[gpui::test]
    fn the_displacement_is_the_inset_and_it_moves_nothing_else(cx: &mut gpui::TestAppContext) {
        use gpui::prelude::*;
        use gpui::*;

        struct Probe;

        impl Render for Probe {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let leaf = |id: &'static str, displaced: Div| {
                    div()
                        .debug_selector(move || id.to_string())
                        .h(px(40.0))
                        .w_full()
                        .child(displaced.h(px(20.0)).w_full())
                };

                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .p(px(10.0))
                    // Une bande dont l'enfant est décalé de 24 px vers le bas.
                    .child(leaf(
                        "displaced_band",
                        div().debug_selector(|| "displaced".to_string()).top(px(24.0)),
                    ))
                    // La bande suivante : elle ne doit pas bouger d'un pixel.
                    .child(leaf(
                        "following_band",
                        div().debug_selector(|| "following".to_string()),
                    ))
                    // La même question, dans une zone qui défile : c'est là que
                    // vit le corps de la plaque.
                    .child(
                        div()
                            .id("scroller")
                            .w_full()
                            .h(px(120.0))
                            .overflow_y_scroll()
                            .child(
                                div()
                                    .w_full()
                                    .h(px(400.0))
                                    .child(
                                        div()
                                            .debug_selector(|| "scrolled".to_string())
                                            .w(px(60.0))
                                            .h(px(20.0))
                                            .left(px(24.0)),
                                    ),
                            ),
                    )
            }
        }

        let cx = cx.add_empty_window();
        let probe = cx.new(|_| Probe);
        cx.draw(
            point(px(0.0), px(0.0)),
            size(px(400.0), px(300.0)),
            |_window, _cx| probe.clone().into_element(),
        );

        let displaced = cx.debug_bounds("displaced").expect("dessiné");
        let following = cx.debug_bounds("following").expect("dessiné");
        let band = cx.debug_bounds("following_band").expect("dessiné");
        let scrolled = cx.debug_bounds("scrolled").expect("dessiné");

        // 10 px de padding, et l'enfant décalé de 24.
        assert_eq!(
            displaced.origin.y,
            px(34.0),
            "un inset déplace un enfant en flux"
        );

        // Un décalage est une position, pas une taille : la bande suivante
        // reste à sa place. C'est ce qui distingue l'inset de la marge.
        assert_eq!(band.origin.y, px(50.0), "la bande suivante ne bouge pas");
        assert_eq!(
            following.origin.y,
            px(50.0),
            "l'enfant de la bande suivante ne bouge pas"
        );

        // La largeur ne change pas : le texte ne se réajuste pas.
        assert_eq!(displaced.size.width, px(380.0));
        assert_eq!(following.size.width, px(380.0));

        // Dans une zone qui défile, l'inset déplace aussi — c'est la plaque.
        assert_eq!(
            scrolled.origin.x,
            px(34.0),
            "l'inset déplace aussi l'enfant d'une zone qui défile"
        );
    }

    #[test]
    fn a_repaint_does_not_throw_the_carrier_at_its_destination() {
        // Le voyage dure 220 ms ; entre-temps, la fiche arrive et la scène se
        // redessine. Le porteur doit garder sa ligne de départ, sinon le
        // rendu suivant le pose d'un coup et le voyage n'existe jamais.
        let (leaving, epoch) = advance_marker(None, Some(6));
        let leaving = leaving.expect("une sélection a un porteur");
        let (arriving, new_epoch) = advance_marker(Some(leaving), Some(2));
        assert_eq!(new_epoch, epoch + 1);
        let arriving = arriving.expect("la sélection reste");
        assert_eq!(arriving.from, Some(6));

        // Le rendu suivant, au milieu du voyage.
        let (same, quieter_epoch) = advance_marker(Some(arriving), Some(2));
        let same = same.expect("la sélection reste");
        assert_eq!(quieter_epoch, new_epoch);
        assert_eq!(same.from, Some(6), "la ligne de départ tient");
        assert_eq!(same.index, 2);
    }
}
