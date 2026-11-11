

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::text::TextBounds;
use crate::{
    game::components::*,
    localization::tr,
    persistence::{Language, Settings},
    ui::UiFont,
};

pub fn update_hud(
    mut commands: Commands,
    session: Res<MatchSession>,
    clock: Res<MechanicsClock>,
    inventory: Res<PowerUpInventory>,
    settings: Res<Settings>,
    balls: Query<Option<&BallBehavior>, With<Ball>>,
    mut score: Query<(&mut Text2d, &mut Transform), (With<HudScore>, Without<HudStatus>, Without<PowerUpHudSlot>)>,
    mut status: Query<(&mut Text2d, &mut Transform, &mut TextBounds), (With<HudStatus>, Without<HudScore>, Without<PowerUpHudSlot>)>,
    asset_server: Res<AssetServer>,
    mut inventory_slots: Query<(&mut PowerUpHudSlot, &mut Sprite, &mut Transform), Without<PowerUpHudTooltip>>,
    mut side_labels: Query<(&HudSideLabel, &mut Transform), (Without<HudScore>, Without<HudStatus>, Without<PowerUpHudSlot>, Without<HudInventoryKeyHint>)>,
    mut key_hints: Query<(&HudInventoryKeyHint, &mut Transform), (Without<HudScore>, Without<HudStatus>, Without<PowerUpHudSlot>, Without<HudSideLabel>)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    
    
    
    mut tooltip: Query<
        (Entity, &mut Text2d, &mut Transform, &mut TextColor),
        (With<PowerUpHudTooltip>, Without<HudScore>, Without<HudStatus>, Without<HudSideLabel>, Without<HudInventoryKeyHint>),
    >,
    ui_font: Res<UiFont>,
) {
    let language = settings.language;
    let ui_scale = settings.ui_scale.clamp(0.80, 1.10);
    let viewport = windows.single().ok().and_then(|window| {
        let (camera, camera_transform) = cameras.single().ok()?;
        let top_left = camera.viewport_to_world_2d(camera_transform, Vec2::new(24.0, 24.0)).ok()?;
        let bottom_right = camera.viewport_to_world_2d(camera_transform, Vec2::new((window.width() - 24.0).max(24.0), (window.height() - 24.0).max(24.0))).ok()?;
        Some((top_left.x, bottom_right.x, top_left.y, bottom_right.y))
    });
    let cursor_world = windows.single().ok().and_then(|window| window.cursor_position()).and_then(|cursor| {
        let (camera, camera_transform) = cameras.single().ok()?;
        camera.viewport_to_world_2d(camera_transform, cursor).ok()
    });
    let capacity = settings.inventory_slots.clamp(1, 5) as usize;
    let anchors = viewport.map(|(left, right, top, bottom)| {
        let left_group = left + 150.0;
        let right_group = right - 150.0;
        (left_group, right_group, top - 26.0, top - 55.0, top - 73.0, bottom + 28.0, top - 34.0)
    });
    let mut hovered_powerup: Option<(String, Vec3, Color)> = None;
    for (mut hud_slot, mut sprite, mut transform) in &mut inventory_slots {
        if let Some((left_group, right_group, _, slot_y, _, _, _)) = anchors {
            let half = (capacity.saturating_sub(1) as f32 * 25.0) * 0.5;
            transform.translation.x = if hud_slot.side == Side::Player { left_group - half + hud_slot.index as f32 * 25.0 } else { right_group + half - hud_slot.index as f32 * 25.0 };
            transform.translation.y = slot_y;
        }
        let kind = inventory.for_side(hud_slot.side).get(hud_slot.index).map(|slot| slot.kind);
        if kind != hud_slot.displayed_kind {
            hud_slot.displayed_kind = kind;
            let path = match kind {
                Some(kind) => format!("powerups/{}/frame_0.png", kind.pixel_art_key()),
                None => "powerups/slot_empty.png".to_owned(),
            };
            sprite.image = asset_server.load(path);
            sprite.color = Color::srgb(1.0, 1.0, 1.0);
        }
        if let (Some(cursor), Some(kind)) = (cursor_world, kind) {
            let center = transform.translation.truncate();
            if (cursor - center).abs().max_element() <= 14.0 * ui_scale {
                let charges = inventory.for_side(hud_slot.side).get(hud_slot.index).map(|slot| slot.charges).unwrap_or(1);
                hovered_powerup = Some((format!("{}  ×{}", powerup_label(kind, language), charges), transform.translation, kind.color()));
            }
        }
    }
    if let Some((label, position, accent)) = hovered_powerup {
        if let Ok((_, mut text, mut transform, mut text_color)) = tooltip.single_mut() {
            text.0 = label;
            transform.translation = Vec3::new(position.x, position.y - 28.0 * ui_scale, 40.0);
            text_color.0 = accent;
        } else {
            commands.spawn((
                GameEntity,
                PowerUpHudTooltip,
                Text2d::new(label),
                TextFont { font: ui_font.source.clone(), font_size: FontSize::Px(8.0), font_smoothing: FontSmoothing::None, ..default() },
                TextLayout::justify(Justify::Center),
                TextColor(accent),
                Transform::from_xyz(position.x, position.y - 28.0 * ui_scale, 40.0),
            ));
        }
    } else {
        for (entity, _, _, _) in &mut tooltip {
            commands.entity(entity).despawn();
        }
    }
    if let Some((_, _, _, _, _, _, score_y)) = anchors {
        if let Ok((mut text, mut transform)) = score.single_mut() {
            transform.translation.y = score_y;
            text.0 = if matches!(session.mode, MatchMode::LocalVersus | MatchMode::Networked) {
            format!("P1 {:02}     :     {:02} P2", session.player_score, session.ai_score)
            } else {
                format!("{:02}     :     {:02}", session.player_score, session.ai_score)
            };
        }
    }
    if let Some((left_group, right_group, label_y, _, hint_y, status_y, _)) = anchors {
        for (label, mut transform) in &mut side_labels {
            transform.translation.x = if label.0 == Side::Player { left_group } else { right_group };
            transform.translation.y = label_y;
        }
        for (hint, mut transform) in &mut key_hints {
            let half = (capacity.saturating_sub(1) as f32 * 25.0) * 0.5;
            transform.translation.x = if hint.side == Side::Player { left_group - half + hint.index as f32 * 25.0 } else { right_group + half - hint.index as f32 * 25.0 };
            transform.translation.y = hint_y;
        }
        if let Ok((mut text, mut transform, mut bounds)) = status.single_mut() {
            transform.translation.y = status_y;
            let available_width = viewport.map(|(left, right, _, _)| (right - left - 48.0).max(160.0)).unwrap_or(850.0);
            *bounds = TextBounds::from(Vec2::new(available_width, 42.0));
            text.0 = if session.mode == MatchMode::OpenPlay && session.open_play_sudden_death {
            tr(language, "SUDDEN DEATH")
        } else if session.mode == MatchMode::OpenPlay && session.phase == MatchPhase::Rally {
            format!("{}  {:02}:{:02}", tr(language, "TIME LEFT"), (session.open_play_remaining / 60.0).floor() as u32, (session.open_play_remaining % 60.0).floor() as u32)
        } else if session.phase == MatchPhase::PointPause {
            tr(language, "POINT!")
        } else if session.phase == MatchPhase::Finished {
            tr(language, "MATCH COMPLETE")
        } else {
            let ball_count = balls.iter().count();
            let attached_manual_ball = balls.iter().flatten().any(|behavior| behavior.attached);
            let mut status_parts = vec![format!("{} {}", tr(language, "BALLS"), ball_count)];
            for effect in &clock.effects {
                let side = match effect.owner { Side::Player => "P1", Side::Ai => "AI", Side::PlayerTwo => "P2" };
                status_parts.push(format!("{} {} {:.0}s", side, powerup_label(effect.kind, language), effect.remaining));
            }
            let active_side_inventory = if matches!(session.mode, MatchMode::SinglePlayer | MatchMode::OpenPlay) {
                inventory.player_one.iter().chain(inventory.ai.iter())
            } else {
                inventory.player_one.iter().chain(inventory.player_two.iter())
            };
            let hybrid_inventory = active_side_inventory.clone().any(|item| item.kind.requires_manual_activation());
            if settings.manual_powerup_activation || hybrid_inventory || attached_manual_ball
                || matches!(session.mode, MatchMode::LocalVersus | MatchMode::Networked)
            {
                let first = inventory.player_one.iter().map(|item| format!("{}x{}", powerup_label(item.kind, language), item.charges)).collect::<Vec<_>>().join(",");
                if matches!(session.mode, MatchMode::LocalVersus | MatchMode::Networked) {
                    
                    
                    let p1_release = settings.controls.binding(14).map(compact_key_label).unwrap_or_else(|| "?".into());
                    let p2_release = settings.controls.binding(15).map(compact_key_label).unwrap_or_else(|| "?".into());
                    status_parts.push(format!("P1 {p1_release} / P2 {p2_release} TO RELEASE"));
                } else {
                    status_parts.push(format!("{} [{}]", tr(language, "ITEMS"), first));
                    let slots = settings.inventory_slots.clamp(1, 5) as usize;
                    let keys = (0..slots).filter_map(|slot| settings.controls.binding(2 + slot).map(compact_key_label)).collect::<Vec<_>>().join("/");
                    status_parts.push(format!("{} {keys}", tr(language, "USE")));
                    if attached_manual_ball || inventory.player_one.iter().any(|item| item.kind == PowerUpKind::StickyBall || item.kind == PowerUpKind::BoomerangBall) {
                        let release = settings.controls.binding(14).map(compact_key_label).unwrap_or_else(|| "?".into());
                        status_parts.push(format!("{release} {}", tr(language, "TO RELEASE")));
                    }
                }
            } else {
                status_parts.push(format!("{} {}", tr(language, "FIRST TO"), session.winning_score));
                status_parts.push(tr(language, "ESC TO PAUSE"));
            }
            status_parts.join("  //  ")
            };
        }
    }
}

fn powerup_label(kind: PowerUpKind, language: Language) -> String {
    let key = match kind {
        PowerUpKind::Speed => "SPEED BOOST",
        PowerUpKind::Slow => "SLOW FIELD",
        PowerUpKind::Enlarge => "BIG PADDLE",
        PowerUpKind::DoubleBall => "DOUBLE BALL",
        PowerUpKind::Fireball => "FIREBALL",
        PowerUpKind::Snowball => "SNOWBALL",
        PowerUpKind::ReversalBall => "REVERSAL BALL",
        PowerUpKind::StickyBall => "STICKY BALL",
        PowerUpKind::BallStorm => "BALL STORM",
        PowerUpKind::BoomerangBall => "BOOMERANG BALL",
        PowerUpKind::MagneticField => "MAGNETIC FIELD",
        PowerUpKind::TeleportBall => "TELEPORT BALL",
        PowerUpKind::EnergyShield => "ENERGY SHIELD",
        PowerUpKind::Overdrive => "OVERDRIVE",
        PowerUpKind::PhantomBall => "PHANTOM BALL",
    };
    tr(language, key)
}


fn compact_key_label(key: &str) -> String {
    match key {
        "ArrowUp" => "UP".into(), "ArrowDown" => "DN".into(),
        "ArrowLeft" => "LT".into(), "ArrowRight" => "RT".into(),
        "Space" => "SPC".into(), "Enter" => "ENT".into(), "Escape" => "ESC".into(),
        value if value.starts_with("Digit") => value.trim_start_matches("Digit").to_owned(),
        value if value.starts_with("Key") => value.trim_start_matches("Key").to_owned(),
        value => value.to_owned(),
    }
}
