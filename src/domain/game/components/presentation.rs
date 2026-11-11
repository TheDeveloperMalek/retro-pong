

use bevy::prelude::*;
use crate::game::Screen;
use std::collections::HashMap;

#[derive(Component)]
pub struct ScreenUi;


#[derive(Component)]
pub struct PauseOverlay;

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct MenuItem(pub usize);

#[derive(Component)]
pub struct HudScore;

#[derive(Component)]
pub struct HudStatus;

#[derive(Component)]
pub struct HudSideLabel(pub crate::game::components::Side);

#[derive(Component)]
pub struct HudInventoryKeyHint {
    pub side: crate::game::components::Side,
    pub index: usize,
}

#[derive(Component)]
pub struct StarField {
    pub speed: f32,
}

#[derive(Component)]
pub struct CelebrationParticle {
    pub velocity: Vec2,
    pub remaining: f32,
}

#[derive(Resource, Default)]
pub struct MenuSelection {
    pub selected: usize,
    pub return_to: Option<Screen>,
    
    pub pause_return_to: Option<Screen>,
}

#[derive(Resource, Default)]
pub struct PowerUpEditorSelection(pub usize);


#[derive(Component)]
pub struct PowerUpHudSlot {
    pub side: super::Side,
    pub index: usize,
    pub displayed_kind: Option<super::PowerUpKind>,
}


#[derive(Component)]
pub struct PowerUpHudTooltip;

#[derive(Component)]
pub struct LanguageFlagPixel;

#[derive(Component)]
pub struct MainMenuLabel(pub usize);


#[derive(Component)]
pub struct MainMenuGithubBar {
    pub height: f32,
}

#[derive(Component)]
pub struct ClickRippleDot {
    pub origin: Vec2,
    pub angle: f32,
    pub remaining: f32,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct ClickBurstParticle {
    pub velocity: Vec2,
    pub remaining: f32,
    pub lifetime: f32,
}

#[derive(Resource, Default)]
pub struct ClickEffectCooldown(pub f32);

#[derive(Component)]
pub struct TransientNotice {
    pub remaining: f32,
}



#[derive(Resource, Default)]
pub struct ControlCapture {
    pub action_index: Option<usize>,
    pub menu_action_index: Option<usize>,
    
    pub gamepad_action_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActiveInputDevice {
    #[default]
    Keyboard,
    Gamepad,
    Mouse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputContext {
    #[default]
    Menu,
    PauseMenu,
    Rebinding,
    TextEntry,
    Gameplay,
}



#[derive(Resource, Default)]
pub struct NavigationState {
    pub active_device: ActiveInputDevice,
    pub context: InputContext,
    pub focus_by_screen: HashMap<Screen, usize>,
    pub last_screen: Option<Screen>,
    pub last_selected: usize,
    pub stick_up: bool,
    pub stick_down: bool,
    pub stick_left: bool,
    pub stick_right: bool,
    pub last_mouse_position: Option<Vec2>,
}



#[derive(Component)]
pub struct SocialIcon {
    
    pub selection_index: usize,
}

#[derive(Component)]
pub struct SocialIconLabel {
    pub selection_index: usize,
}

#[derive(Component)]
pub struct DeveloperPongBall {
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct DeveloperPongPaddle {
    pub side: f32,
    pub follow_speed: f32,
}

#[derive(Resource, Default)]
pub struct DeveloperDocumentState {
    pub closing: bool,
    pub close_elapsed: f32,
}


#[derive(Resource, Default)]
pub struct SkinSelection {
    
    pub tab: u8,
}

#[derive(Component)]
pub struct SkinPreview(pub bool);

#[derive(Component)]
pub struct NavigationHint;


#[derive(Component)]
pub struct UiScaleSliderKnob;



#[derive(Component, Clone)]
pub struct UiScaleBaseline {
    pub transform: Transform,
    pub font_size_px: Option<f32>,
    pub sprite_custom_size: Option<Vec2>,
    
    pub text_bounds: Option<(Option<f32>, Option<f32>)>,
}

#[derive(Component)]
pub struct ArenaPreviewTitle;

#[derive(Component)]
pub struct ArenaPreviewPanel;

#[derive(Component)]
pub struct ArenaPreviewAccent;

#[derive(Component)]
pub struct ArenaPreviewVisual;

#[derive(Component)]
pub struct ArenaPreviewSurface;

#[derive(Component)]
pub struct ArenaPreviewCounter;

#[derive(Component)]
pub struct ArenaPreviewPrevious;

#[derive(Component)]
pub struct ArenaPreviewNext;

#[derive(Component)]
pub struct ContextHelpTitle;

#[derive(Component)]
pub struct ContextHelpBody;




#[derive(Resource, Default)]
pub struct HighScoreFilters {
    
    pub match_type: u8,
    
    pub difficulty: u8,
    
    pub period: u8,
    
    pub page: usize,
}


#[derive(Resource, Default)]
pub struct LanguageDropdownState {
    pub open: bool,
    pub selected: usize,
}

#[derive(Component)]
pub struct LanguageDropdownItem(pub usize);

#[derive(Component)]
pub struct LanguageDropdownPanel;


#[derive(Component)]
pub struct PowerUpGridPreview {
    pub displayed_index: usize,
}


#[derive(Component)]
pub struct PowerUpGridPreviewLabel;



#[derive(Component)]
pub struct NetworkGuideIllustration(pub usize);
