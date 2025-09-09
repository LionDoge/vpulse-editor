
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::VariantArray as _;
use strum_macros::VariantArray;

pub trait PulseEnumTrait {
    fn to_str(self) -> &'static str;
    fn to_str_ui(&self) -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum PulseCursorCancelPriority {
    None,
    CancelOnSucceeded,
    #[default]
    SoftCancel,
    HardCancel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum PulseTraceContents {
    #[default]
    StaticLevel,
    Solid,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum PulseCollisionGroup {
    #[default]
    Default,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum ParticleAttachment {
    #[default]
    AbsOrigin,
    AbsOriginFollow,
    AbsCustomOrigin,
    AbsCustomOriginFollow,
    Point,
    PointFollow,
    EyesFollow,
    OverheadFollow,
    WorldOrigin,
    RootBoneFollow,
    RenderOriginFollow,
    MainView,
    WaterWake,
    CenterFollow,
    CustomGameState1,
    HealthBar,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum BaseExplosionTypes {
    #[default]
    Default,
    Grenade,
    Molotov,
    Fireworks,
    Gascan,
    GasCylinder,
    ExplosiveBarrel,
    Electrical,
    Emp,
    Shrapnel,
    SmokeGrenade,
    Flashbang,
    Tripmine,
    Ice,
    None,
    Custom,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum HitGroup {
    #[default]
    Invalid,
    Generic,
    Head,
    Chest,
    Stomach,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
    Neck,
    Unused,
    Gear,
    Special,
    T2BossFrontLeftLegWeakpoint,
    T2BossFrontRightLegWeakpoint,
    T2BossRearLeftLegWeakpoint,
    T2BossRearRightLegWeakpoint,
    T2BossHeadWeakpoint,
    T2BossBackWeakpoint,
    DroneBossDroneWeakpoint,
    HeadNoResist,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaEnumType {
    CursorCancelPriority,
    TraceContents,
    CollisionGroup,
    ParticleAttachment,
    BaseExplosionTypes,
    HitGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemaEnumValue {
    CursorCancelPriority(PulseCursorCancelPriority),
    TraceContents(PulseTraceContents),
    CollisionGroup(PulseCollisionGroup),
    ParticleAttachment(ParticleAttachment),
    BaseExplosionTypes(BaseExplosionTypes),
    HitGtoup(HitGroup),
}

impl FromStr for SchemaEnumType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "PulseCursorCancelPriority_t" => Ok(SchemaEnumType::CursorCancelPriority),
            "PulseTraceContents_t" => Ok(SchemaEnumType::TraceContents),
            "PulseCollisionGroup_t" => Ok(SchemaEnumType::CollisionGroup),
            "ParticleAttachment_t" => Ok(SchemaEnumType::ParticleAttachment),
            "BaseExplosionTypes_t" => Ok(SchemaEnumType::BaseExplosionTypes),
            "HitGroup_t" => Ok(SchemaEnumType::HitGroup),
            _ => Err(anyhow::anyhow!(
                "Unknown SchemaEnumType: {}",
                s
            )),
        }
    }
}

impl SchemaEnumType {
    pub fn get_all_types_as_enums(&self) -> Vec<SchemaEnumValue> {
        match self {
            SchemaEnumType::CursorCancelPriority => {
                PulseCursorCancelPriority::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::CursorCancelPriority(v))
                    .collect()
            }
            SchemaEnumType::TraceContents => {
                PulseTraceContents::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::TraceContents(v))
                    .collect()
            }
            SchemaEnumType::CollisionGroup => {
                PulseCollisionGroup::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::CollisionGroup(v))
                    .collect()
            }
            SchemaEnumType::ParticleAttachment => {
                ParticleAttachment::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::ParticleAttachment(v))
                    .collect()
            }
            SchemaEnumType::BaseExplosionTypes => {
                BaseExplosionTypes::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::BaseExplosionTypes(v))
                    .collect()
            }
            SchemaEnumType::HitGroup => {
                HitGroup::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::HitGtoup(v))
                    .collect()
            }
        }
    }
    pub fn to_str(self) -> &'static str {
        match self {
            SchemaEnumType::CursorCancelPriority => "PulseCursorCancelPriority_t",
            SchemaEnumType::TraceContents => "PulseTraceContents_t",
            SchemaEnumType::CollisionGroup => "PulseCollisionGroup_t",
            SchemaEnumType::ParticleAttachment => "ParticleAttachment_t",
            SchemaEnumType::BaseExplosionTypes => "BaseExplosionTypes_t",
            SchemaEnumType::HitGroup => "HitGroup_t",
        }
    }
    pub fn to_str_ui(self) -> &'static str {
        match self {
            SchemaEnumType::CursorCancelPriority => "Cursor Cancel Priority",
            SchemaEnumType::TraceContents => "Trace Contents",
            SchemaEnumType::CollisionGroup => "Collision Group",
            SchemaEnumType::ParticleAttachment => "Particle Attachment",
            SchemaEnumType::BaseExplosionTypes => "Base Explosion Types",
            SchemaEnumType::HitGroup => "Hit Group",
        }
    }
}

impl SchemaEnumValue {
    pub fn get_ui_name(&self) -> &'static str {
        match self {
            SchemaEnumValue::CursorCancelPriority(value) => value.to_str_ui(),
            SchemaEnumValue::TraceContents(value) => value.to_str_ui(),
            SchemaEnumValue::CollisionGroup(value) => value.to_str_ui(),
            SchemaEnumValue::ParticleAttachment(value) => value.to_str_ui(),
            SchemaEnumValue::BaseExplosionTypes(value) => value.to_str_ui(),
            SchemaEnumValue::HitGtoup(value) => value.to_str_ui(),
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            SchemaEnumValue::CursorCancelPriority(value) => value.to_str(),
            SchemaEnumValue::TraceContents(value) => value.to_str(),
            SchemaEnumValue::CollisionGroup(value) => value.to_str(),
            SchemaEnumValue::ParticleAttachment(value) => value.to_str(),
            SchemaEnumValue::BaseExplosionTypes(value) => value.to_str(),
            SchemaEnumValue::HitGtoup(value) => value.to_str(),
        }
    }
    pub fn default_from_type(typ: &SchemaEnumType) -> Self {
        match typ {
            SchemaEnumType::CursorCancelPriority => 
                SchemaEnumValue::CursorCancelPriority(PulseCursorCancelPriority::default()),
            SchemaEnumType::TraceContents => 
                SchemaEnumValue::TraceContents(PulseTraceContents::default()),
            SchemaEnumType::CollisionGroup => 
                SchemaEnumValue::CollisionGroup(PulseCollisionGroup::default()),
            SchemaEnumType::ParticleAttachment => 
                SchemaEnumValue::ParticleAttachment(ParticleAttachment::default()),
            SchemaEnumType::BaseExplosionTypes => 
                SchemaEnumValue::BaseExplosionTypes(BaseExplosionTypes::default()),
            SchemaEnumType::HitGroup =>
                SchemaEnumValue::HitGtoup(HitGroup::default()),
        }
    }
}

impl PulseEnumTrait for PulseCursorCancelPriority {
    fn to_str(self) -> &'static str {
        match self {
            PulseCursorCancelPriority::None => "None",
            PulseCursorCancelPriority::CancelOnSucceeded => "CancelOnSucceeded",
            PulseCursorCancelPriority::SoftCancel => "SoftCancel",
            PulseCursorCancelPriority::HardCancel => "HardCancel",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            PulseCursorCancelPriority::None => "Keep running normally.",
            PulseCursorCancelPriority::CancelOnSucceeded => "Kill after current node.",
            PulseCursorCancelPriority::SoftCancel => "Kill elegantly.",
            PulseCursorCancelPriority::HardCancel => "Kill immediately.",
        }
    }
}

impl PulseEnumTrait for PulseTraceContents {
    fn to_str(self) -> &'static str {
        match self {
            PulseTraceContents::StaticLevel => "STATIC_LEVEL",
            PulseTraceContents::Solid => "SOLID",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            PulseTraceContents::StaticLevel => "Static Level",
            PulseTraceContents::Solid => "Solid",
        }
    }
}

impl PulseEnumTrait for PulseCollisionGroup {
    fn to_str(self) -> &'static str {
        match self {
            PulseCollisionGroup::Default => "DEFAULT",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            PulseCollisionGroup::Default => "Default",
        }
    }
}

impl PulseEnumTrait for ParticleAttachment {
    fn to_str(self) -> &'static str {
        match self {
            ParticleAttachment::AbsOrigin => "PATTACH_ABSORIGIN",
            ParticleAttachment::AbsOriginFollow => "PATTACH_ABSORIGIN_FOLLOW",
            ParticleAttachment::AbsCustomOrigin => "PATTACH_CUSTOMORIGIN",
            ParticleAttachment::AbsCustomOriginFollow => "PATTACH_CUSTOMORIGIN_FOLLOW",
            ParticleAttachment::Point => "PATTACH_POINT",
            ParticleAttachment::PointFollow => "PATTACH_POINT_FOLLOW",
            ParticleAttachment::EyesFollow => "PATTACH_EYES_FOLLOW",
            ParticleAttachment::OverheadFollow => "PATTACH_OVERHEAD_FOLLOW",
            ParticleAttachment::WorldOrigin => "PATTACH_WORLDORIGIN",
            ParticleAttachment::RootBoneFollow => "PATTACH_ROOTBONE_FOLLOW",
            ParticleAttachment::RenderOriginFollow => "PATTACH_RENDERORIGIN_FOLLOW",
            ParticleAttachment::MainView => "PATTACH_MAIN_VIEW",
            ParticleAttachment::WaterWake => "PATTACH_WATERWAKE",
            ParticleAttachment::CenterFollow => "PATTACH_CENTER_FOLLOW",
            ParticleAttachment::CustomGameState1 => "PATTACH_CUSTOM_GAME_STATE_1",
            ParticleAttachment::HealthBar => "PATTACH_HEALTHBAR",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            ParticleAttachment::AbsOrigin => "Absolute Origin",
            ParticleAttachment::AbsOriginFollow => "Absolute Origin Follow",
            ParticleAttachment::AbsCustomOrigin => "Absolute Custom Origin",
            ParticleAttachment::AbsCustomOriginFollow => "Absolute Custom Origin Follow",
            ParticleAttachment::Point => "Point",
            ParticleAttachment::PointFollow => "Point Follow",
            ParticleAttachment::EyesFollow => "Eyes Follow",
            ParticleAttachment::OverheadFollow => "Overhead Follow",
            ParticleAttachment::WorldOrigin => "World Origin",
            ParticleAttachment::RootBoneFollow => "Root Bone Follow",
            ParticleAttachment::RenderOriginFollow => "Render Origin Follow",
            ParticleAttachment::MainView => "Main View",
            ParticleAttachment::WaterWake => "Water Wake",
            ParticleAttachment::CenterFollow => "Center Follow",
            ParticleAttachment::CustomGameState1 => "Custom Game State 1",
            ParticleAttachment::HealthBar => "Health Bar",
        }
    }
}

impl PulseEnumTrait for BaseExplosionTypes {
    fn to_str(self) -> &'static str {
        match self {
            BaseExplosionTypes::Default => "EXPLOSION_TYPE_DEFAULT",
            BaseExplosionTypes::Grenade => "EXPLOSION_TYPE_GRENADE",
            BaseExplosionTypes::Molotov => "EXPLOSION_TYPE_MOLOTOV",
            BaseExplosionTypes::Fireworks => "EXPLOSION_TYPE_FIREWORKS",
            BaseExplosionTypes::Gascan => "EXPLOSION_TYPE_GASCAN",
            BaseExplosionTypes::GasCylinder => "EXPLOSION_TYPE_GASCYLINDER",
            BaseExplosionTypes::ExplosiveBarrel => "EXPLOSION_TYPE_EXPLOSIVEBARREL",
            BaseExplosionTypes::Electrical => "EXPLOSION_TYPE_ELECTRICAL",
            BaseExplosionTypes::Emp => "EXPLOSION_TYPE_EMP",
            BaseExplosionTypes::Shrapnel => "EXPLOSION_TYPE_SHRAPNEL",
            BaseExplosionTypes::SmokeGrenade => "EXPLOSION_TYPE_SMOKEGRENADE",
            BaseExplosionTypes::Flashbang => "EXPLOSION_TYPE_FLASHBANG",
            BaseExplosionTypes::Tripmine => "EXPLOSION_TYPE_TRIPMINE",
            BaseExplosionTypes::Ice => "EXPLOSION_TYPE_ICE",
            BaseExplosionTypes::None => "EXPLOSION_TYPE_NONE",
            BaseExplosionTypes::Custom => "EXPLOSION_TYPE_CUSTOM",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            BaseExplosionTypes::Default => "Default",
            BaseExplosionTypes::Grenade => "Grenade",
            BaseExplosionTypes::Molotov => "Molotov",
            BaseExplosionTypes::Fireworks => "Fireworks",
            BaseExplosionTypes::Gascan => "Gascan",
            BaseExplosionTypes::GasCylinder => "Gascan",
            BaseExplosionTypes::ExplosiveBarrel => "Explosive Barrel",
            BaseExplosionTypes::Electrical => "Electrical",
            BaseExplosionTypes::Emp => "EMP",
            BaseExplosionTypes::Shrapnel => "Shrapnel",
            BaseExplosionTypes::SmokeGrenade => "Smoke Grenade",
            BaseExplosionTypes::Flashbang => "Flashbang",
            BaseExplosionTypes::Tripmine => "Tripmine",
            BaseExplosionTypes::Ice => "Ice",
            BaseExplosionTypes::None => "None",
            BaseExplosionTypes::Custom => "Custom",
        }
    }
}

impl PulseEnumTrait for HitGroup {
    fn to_str(self) -> &'static str {
        match self {
            HitGroup::Invalid => "HITGROUP_INVALID",
            HitGroup::Generic => "HITGROUP_GENERIC",
            HitGroup::Head => "HITGROUP_HEAD",
            HitGroup::Chest => "HITGROUP_CHEST",
            HitGroup::Stomach => "HITGROUP_STOMACH",
            HitGroup::LeftArm => "HITGROUP_LEFTARM",
            HitGroup::RightArm => "HITGROUP_RIGHTARM",
            HitGroup::LeftLeg => "HITGROUP_LEFTLEG",
            HitGroup::RightLeg => "HITGROUP_RIGHTLEG",
            HitGroup::Neck => "HITGROUP_NECK",
            HitGroup::Unused => "HITGROUP_UNUSED",
            HitGroup::Gear => "HITGROUP_GEAR",
            HitGroup::Special => "HITGROUP_SPECIAL",
            HitGroup::T2BossFrontLeftLegWeakpoint => "HITGROUP_T2_BOSS_FRONT_LEFT_LEG_WEAKPOINT",
            HitGroup::T2BossFrontRightLegWeakpoint => "HITGROUP_T2_BOSS_FRONT_RIGHT_LEG_WEAKPOINT",
            HitGroup::T2BossRearLeftLegWeakpoint => "HITGROUP_T2_BOSS_REAR_LEFT_LEG_WEAKPOINT",
            HitGroup::T2BossRearRightLegWeakpoint => "HITGROUP_T2_BOSS_REAR_RIGHT_LEG_WEAKPOINT",
            HitGroup::T2BossHeadWeakpoint => "HITGROUP_T2_BOSS_HEAD_WEAKPOINT",
            HitGroup::T2BossBackWeakpoint => "HITGROUP_T2_BOSS_BACK_WEAKPOINT",
            HitGroup::DroneBossDroneWeakpoint => "HITGROUP_DRONE_BOSS_DRONE_WEAKPOINT",
            HitGroup::HeadNoResist => "HITGROUP_HEAD_NO_RESIST",
            HitGroup::Count => "HITGROUP_COUNT",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        self.to_str()
    }
}

//----------------- CELL RELATED ENUMS -----------------//
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GeneralEnumChoice {
    SoundEventStartType(SoundEventStartType),
}
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum SoundEventStartType {
    Player,
    World,
    #[default]
    Entity,
}

impl GeneralEnumChoice {
    pub fn get_all_choices(&self) -> Vec<GeneralEnumChoice> {
        match self {
            GeneralEnumChoice::SoundEventStartType(_) => {
                SoundEventStartType::VARIANTS
                    .iter()
                    .map(|&v| GeneralEnumChoice::SoundEventStartType(v))
                    .collect()
            }
        }
    }
    pub fn to_str_ui(&self) -> &'static str {
        match self {
            GeneralEnumChoice::SoundEventStartType(value) => value.to_str_ui(),
        }
    }
}

impl PulseEnumTrait for SoundEventStartType {
    fn to_str(self) -> &'static str {
        match self {
            SoundEventStartType::Player => "SOUNDEVENT_START_PLAYER",
            SoundEventStartType::World => "SOUNDEVENT_START_WORLD",
            SoundEventStartType::Entity => "SOUNDEVENT_START_ENTITY",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            SoundEventStartType::Player => "Player",
            SoundEventStartType::World => "World",
            SoundEventStartType::Entity => "Entity",
        }
    }
}
