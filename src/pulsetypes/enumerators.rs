
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum AILOD {
    #[default]
    High,
    Medium,
    Low,
    VeryLow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum NPCSTATE {
    #[default]
    Idle,
    Alert,
    Combat,
    Dead,
    Inert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum PulseNPCCondition {
    #[default]
    SeePlayer,
    LostPlayer,
    HearPlayer,
    PlayerPushing,
    NoPrimaryAmmo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum NPCFollowFormation {
    #[default]
    Default,
    CloseCircle,
    WideCircle,
    MediumCircle,
    Sidekick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum AIStrafing {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum AIVolumetricEventType {
    #[default]
    Combat,
    Player,
    Danger,
    BulletImpact,
    PhysicsDanger,
    MoveAway,
    PlayerVehicle,
    GlassBreak,
    PhysicsObject,
    WarnFriends,
    GunFire,
    Explosion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum DamageTypes {
    #[default]
    Generic,
    Crush,
    Bullet,
    Slash,
    Burn,
    Vehicle,
    Fall,
    Blast,
    Club,
    Shock,
    Sonic,
    EnergyBeam,
    Buckshot,
    Drown,
    Poison,
    Radiation,
    DrownRecover,
    Acid,
    Physgun,
    Dissolve,
    BlastSurface,
    Headshot,
    Crit,
    Buffed,
    Dot,
    GroundAura,
    Lethal,
    Dangerzone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum StanceType {
    #[default]
    Default,
    Crouching,
    Prone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum SharedMovementGait {
    #[default]
    Slow,
    Medium,
    Fast,
    VeryFast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum ChoreoLookAtSpeed {
    #[default]
    Slow,
    Medium,
    Fast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, VariantArray)]
pub enum ChoreoLookAtMode {
    #[default]
    Chest,
    Head,
    EyesOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaEnumType {
    CursorCancelPriority,
    TraceContents,
    CollisionGroup,
    ParticleAttachment,
    BaseExplosionTypes,
    HitGroup,
    AILOD,
    NPCSTATE,
    PulseNPCCondition,
    NPCFollowFormation,
    AIStrafing,
    AIVolumetricEventType,
    DamageTypes,
    StanceType,
    SharedMovementGait,
    ChoreoLookAtSpeed,
    ChoreoLookAtMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemaEnumValue {
    CursorCancelPriority(PulseCursorCancelPriority),
    TraceContents(PulseTraceContents),
    CollisionGroup(PulseCollisionGroup),
    ParticleAttachment(ParticleAttachment),
    BaseExplosionTypes(BaseExplosionTypes),
    HitGtoup(HitGroup),
    AILOD(AILOD),
    NPCSTATE(NPCSTATE),
    PulseNPCCondition(PulseNPCCondition),
    NPCFollowFormation(NPCFollowFormation),
    AIStrafing(AIStrafing),
    AIVolumetricEventType(AIVolumetricEventType),
    DamageTypes(DamageTypes),
    StanceType(StanceType),
    SharedMovementGait(SharedMovementGait),
    ChoreoLookAtSpeed(ChoreoLookAtSpeed),
    ChoreoLookAtMode(ChoreoLookAtMode),
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
            "AILOD_t" => Ok(SchemaEnumType::AILOD),
            "NPC_STATE" => Ok(SchemaEnumType::NPCSTATE),
            "PulseNPCCondition_t" => Ok(SchemaEnumType::PulseNPCCondition),
            "NPCFollowFormation_t" => Ok(SchemaEnumType::NPCFollowFormation),
            "AI_Strafing_t" => Ok(SchemaEnumType::AIStrafing),
            "AI_VolumetricEventType_t" => Ok(SchemaEnumType::AIVolumetricEventType),
            "DamageTypes_t" => Ok(SchemaEnumType::DamageTypes),
            "StanceType_t" => Ok(SchemaEnumType::StanceType),
            "SharedMovementGait_t" => Ok(SchemaEnumType::SharedMovementGait),
            "ChoreoLookAtSpeed_t" => Ok(SchemaEnumType::ChoreoLookAtSpeed),
            "ChoreoLookAtMode_t" => Ok(SchemaEnumType::ChoreoLookAtMode),
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
            SchemaEnumType::AILOD => {
                AILOD::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::AILOD(v))
                    .collect()
            }
            SchemaEnumType::NPCSTATE => {
                NPCSTATE::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::NPCSTATE(v))
                    .collect()
            }
            SchemaEnumType::PulseNPCCondition => {
                PulseNPCCondition::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::PulseNPCCondition(v))
                    .collect()
            }
            SchemaEnumType::NPCFollowFormation => {
                NPCFollowFormation::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::NPCFollowFormation(v))
                    .collect()
            }
            SchemaEnumType::AIStrafing => {
                AIStrafing::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::AIStrafing(v))
                    .collect()
            }
            SchemaEnumType::AIVolumetricEventType => {
                AIVolumetricEventType::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::AIVolumetricEventType(v))
                    .collect()
            }
            SchemaEnumType::DamageTypes => {
                DamageTypes::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::DamageTypes(v))
                    .collect()
            }
            SchemaEnumType::StanceType => {
                StanceType::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::StanceType(v))
                    .collect()
            }
            SchemaEnumType::SharedMovementGait => {
                SharedMovementGait::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::SharedMovementGait(v))
                    .collect()
            }
            SchemaEnumType::ChoreoLookAtSpeed => {
                ChoreoLookAtSpeed::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::ChoreoLookAtSpeed(v))
                    .collect()
            }
            SchemaEnumType::ChoreoLookAtMode => {
                ChoreoLookAtMode::VARIANTS
                    .iter()
                    .map(|&v| SchemaEnumValue::ChoreoLookAtMode(v))
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
            SchemaEnumType::AILOD => "AILOD_t",
            SchemaEnumType::NPCSTATE => "NPC_STATE",
            SchemaEnumType::PulseNPCCondition => "PulseNPCCondition_t",
            SchemaEnumType::NPCFollowFormation => "NPCFollowFormation_t",
            SchemaEnumType::AIStrafing => "AI_Strafing_t",
            SchemaEnumType::AIVolumetricEventType => "AI_VolumetricEventType_t",
            SchemaEnumType::DamageTypes => "DamageTypes_t",
            SchemaEnumType::StanceType => "StanceType_t",
            SchemaEnumType::SharedMovementGait => "SharedMovementGait_t",
            SchemaEnumType::ChoreoLookAtSpeed => "ChoreoLookAtSpeed_t",
            SchemaEnumType::ChoreoLookAtMode => "ChoreoLookAtMode_t",
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
            SchemaEnumType::AILOD => "AILOD_t",
            SchemaEnumType::NPCSTATE => "NPC_STATE",
            SchemaEnumType::PulseNPCCondition => "Pulse NPC Conditions",
            SchemaEnumType::NPCFollowFormation => "NPC Follow Formations",
            SchemaEnumType::AIStrafing => "AI Strafing",
            SchemaEnumType::AIVolumetricEventType => "AI Volumetric Event Types",
            SchemaEnumType::DamageTypes => "Damage Types",
            SchemaEnumType::StanceType => "Stance Type",
            SchemaEnumType::SharedMovementGait => "Shared Movement Gait",
            SchemaEnumType::ChoreoLookAtSpeed => "Choreo LookAt Speed",
            SchemaEnumType::ChoreoLookAtMode => "Choreo LookAt Move",
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
            SchemaEnumValue::AILOD(value) => value.to_str_ui(),
            SchemaEnumValue::NPCSTATE(value) => value.to_str_ui(),
            SchemaEnumValue::PulseNPCCondition(value) => value.to_str_ui(),
            SchemaEnumValue::NPCFollowFormation(value) => value.to_str_ui(),
            SchemaEnumValue::AIStrafing(value) => value.to_str_ui(),
            SchemaEnumValue::AIVolumetricEventType(value) => value.to_str_ui(),
            SchemaEnumValue::DamageTypes(value) => value.to_str_ui(),
            SchemaEnumValue::StanceType(value) => value.to_str_ui(),
            SchemaEnumValue::SharedMovementGait(value) => value.to_str_ui(),
            SchemaEnumValue::ChoreoLookAtSpeed(value) => value.to_str_ui(),
            SchemaEnumValue::ChoreoLookAtMode(value) => value.to_str_ui(),
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
            SchemaEnumValue::AILOD(value) => value.to_str(),
            SchemaEnumValue::NPCSTATE(value) => value.to_str(),
            SchemaEnumValue::PulseNPCCondition(value) => value.to_str(),
            SchemaEnumValue::NPCFollowFormation(value) => value.to_str(),
            SchemaEnumValue::AIStrafing(value) => value.to_str(),
            SchemaEnumValue::AIVolumetricEventType(value) => value.to_str(),
            SchemaEnumValue::DamageTypes(value) => value.to_str(),
            SchemaEnumValue::StanceType(value) => value.to_str(),
            SchemaEnumValue::SharedMovementGait(value) => value.to_str(),
            SchemaEnumValue::ChoreoLookAtSpeed(value) => value.to_str(),
            SchemaEnumValue::ChoreoLookAtMode(value) => value.to_str(),
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
            SchemaEnumType::AILOD => 
                SchemaEnumValue::AILOD(AILOD::default()),
            SchemaEnumType::NPCSTATE => 
                SchemaEnumValue::NPCSTATE(NPCSTATE::default()),
            SchemaEnumType::PulseNPCCondition => 
                SchemaEnumValue::PulseNPCCondition(PulseNPCCondition::default()),
            SchemaEnumType::NPCFollowFormation => 
                SchemaEnumValue::NPCFollowFormation(NPCFollowFormation::default()),
            SchemaEnumType::AIStrafing => 
                SchemaEnumValue::AIStrafing(AIStrafing::default()),
            SchemaEnumType::AIVolumetricEventType => 
                SchemaEnumValue::AIVolumetricEventType(AIVolumetricEventType::default()),
            SchemaEnumType::DamageTypes => 
                SchemaEnumValue::DamageTypes(DamageTypes::default()),
            SchemaEnumType::StanceType => 
                SchemaEnumValue::StanceType(StanceType::default()),
            SchemaEnumType::SharedMovementGait => 
                SchemaEnumValue::SharedMovementGait(SharedMovementGait::default()),
            SchemaEnumType::ChoreoLookAtSpeed => 
                SchemaEnumValue::ChoreoLookAtSpeed(ChoreoLookAtSpeed::default()),
            SchemaEnumType::ChoreoLookAtMode => 
                SchemaEnumValue::ChoreoLookAtMode(ChoreoLookAtMode::default()),
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

impl PulseEnumTrait for AILOD {
    fn to_str(self) -> &'static str {
        match self {
            AILOD::High => "eHigh",
            AILOD::Medium => "eMedium",
            AILOD::Low => "eLow",
            AILOD::VeryLow => "eVeryLow",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            AILOD::High => "High",
            AILOD::Medium => "Medium",
            AILOD::Low => "Low",
            AILOD::VeryLow => "Very Low",
        }
    }
}

impl PulseEnumTrait for NPCSTATE {
    fn to_str(self) -> &'static str {
        match self {
            NPCSTATE::Idle => "NPC_STATE_IDLE",
            NPCSTATE::Alert => "NPC_STATE_ALERT",
            NPCSTATE::Combat => "NPC_STATE_COMBAT",
            NPCSTATE::Dead => "NPC_STATE_DEAD",
            NPCSTATE::Inert => "NPC_STATE_INERT",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            NPCSTATE::Idle => "Idle",
            NPCSTATE::Alert => "Alert",
            NPCSTATE::Combat => "Combat",
            NPCSTATE::Dead => "Dead",
            NPCSTATE::Inert => "Inert",
        }
    }
}

impl PulseEnumTrait for PulseNPCCondition {
    fn to_str(self) -> &'static str {
        match self {
            PulseNPCCondition::SeePlayer => "COND_SEE_PLAYER",
            PulseNPCCondition::LostPlayer => "COND_LOST_PLAYER",
            PulseNPCCondition::HearPlayer => "COND_HEAR_PLAYER",
            PulseNPCCondition::PlayerPushing => "COND_PLAYER_PUSHING",
            PulseNPCCondition::NoPrimaryAmmo => "COND_NO_PRIMARY_AMMO",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            PulseNPCCondition::SeePlayer => "Can See the Player",
            PulseNPCCondition::LostPlayer => "Lost Sight of the Player",
            PulseNPCCondition::HearPlayer => "Can Hear the Player",
            PulseNPCCondition::PlayerPushing => "Is Being Pushed by the Player",
            PulseNPCCondition::NoPrimaryAmmo => "No Primary Ammo",
        }
    }
}

impl PulseEnumTrait for NPCFollowFormation {
    fn to_str(self) -> &'static str {
        match self {
            NPCFollowFormation::Default => "Default",
            NPCFollowFormation::CloseCircle => "CloseCircle",
            NPCFollowFormation::WideCircle => "WideCircle",
            NPCFollowFormation::MediumCircle => "MediumCircle",
            NPCFollowFormation::Sidekick => "Sidekick",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            NPCFollowFormation::Default => "Default",
            NPCFollowFormation::CloseCircle => "Close Circle",
            NPCFollowFormation::WideCircle => "Wide Circle",
            NPCFollowFormation::MediumCircle => "Medium Circle",
            NPCFollowFormation::Sidekick => "Sidekick",
        }
    }
}

impl PulseEnumTrait for AIStrafing {
    fn to_str(self) -> &'static str {
        match self {
            AIStrafing::Disabled => "eDisabled",
            AIStrafing::Enabled => "eEnabled",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            AIStrafing::Disabled => "Disabled ( Face Path )",
            AIStrafing::Enabled => "Enabled ( Face Target )",
        }
    }
}

impl PulseEnumTrait for StanceType {
    fn to_str(self) -> &'static str {
        match self {
            StanceType::Default => "STANCE_DEFAULT",
            StanceType::Crouching => "STANCE_CROUCHING",
            StanceType::Prone => "STANCE_PRONE",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            StanceType::Default => "Default",
            StanceType::Crouching => "Crouching",
            StanceType::Prone => "Prone",
        }
    }
}

impl PulseEnumTrait for AIVolumetricEventType {
    fn to_str(self) -> &'static str {
        match self {
            AIVolumetricEventType::Combat => "eCombat",
            AIVolumetricEventType::Player => "ePlayer",
            AIVolumetricEventType::Danger => "eDanger",
            AIVolumetricEventType::BulletImpact => "eBulletImpact",
            AIVolumetricEventType::PhysicsDanger => "ePhysicsDanger",
            AIVolumetricEventType::MoveAway => "eMoveAway",
            AIVolumetricEventType::PlayerVehicle => "ePlayerVehicle",
            AIVolumetricEventType::GlassBreak => "eGlassBreak",
            AIVolumetricEventType::PhysicsObject => "ePhysicsObject",
            AIVolumetricEventType::WarnFriends => "eWarnFriends",
            AIVolumetricEventType::GunFire => "eGunfire",
            AIVolumetricEventType::Explosion => "eExplosion",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            AIVolumetricEventType::Combat => "Combat",
            AIVolumetricEventType::Player => "Player",
            AIVolumetricEventType::Danger => "Danger",
            AIVolumetricEventType::BulletImpact => "Bullet Impact",
            AIVolumetricEventType::PhysicsDanger => "Physics Danger",
            AIVolumetricEventType::MoveAway => "Move Away",
            AIVolumetricEventType::PlayerVehicle => "Player Vehicle",
            AIVolumetricEventType::GlassBreak => "Glass Break",
            AIVolumetricEventType::PhysicsObject => "Physics Object",
            AIVolumetricEventType::WarnFriends => "Warn Friends",
            AIVolumetricEventType::GunFire => "Gunfire",
            AIVolumetricEventType::Explosion => "Explosion",
        }
    }
}

impl PulseEnumTrait for DamageTypes {
    fn to_str(self) -> &'static str {
        match self {
            DamageTypes::Generic => "DMG_GENERIC",
            DamageTypes::Crush => "DMG_CRUSH",
            DamageTypes::Bullet => "DMG_BULLET",
            DamageTypes::Slash => "DMG_SLASH",
            DamageTypes::Burn => "DMG_BURN",
            DamageTypes::Vehicle => "DMG_VEHICLE",
            DamageTypes::Fall => "DMG_FALL",
            DamageTypes::Blast => "DMG_BLAST",
            DamageTypes::Club => "DMG_CLUB",
            DamageTypes::Shock => "DMG_SHOCK",
            DamageTypes::Sonic => "DMG_SONIC",
            DamageTypes::EnergyBeam => "DMG_ENERGYBEAM",
            DamageTypes::Buckshot => "DMG_BUCKSHOT",
            DamageTypes::Drown => "DMG_DROWN",
            DamageTypes::Poison => "DMG_POISON",
            DamageTypes::Radiation => "DMG_RADIATION",
            DamageTypes::DrownRecover => "DMG_DROWNRECOVER",
            DamageTypes::Acid => "DMG_ACID",
            DamageTypes::Physgun => "DMG_PHYSGUN",
            DamageTypes::Dissolve => "DMG_DISSOLVE",
            DamageTypes::BlastSurface => "DMG_BLAST_SURFACE",
            DamageTypes::Headshot => "DMG_HEADSHOT",
            DamageTypes::Crit => "DMG_CRIT",
            DamageTypes::Buffed => "DMG_BUFFED",
            DamageTypes::Dot => "DMG_DOT",
            DamageTypes::GroundAura => "DMG_GROUND_AURA",
            DamageTypes::Lethal => "DMG_LETHAL",
            DamageTypes::Dangerzone => "DMG_DANGERZONE",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            DamageTypes::Generic => "Generic",
            DamageTypes::Crush => "Crush",
            DamageTypes::Bullet => "Bullet",
            DamageTypes::Slash => "Slash",
            DamageTypes::Burn => "Burn",
            DamageTypes::Vehicle => "Vehicle",
            DamageTypes::Fall => "Fall",
            DamageTypes::Blast => "Blast",
            DamageTypes::Club => "Club",
            DamageTypes::Shock => "Shock",
            DamageTypes::Sonic => "Sonic",
            DamageTypes::EnergyBeam => "Energy Beam",
            DamageTypes::Buckshot => "Buckshot",
            DamageTypes::Drown => "Drown",
            DamageTypes::Poison => "Poison",
            DamageTypes::Radiation => "Radiation",
            DamageTypes::DrownRecover => "DrownRecover",
            DamageTypes::Acid => "Acid",
            DamageTypes::Physgun => "Physgun",
            DamageTypes::Dissolve => "Dissolve",
            DamageTypes::BlastSurface => "Blast Surface",
            DamageTypes::Headshot => "Headshot",
            DamageTypes::Crit => "Crit",
            DamageTypes::Buffed => "Buffed",
            DamageTypes::Dot => "Dot",
            DamageTypes::GroundAura => "Ground Aura",
            DamageTypes::Lethal => "Lethal",
            DamageTypes::Dangerzone => "Danger Zone",
        }
    }
}

impl PulseEnumTrait for SharedMovementGait {
    fn to_str(self) -> &'static str {
        match self {
            SharedMovementGait::Slow => "eSlow",
            SharedMovementGait::Medium => "eMedium",
            SharedMovementGait::Fast => "eFast",
            SharedMovementGait::VeryFast => "eVeryFast",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            SharedMovementGait::Slow => "Slow",
            SharedMovementGait::Medium => "Medium",
            SharedMovementGait::Fast => "Fast",
            SharedMovementGait::VeryFast => "Very Fast",
        }
    }
}

impl PulseEnumTrait for ChoreoLookAtSpeed {
    fn to_str(self) -> &'static str {
        match self {
            ChoreoLookAtSpeed::Slow => "eSlow",
            ChoreoLookAtSpeed::Medium => "eMedium",
            ChoreoLookAtSpeed::Fast => "eFast",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            ChoreoLookAtSpeed::Slow => "Slow",
            ChoreoLookAtSpeed::Medium => "Medium",
            ChoreoLookAtSpeed::Fast => "Fast",
        }
    }
}

impl PulseEnumTrait for ChoreoLookAtMode {
    fn to_str(self) -> &'static str {
        match self {
            ChoreoLookAtMode::Chest => "eChest",
            ChoreoLookAtMode::Head => "eHead",
            ChoreoLookAtMode::EyesOnly => "eEyesOnly",
        }
    }
    fn to_str_ui(&self) -> &'static str {
        match self {
            ChoreoLookAtMode::Chest => "Chest",
            ChoreoLookAtMode::Head => "Head",
            ChoreoLookAtMode::EyesOnly => "Eyes Only",
        }
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
