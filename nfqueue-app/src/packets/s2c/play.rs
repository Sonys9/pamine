//! Clientbound play packets (protocol 760 / 1.19.2).
//!
//! Packet ids follow the 1.19.2 table. Fields whose layout depends on a
//! preceding discriminator get a hand-written [`Decode`] impl; everything else
//! uses the derive macro.

use pcore::{
    Decode,
    decoder::{Decoder, MAX_ELEMENTS, NbtTag, Slot, Uuid, VarInt, VarLong},
    errors::Error,
};
use pcore_gen::DecodePacket;

/// Reads a VarInt-counted array, decoding each element with `read`.
///
/// The count is validated against [`MAX_ELEMENTS`] before allocating, so a
/// hostile packet cannot ask for a multi-gigabyte `Vec`.
fn counted<'a, T>(
    decoder: &mut Decoder<'a>,
    read: impl Fn(&mut Decoder<'a>) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    let count = decoder.varint()?.get();
    if count < 0 {
        return Err(Error::Read(0));
    }
    let count = count as usize;
    if count > MAX_ELEMENTS {
        return Err(Error::LenLimit(count));
    }
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push(read(decoder)?);
    }
    Ok(out)
}

/// Reads an "Optional Byte Array": a VarInt length where `0` means absent.
fn optional_bytes<'a>(decoder: &mut Decoder<'a>) -> Result<Option<&'a [u8]>, Error> {
    let bytes = decoder.slice()?;
    Ok(if bytes.is_empty() { None } else { Some(bytes) })
}

/// Reads an "Optional Chat": a `bool` gate followed by a JSON string.
fn optional_chat<'a>(decoder: &mut Decoder<'a>) -> Result<Option<&'a str>, Error> {
    Ok(if decoder.bool()? {
        Some(decoder.string()?)
    } else {
        None
    })
}

/// Reads a VarInt-counted array of Identifiers (VarInt-prefixed strings).
fn identifiers<'a>(decoder: &mut Decoder<'a>) -> Result<Vec<&'a str>, Error> {
    counted(decoder, |d| d.string())
}

#[derive(DecodePacket, Clone, Debug)]
pub enum Play<'a> {
    #[id(0x00)]
    SpawnEntity(SpawnEntity),
    #[id(0x01)]
    SpawnExperienceOrb(SpawnExperienceOrb),
    #[id(0x02)]
    SpawnPlayer(SpawnPlayer),
    #[id(0x03)]
    EntityAnimation(EntityAnimation),
    #[id(0x04)]
    AwardStatistics(AwardStatistics<'a>),
    #[id(0x05)]
    AcknowledgeBlockChange(AcknowledgeBlockChange),
    #[id(0x06)]
    SetBlockDestroyStage(SetBlockDestroyStage),
    #[id(0x07)]
    BlockEntityData(BlockEntityData<'a>),
    #[id(0x08)]
    BlockAction(BlockAction),
    #[id(0x09)]
    BlockUpdate(BlockUpdate),
    #[id(0x0A)]
    BossBar(BossBar<'a>),
    #[id(0x0B)]
    ChangeDifficulty(ChangeDifficulty),
    #[id(0x0C)]
    ChatPreview(ChatPreview<'a>),
    #[id(0x0D)]
    ClearTitles(ClearTitles),
    #[id(0x0E)]
    CommandSuggestionsResponse(CommandSuggestionsResponse<'a>),
    #[id(0x0F)]
    Commands(Commands<'a>),
    #[id(0x10)]
    CloseContainer(CloseContainer),
    #[id(0x11)]
    SetContainerContent(SetContainerContent<'a>),
    #[id(0x12)]
    SetContainerProperty(SetContainerProperty),
    #[id(0x13)]
    SetContainerSlot(SetContainerSlot<'a>),
    #[id(0x14)]
    SetCooldown(SetCooldown),
    #[id(0x15)]
    ChatSuggestions(ChatSuggestions<'a>),
    #[id(0x16)]
    PluginMessage(PluginMessage<'a>),
    #[id(0x17)]
    CustomSoundEffect(CustomSoundEffect<'a>),
    #[id(0x18)]
    HideMessage(HideMessage<'a>),
    #[id(0x19)]
    Disconnect(Disconnect<'a>),
    #[id(0x1A)]
    EntityEvent(EntityEvent),
    #[id(0x1B)]
    Explosion(Explosion),
    #[id(0x1C)]
    UnloadChunk(UnloadChunk),
    #[id(0x1D)]
    GameEvent(GameEvent),
    #[id(0x1E)]
    OpenHorseScreen(OpenHorseScreen),
    #[id(0x1F)]
    InitializeWorldBorder(InitializeWorldBorder),
    #[id(0x20)]
    KeepAlive(KeepAlive),
    #[id(0x21)]
    ChunkDataAndUpdateLight(ChunkDataAndUpdateLight<'a>),
    #[id(0x22)]
    WorldEvent(WorldEvent),
    #[id(0x23)]
    Particle(Particle<'a>),
    #[id(0x24)]
    UpdateLight(UpdateLight<'a>),
    #[id(0x25)]
    Login(Login<'a>),
    #[id(0x26)]
    MapData(MapData<'a>),
    #[id(0x27)]
    MerchantOffers(MerchantOffers<'a>),
    #[id(0x28)]
    UpdateEntityPosition(UpdateEntityPosition),
    #[id(0x29)]
    UpdateEntityPositionAndRotation(UpdateEntityPositionAndRotation),
    #[id(0x2A)]
    UpdateEntityRotation(UpdateEntityRotation),
    #[id(0x2B)]
    MoveVehicle(MoveVehicle),
    #[id(0x2C)]
    OpenBook(OpenBook),
    #[id(0x2D)]
    OpenScreen(OpenScreen<'a>),
    #[id(0x2E)]
    OpenSignEditor(OpenSignEditor),
    #[id(0x2F)]
    Ping(Ping),
    #[id(0x30)]
    PlaceGhostRecipe(PlaceGhostRecipe<'a>),
    #[id(0x31)]
    PlayerAbilities(PlayerAbilities),
    #[id(0x32)]
    MessageHeader(MessageHeader<'a>),
    #[id(0x33)]
    PlayerChatMessage(PlayerChatMessage<'a>),
    #[id(0x34)]
    EndCombat(EndCombat),
    #[id(0x35)]
    EnterCombat(EnterCombat),
    #[id(0x36)]
    CombatDeath(CombatDeath<'a>),
    #[id(0x37)]
    PlayerInfo(PlayerInfo<'a>),
    #[id(0x38)]
    LookAt(LookAt),
    #[id(0x39)]
    SynchronizePlayerPosition(SynchronizePlayerPosition),
    #[id(0x3A)]
    UpdateRecipeBook(UpdateRecipeBook<'a>),
    #[id(0x3B)]
    RemoveEntities(RemoveEntities),
    #[id(0x3C)]
    RemoveEntityEffect(RemoveEntityEffect),
    #[id(0x3D)]
    ResourcePack(ResourcePack<'a>),
    #[id(0x3E)]
    Respawn(Respawn<'a>),
    #[id(0x3F)]
    SetHeadRotation(SetHeadRotation),
    #[id(0x40)]
    UpdateSectionBlocks(UpdateSectionBlocks),
    #[id(0x41)]
    SelectAdvancementsTab(SelectAdvancementsTab<'a>),
    #[id(0x42)]
    ServerData(ServerData<'a>),
    #[id(0x43)]
    SetActionBarText(SetActionBarText<'a>),
    #[id(0x44)]
    SetBorderCenter(SetBorderCenter),
    #[id(0x45)]
    SetBorderLerpSize(SetBorderLerpSize),
    #[id(0x46)]
    SetBorderSize(SetBorderSize),
    #[id(0x47)]
    SetBorderWarningDelay(SetBorderWarningDelay),
    #[id(0x48)]
    SetBorderWarningDistance(SetBorderWarningDistance),
    #[id(0x49)]
    SetCamera(SetCamera),
    #[id(0x4A)]
    SetHeldItem(SetHeldItem),
    #[id(0x4B)]
    SetCenterChunk(SetCenterChunk),
    #[id(0x4C)]
    SetRenderDistance(SetRenderDistance),
    #[id(0x4D)]
    SetDefaultSpawnPosition(SetDefaultSpawnPosition),
    #[id(0x4E)]
    SetDisplayChatPreview(SetDisplayChatPreview),
    #[id(0x4F)]
    DisplayObjective(DisplayObjective<'a>),
    #[id(0x50)]
    SetEntityMetadata(SetEntityMetadata<'a>),
    #[id(0x51)]
    LinkEntities(LinkEntities),
    #[id(0x52)]
    SetEntityVelocity(SetEntityVelocity),
    #[id(0x53)]
    SetEquipment(SetEquipment<'a>),
    #[id(0x54)]
    SetExperience(SetExperience),
    #[id(0x55)]
    SetHealth(SetHealth),
    #[id(0x56)]
    UpdateObjectives(UpdateObjectives<'a>),
    #[id(0x57)]
    SetPassengers(SetPassengers),
    #[id(0x58)]
    UpdateTeams(UpdateTeams<'a>),
    #[id(0x59)]
    UpdateScore(UpdateScore<'a>),
    #[id(0x5A)]
    SetSimulationDistance(SetSimulationDistance),
    #[id(0x5B)]
    SetSubtitleText(SetSubtitleText<'a>),
    #[id(0x5C)]
    UpdateTime(UpdateTime),
    #[id(0x5D)]
    SetTitleText(SetTitleText<'a>),
    #[id(0x5E)]
    SetTitleAnimationTimes(SetTitleAnimationTimes),
    #[id(0x5F)]
    EntitySoundEffect(EntitySoundEffect),
    #[id(0x60)]
    SoundEffect(SoundEffect),
    #[id(0x61)]
    StopSound(StopSound<'a>),
    #[id(0x62)]
    SystemChatMessage(SystemChatMessage<'a>),
    #[id(0x63)]
    SetTabListHeaderAndFooter(SetTabListHeaderAndFooter<'a>),
    #[id(0x64)]
    TagQueryResponse(TagQueryResponse<'a>),
    #[id(0x65)]
    PickupItem(PickupItem),
    #[id(0x66)]
    TeleportEntity(TeleportEntity),
    #[id(0x67)]
    UpdateAdvancements(UpdateAdvancements<'a>),
    #[id(0x68)]
    UpdateAttributes(UpdateAttributes<'a>),
    #[id(0x69)]
    EntityEffect(EntityEffect<'a>),
    #[id(0x6A)]
    UpdateRecipes(UpdateRecipes<'a>),
    #[id(0x6B)]
    UpdateTags(UpdateTags<'a>),
}

#[derive(Decode, Debug, Clone)]
pub struct SpawnEntity {
    pub entity_id: VarInt,
    pub entity_uuid: Uuid,
    pub kind: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pitch: u8,
    pub yaw: u8,
    pub head_yaw: u8,
    pub data: VarInt,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

#[derive(Decode, Debug, Clone)]
pub struct SpawnExperienceOrb {
    pub entity_id: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub count: i16,
}

#[derive(Decode, Debug, Clone)]
pub struct SpawnPlayer {
    pub entity_id: VarInt,
    pub player_uuid: Uuid,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: u8,
    pub pitch: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct EntityAnimation {
    pub entity_id: VarInt,
    pub animation: u8,
}

/// `statistics` is `count` entries of `(category, statistic, value)`, not a
/// byte array — the count is an element count, so the derive macro cannot
/// express it.
#[derive(Debug, Clone)]
pub struct AwardStatistics<'a> {
    pub statistics: Vec<Statistic<'a>>,
}

#[derive(Debug, Clone)]
pub struct Statistic<'a> {
    pub category_id: VarInt,
    pub statistic_id: VarInt,
    pub value: VarInt,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Decode<'a> for AwardStatistics<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Nbt("negative statistic count"));
        }
        let mut statistics = Vec::with_capacity(count as usize);
        for _ in 0..count {
            statistics.push(Statistic {
                category_id: decoder.varint()?,
                statistic_id: decoder.varint()?,
                value: decoder.varint()?,
                _marker: std::marker::PhantomData,
            });
        }
        Ok(Self { statistics })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct AcknowledgeBlockChange {
    pub sequence_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBlockDestroyStage {
    pub entity_id: VarInt,
    pub location: u64,
    pub destroy_stage: i8,
}

#[derive(Decode, Debug, Clone)]
pub struct BlockEntityData<'a> {
    pub location: u64,
    pub kind: VarInt,
    pub nbt: NbtTag<'a>,
}

#[derive(Decode, Debug, Clone)]
pub struct BlockAction {
    pub location: u64,
    pub action_id: u8,
    pub action_parameter: u8,
    pub block_type: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct BlockUpdate {
    pub location: u64,
    pub block_id: VarInt,
}

/// The `action`-dependent body of [`BossBar`].
#[derive(Debug, Clone)]
pub enum BossBarAction<'a> {
    /// `0` — a new bar appears.
    Add {
        title: &'a str,
        health: f32,
        color: VarInt,
        division: VarInt,
        flags: u8,
    },
    /// `1` — the bar disappears; nothing follows.
    Remove,
    /// `2` — only the health changed.
    UpdateHealth { health: f32 },
    /// `3` — only the title changed.
    UpdateTitle { title: &'a str },
    /// `4` — only the colour and division changed.
    UpdateStyle { color: VarInt, division: VarInt },
    /// `5` — only the flags changed.
    UpdateFlags { flags: u8 },
}

#[derive(Debug, Clone)]
pub struct BossBar<'a> {
    pub uuid: Uuid,
    pub action: BossBarAction<'a>,
}

impl<'a> Decode<'a> for BossBar<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let uuid = decoder.uuid()?;
        let action = match decoder.varint()?.get() {
            0 => BossBarAction::Add {
                title: decoder.string()?,
                health: decoder.f32()?,
                color: decoder.varint()?,
                division: decoder.varint()?,
                flags: decoder.u8()?,
            },
            1 => BossBarAction::Remove,
            2 => BossBarAction::UpdateHealth {
                health: decoder.f32()?,
            },
            3 => BossBarAction::UpdateTitle {
                title: decoder.string()?,
            },
            4 => BossBarAction::UpdateStyle {
                color: decoder.varint()?,
                division: decoder.varint()?,
            },
            5 => BossBarAction::UpdateFlags {
                flags: decoder.u8()?,
            },
            other => return Err(Error::UnknownOpcode(other)),
        };
        Ok(Self { uuid, action })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct ChangeDifficulty {
    pub difficulty: u8,
    pub locked: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct ChatPreview<'a> {
    pub query_id: i32,
    pub has_component: bool,
    pub message: Option<&'a str>,
}

#[derive(Decode, Debug, Clone)]
pub struct ClearTitles {
    pub reset: bool,
}

/// One entry of a tab-completion response.
#[derive(Debug, Clone)]
pub struct SuggestionMatch<'a> {
    pub text: &'a str,
    /// The tooltip is a `bool`-gated chat component.
    pub tooltip: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct CommandSuggestionsResponse<'a> {
    pub id: VarInt,
    pub start: VarInt,
    pub length: VarInt,
    pub matches: Vec<SuggestionMatch<'a>>,
}

impl<'a> Decode<'a> for CommandSuggestionsResponse<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let id = decoder.varint()?;
        let start = decoder.varint()?;
        let length = decoder.varint()?;
        let matches = counted(decoder, |d| {
            Ok(SuggestionMatch {
                text: d.string()?,
                tooltip: optional_chat(d)?,
            })
        })?;
        Ok(Self {
            id,
            start,
            length,
            matches,
        })
    }
}

/// The command graph. `nodes` is a VarInt **count** of variable-length node
/// structures, so it has to be parsed element by element.
#[derive(Debug, Clone)]
pub struct Commands<'a> {
    pub nodes: Vec<CommandNode<'a>>,
    pub root_index: VarInt,
}

/// One node of the command graph.
///
/// `flags` bit 0x03 selects the node type: 0 = root, 1 = literal, 2 = argument.
/// Bit 0x04 marks an executable node, 0x08 a redirect and 0x10 suggestions.
#[derive(Debug, Clone)]
pub struct CommandNode<'a> {
    pub flags: u8,
    pub children: Vec<VarInt>,
    pub redirect: Option<VarInt>,
    pub name: Option<&'a str>,
    pub parser_id: Option<VarInt>,
    pub suggestions: Option<&'a str>,
}

impl<'a> Decode<'a> for Commands<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Nbt("negative command node count"));
        }
        let mut nodes = Vec::with_capacity(count as usize);
        for _ in 0..count {
            nodes.push(CommandNode::decode(decoder)?);
        }
        let root_index = decoder.varint()?;
        Ok(Self { nodes, root_index })
    }
}

impl<'a> Decode<'a> for CommandNode<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        const TYPE_MASK: u8 = 0x03;
        const REDIRECT: u8 = 0x08;
        const SUGGESTIONS: u8 = 0x10;

        let flags = decoder.u8()?;

        let child_count = decoder.varint()?.get();
        if child_count < 0 {
            return Err(Error::Nbt("negative child count"));
        }
        let mut children = Vec::with_capacity(child_count as usize);
        for _ in 0..child_count {
            children.push(decoder.varint()?);
        }

        let redirect = if flags & REDIRECT != 0 {
            Some(decoder.varint()?)
        } else {
            None
        };

        // Root nodes (type 0) carry no name. Literal (1) and argument (2) nodes
        // both start with a name; argument nodes add a parser id and then
        // parser-specific properties, which have no length prefix and are
        // therefore not consumed here.
        let (name, parser_id) = match flags & TYPE_MASK {
            0 => (None, None),
            1 => (Some(decoder.string()?), None),
            _ => (Some(decoder.string()?), Some(decoder.varint()?)),
        };

        let suggestions = if flags & SUGGESTIONS != 0 {
            Some(decoder.string()?)
        } else {
            None
        };

        Ok(Self {
            flags,
            children,
            redirect,
            name,
            parser_id,
            suggestions,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct CloseContainer {
    pub window_id: u8,
}

#[derive(Debug, Clone)]
pub struct SetContainerContent<'a> {
    pub window_id: u8,
    pub state_id: VarInt,
    /// Every slot of the window, in order.
    pub slots: Vec<Slot<'a>>,
    /// The item held by the cursor.
    pub carried_item: Slot<'a>,
}

impl<'a> Decode<'a> for SetContainerContent<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let window_id = decoder.u8()?;
        let state_id = decoder.varint()?;

        // `count` is a VarInt, and each of the `count` slots is a full Slot.
        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Read(0));
        }
        let mut slots = Vec::with_capacity(count as usize);
        for _ in 0..count {
            slots.push(Slot::decode(decoder)?);
        }
        let carried_item = Slot::decode(decoder)?;

        Ok(Self {
            window_id,
            state_id,
            slots,
            carried_item,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct SetContainerProperty {
    pub window_id: u8,
    pub property: i16,
    pub value: i16,
}

#[derive(Decode, Debug, Clone)]
pub struct SetContainerSlot<'a> {
    pub window_id: i8,
    pub state_id: VarInt,
    pub slot: i16,
    /// A single Slot: `bool present + VarInt id + i8 count + NBT`.
    pub slot_data: Slot<'a>,
}

#[derive(Decode, Debug, Clone)]
pub struct SetCooldown {
    pub item_id: VarInt,
    pub cooldown_ticks: VarInt,
}

#[derive(Debug, Clone)]
pub struct ChatSuggestions<'a> {
    pub action: VarInt,
    pub entries: Vec<&'a str>,
}

impl<'a> Decode<'a> for ChatSuggestions<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let action = decoder.varint()?;
        let entries = counted(decoder, |d| d.string())?;
        Ok(Self { action, entries })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct PluginMessage<'a> {
    pub channel: &'a str,
    /// Everything after the channel: the payload has no length prefix, its
    /// length is whatever remains in the packet. This is genuinely opaque —
    /// the layout is defined by the plugin, not by the protocol.
    #[decode(rest)]
    pub data: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct CustomSoundEffect<'a> {
    pub sound_name: &'a str,
    pub sound_category: VarInt,
    pub effect_position_x: i32,
    pub effect_position_y: i32,
    pub effect_position_z: i32,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct HideMessage<'a> {
    pub signature: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct Disconnect<'a> {
    pub reason: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct EntityEvent {
    pub entity_id: i32,
    pub entity_status: i8,
}

/// One destroyed block, as a signed offset from the explosion centre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExplosionRecord {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

#[derive(Debug, Clone)]
pub struct Explosion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub strength: f32,
    pub records: Vec<ExplosionRecord>,
    /// The knockback applied to the player, one float per axis.
    pub player_motion: [f32; 3],
}

impl<'a> Decode<'a> for Explosion {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let x = decoder.f32()?;
        let y = decoder.f32()?;
        let z = decoder.f32()?;
        let strength = decoder.f32()?;
        let records = counted(decoder, |d| {
            Ok(ExplosionRecord {
                x: d.i8()?,
                y: d.i8()?,
                z: d.i8()?,
            })
        })?;
        let player_motion = [decoder.f32()?, decoder.f32()?, decoder.f32()?];
        Ok(Self {
            x,
            y,
            z,
            strength,
            records,
            player_motion,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct UnloadChunk {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct GameEvent {
    pub event: u8,
    pub value: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct OpenHorseScreen {
    pub window_id: u8,
    pub slot_count: VarInt,
    pub entity_id: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct InitializeWorldBorder {
    pub x: f64,
    pub z: f64,
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: i64,
    pub portal_teleport_boundary: VarInt,
    pub warning_blocks: VarInt,
    pub warning_time: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct KeepAlive {
    pub keep_alive_id: i64,
}

/// Chunk data plus light updates.
///
/// Hand-written `Decode`: the wire layout has three parts the derive macro
/// cannot express.
///
/// - `data` is a byte array whose length is the `Size` field, i.e. the VarInt
///   prefix *is* `Size` — there is no separate field to read.
/// - `block_entities` is a VarInt **count** followed by that many structures,
///   not a byte length.
/// - the light section is four BitSets followed by two length-prefixed arrays
///   of 2048-byte arrays.
#[derive(Debug, Clone)]
pub struct ChunkDataAndUpdateLight<'a> {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub heightmaps: NbtTag<'a>,
    pub data: &'a [u8],
    pub block_entities: Vec<BlockEntity<'a>>,
    pub trust_edges: bool,
    pub light: LightData<'a>,
}

impl<'a> Decode<'a> for ChunkDataAndUpdateLight<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let chunk_x = decoder.i32()?;
        let chunk_z = decoder.i32()?;
        let heightmaps = decoder.nbt()?;
        // `slice()` consumes the `Size` VarInt as its own length prefix.
        let data = decoder.slice()?;

        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Nbt("negative block entity count"));
        }
        let mut block_entities = Vec::with_capacity(count as usize);
        for _ in 0..count {
            block_entities.push(BlockEntity::decode(decoder)?);
        }

        let trust_edges = decoder.bool()?;
        let light = LightData::decode(decoder)?;

        Ok(Self {
            chunk_x,
            chunk_z,
            heightmaps,
            data,
            block_entities,
            trust_edges,
            light,
        })
    }
}

/// One block entity inside a chunk.
#[derive(Debug, Clone)]
pub struct BlockEntity<'a> {
    /// `(blockX & 15) << 4 | (blockZ & 15)`
    pub packed_xz: u8,
    pub y: i16,
    pub kind: VarInt,
    pub data: NbtTag<'a>,
}

impl<'a> Decode<'a> for BlockEntity<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        Ok(Self {
            packed_xz: decoder.u8()?,
            y: decoder.i16()?,
            kind: decoder.varint()?,
            data: decoder.nbt()?,
        })
    }
}

/// Light section shared by `Chunk Data` and `Update Light`.
#[derive(Debug, Clone)]
pub struct LightData<'a> {
    pub sky_light_mask: &'a [u8],
    pub block_light_mask: &'a [u8],
    pub empty_sky_light_mask: &'a [u8],
    pub empty_block_light_mask: &'a [u8],
    pub sky_light: Vec<&'a [u8]>,
    pub block_light: Vec<&'a [u8]>,
}

impl<'a> Decode<'a> for LightData<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let sky_light_mask = bitset(decoder)?;
        let block_light_mask = bitset(decoder)?;
        let empty_sky_light_mask = bitset(decoder)?;
        let empty_block_light_mask = bitset(decoder)?;
        let sky_light = light_arrays(decoder)?;
        let block_light = light_arrays(decoder)?;
        Ok(Self {
            sky_light_mask,
            block_light_mask,
            empty_sky_light_mask,
            empty_block_light_mask,
            sky_light,
            block_light,
        })
    }
}

/// A BitSet is a VarInt count of `i64` words, then that many words.
fn bitset<'a>(decoder: &mut Decoder<'a>) -> Result<&'a [u8], Error> {
    let count = decoder.varint()?.get();
    if count < 0 {
        return Err(Error::Nbt("negative bitset length"));
    }
    decoder.read_len(count as usize * 8)
}

/// A length-prefixed array of length-prefixed 2048-byte arrays.
fn light_arrays<'a>(decoder: &mut Decoder<'a>) -> Result<Vec<&'a [u8]>, Error> {
    let count = decoder.varint()?.get();
    if count < 0 {
        return Err(Error::Nbt("negative light array count"));
    }
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        out.push(decoder.slice()?);
    }
    Ok(out)
}

#[derive(Decode, Debug, Clone)]
pub struct WorldEvent {
    pub event: i32,
    pub location: u64,
    pub data: i32,
    pub disable_relative_volume: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct Particle<'a> {
    pub particle_id: VarInt,
    pub long_distance: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    pub max_speed: f32,
    pub particle_count: i32,
    pub data: &'a [u8],
}

/// Light-only update for a chunk. Shares the light section with
/// [`ChunkDataAndUpdateLight`], so it needs the same hand-written `Decode`.
#[derive(Debug, Clone)]
pub struct UpdateLight<'a> {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
    pub trust_edges: bool,
    pub light: LightData<'a>,
}

impl<'a> Decode<'a> for UpdateLight<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        Ok(Self {
            chunk_x: decoder.varint()?,
            chunk_z: decoder.varint()?,
            trust_edges: decoder.bool()?,
            light: LightData::decode(decoder)?,
        })
    }
}

/// Join-game packet. Hand-written `Decode` because `dimension_names` is a
/// VarInt **count** of Identifiers, not a byte array.
#[derive(Debug, Clone)]
pub struct Login<'a> {
    pub entity_id: i32,
    pub is_hardcore: bool,
    pub gamemode: u8,
    pub previous_gamemode: i8,
    pub dimension_names: Vec<&'a str>,
    pub registry_codec: NbtTag<'a>,
    pub dimension_type: &'a str,
    pub dimension_name: &'a str,
    pub hashed_seed: i64,
    pub max_players: VarInt,
    pub view_distance: VarInt,
    pub simulation_distance: VarInt,
    pub reduced_debug_info: bool,
    pub enable_respawn_screen: bool,
    pub is_debug: bool,
    pub is_flat: bool,
    pub death_dimension_name: Option<&'a str>,
    pub death_location: Option<u64>,
}

impl<'a> Decode<'a> for Login<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_id = decoder.i32()?;
        let is_hardcore = decoder.bool()?;
        let gamemode = decoder.u8()?;
        let previous_gamemode = decoder.i8()?;

        let dimension_count = decoder.varint()?.get();
        if dimension_count < 0 {
            return Err(Error::Nbt("negative dimension count"));
        }
        let mut dimension_names = Vec::with_capacity(dimension_count as usize);
        for _ in 0..dimension_count {
            dimension_names.push(decoder.string()?);
        }

        let registry_codec = decoder.nbt()?;
        let dimension_type = decoder.string()?;
        let dimension_name = decoder.string()?;
        let hashed_seed = decoder.i64()?;
        let max_players = decoder.varint()?;
        let view_distance = decoder.varint()?;
        let simulation_distance = decoder.varint()?;
        let reduced_debug_info = decoder.bool()?;
        let enable_respawn_screen = decoder.bool()?;
        let is_debug = decoder.bool()?;
        let is_flat = decoder.bool()?;

        let has_death_location = decoder.bool()?;
        let (death_dimension_name, death_location) = if has_death_location {
            (Some(decoder.string()?), Some(decoder.u64()?))
        } else {
            (None, None)
        };

        Ok(Self {
            entity_id,
            is_hardcore,
            gamemode,
            previous_gamemode,
            dimension_names,
            registry_codec,
            dimension_type,
            dimension_name,
            hashed_seed,
            max_players,
            view_distance,
            simulation_distance,
            reduced_debug_info,
            enable_respawn_screen,
            is_debug,
            is_flat,
            death_dimension_name,
            death_location,
        })
    }
}

/// One marker drawn on a map.
#[derive(Debug, Clone)]
pub struct MapIcon<'a> {
    /// `type << 4 | direction`, packed into a single byte.
    pub kind_and_direction: u8,
    pub x: i8,
    pub z: i8,
    /// A `bool`-gated display name.
    pub display_name: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct MapData<'a> {
    pub map_id: VarInt,
    pub scale: i8,
    pub locked: bool,
    pub icons: Vec<MapIcon<'a>>,
}

impl<'a> Decode<'a> for MapData<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let map_id = decoder.varint()?;
        let scale = decoder.i8()?;
        let locked = decoder.bool()?;
        // The icon array is only on the wire when tracking is enabled; the
        // count is a VarInt and `0` covers the "not tracking" case.
        let icons = counted(decoder, |d| {
            Ok(MapIcon {
                kind_and_direction: d.u8()?,
                x: d.i8()?,
                z: d.i8()?,
                display_name: optional_chat(d)?,
            })
        })?;
        Ok(Self {
            map_id,
            scale,
            locked,
            icons,
        })
    }
}

/// One trade offered by a villager.
#[derive(Debug, Clone)]
pub struct TradeOffer<'a> {
    pub input_item_1: Slot<'a>,
    pub output_item: Slot<'a>,
    pub input_item_2: Slot<'a>,
    pub trade_disabled: bool,
    pub number_of_trade_uses: i32,
    pub maximum_number_of_trade_uses: i32,
    pub xp: i32,
    pub special_price: i32,
    pub price_multiplier: f32,
    pub demand: i32,
}

#[derive(Debug, Clone)]
pub struct MerchantOffers<'a> {
    pub window_id: VarInt,
    pub trades: Vec<TradeOffer<'a>>,
}

impl<'a> Decode<'a> for MerchantOffers<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let window_id = decoder.varint()?;
        let trades = counted(decoder, |d| {
            Ok(TradeOffer {
                input_item_1: Slot::decode(d)?,
                output_item: Slot::decode(d)?,
                input_item_2: Slot::decode(d)?,
                trade_disabled: d.bool()?,
                number_of_trade_uses: d.i32()?,
                maximum_number_of_trade_uses: d.i32()?,
                xp: d.i32()?,
                special_price: d.i32()?,
                price_multiplier: d.f32()?,
                demand: d.i32()?,
            })
        })?;
        Ok(Self { window_id, trades })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateEntityPosition {
    pub entity_id: VarInt,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub on_ground: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateEntityPositionAndRotation {
    pub entity_id: VarInt,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateEntityRotation {
    pub entity_id: VarInt,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct MoveVehicle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct OpenBook {
    pub hand: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct OpenScreen<'a> {
    pub window_id: VarInt,
    pub window_type: VarInt,
    pub window_title: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct OpenSignEditor {
    pub location: u64,
}

#[derive(Decode, Debug, Clone)]
pub struct Ping {
    pub id: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct PlaceGhostRecipe<'a> {
    pub window_id: u8,
    pub recipe: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct PlayerAbilities {
    pub flags: u8,
    pub flying_speed: f32,
    pub field_of_view_modifier: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct MessageHeader<'a> {
    pub has_preceding_signature: bool,
    pub preceding_signature: Option<&'a [u8]>,
    pub sender: Uuid,
}

/// A previously sent message, referenced by its id and signature.
#[derive(Debug, Clone)]
pub struct PreviousMessage<'a> {
    pub message_id: VarInt,
    pub signature: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct PlayerChatMessage<'a> {
    pub sender: Uuid,
    pub index: VarInt,
    /// `None` when the signature length prefix is `0`.
    pub signature: Option<&'a [u8]>,
    pub message: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    pub previous_messages: Vec<PreviousMessage<'a>>,
    pub unsigned_content: Option<&'a str>,
    pub filter_type: VarInt,
    pub network_target: VarInt,
    pub network_name: &'a str,
}

impl<'a> Decode<'a> for PlayerChatMessage<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let sender = decoder.uuid()?;
        let index = decoder.varint()?;
        let signature = optional_bytes(decoder)?;
        let message = decoder.string()?;
        let timestamp = decoder.i64()?;
        let salt = decoder.i64()?;
        let previous_messages = counted(decoder, |d| {
            Ok(PreviousMessage {
                message_id: d.varint()?,
                signature: d.slice()?,
            })
        })?;
        let unsigned_content = optional_chat(decoder)?;
        let filter_type = decoder.varint()?;
        let network_target = decoder.varint()?;
        let network_name = decoder.string()?;
        Ok(Self {
            sender,
            index,
            signature,
            message,
            timestamp,
            salt,
            previous_messages,
            unsigned_content,
            filter_type,
            network_target,
            network_name,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct EndCombat {
    pub duration: VarInt,
    pub entity_id: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct EnterCombat {}

#[derive(Decode, Debug, Clone)]
pub struct CombatDeath<'a> {
    pub player_id: VarInt,
    pub entity_id: i32,
    pub message: &'a str,
}

/// Bit flags of the `Player Info` action mask.
pub const PLAYER_INFO_ADD_PLAYER: u8 = 0x01;
pub const PLAYER_INFO_INITIALIZE_CHAT: u8 = 0x02;
pub const PLAYER_INFO_UPDATE_GAME_MODE: u8 = 0x04;
pub const PLAYER_INFO_UPDATE_LISTED: u8 = 0x08;
pub const PLAYER_INFO_UPDATE_LATENCY: u8 = 0x10;
pub const PLAYER_INFO_UPDATE_DISPLAY_NAME: u8 = 0x20;

/// A signed skin/cape property.
#[derive(Debug, Clone)]
pub struct PlayerProperty<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub signature: Option<&'a str>,
}

/// One player entry of `Player Info`. Every field past `uuid` is gated by the
/// packet's `actions` bitmask, so absent fields stay `None`.
#[derive(Debug, Clone)]
pub struct PlayerInfoEntry<'a> {
    pub uuid: Uuid,
    pub name: Option<&'a str>,
    pub properties: Option<Vec<PlayerProperty<'a>>>,
    pub game_mode: Option<VarInt>,
    pub listed: Option<bool>,
    pub latency: Option<VarInt>,
    pub display_name: Option<Option<&'a str>>,
}

#[derive(Debug, Clone)]
pub struct PlayerInfo<'a> {
    pub actions: u8,
    pub players: Vec<PlayerInfoEntry<'a>>,
}

impl<'a> Decode<'a> for PlayerInfo<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let actions = decoder.u8()?;
        let players = counted(decoder, |d| {
            let uuid = d.uuid()?;

            let (name, properties) = if actions & PLAYER_INFO_ADD_PLAYER != 0 {
                let name = d.string()?;
                let properties = counted(d, |d| {
                    Ok(PlayerProperty {
                        name: d.string()?,
                        value: d.string()?,
                        signature: if d.bool()? { Some(d.string()?) } else { None },
                    })
                })?;
                (Some(name), Some(properties))
            } else {
                (None, None)
            };

            // The chat-initialisation block (session id + key signature) is
            // skipped: it is opaque key material this proxy never inspects.
            if actions & PLAYER_INFO_INITIALIZE_CHAT != 0 {
                let _has_signature = d.bool()?;
                let _session_id = d.uuid()?;
                let _key_signature = d.slice()?;
            }

            let game_mode = if actions & PLAYER_INFO_UPDATE_GAME_MODE != 0 {
                Some(d.varint()?)
            } else {
                None
            };
            let listed = if actions & PLAYER_INFO_UPDATE_LISTED != 0 {
                Some(d.bool()?)
            } else {
                None
            };
            let latency = if actions & PLAYER_INFO_UPDATE_LATENCY != 0 {
                Some(d.varint()?)
            } else {
                None
            };
            let display_name = if actions & PLAYER_INFO_UPDATE_DISPLAY_NAME != 0 {
                Some(optional_chat(d)?)
            } else {
                None
            };

            Ok(PlayerInfoEntry {
                uuid,
                name,
                properties,
                game_mode,
                listed,
                latency,
                display_name,
            })
        })?;
        Ok(Self { actions, players })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct LookAt {
    pub feet_eyes: VarInt,
    pub target_x: f64,
    pub target_y: f64,
    pub target_z: f64,
    pub is_entity: bool,
    pub entity_id: Option<VarInt>,
    pub entity_feet_eyes: Option<VarInt>,
}

#[derive(Decode, Debug, Clone)]
pub struct SynchronizePlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
    pub teleport_id: VarInt,
    pub dismount_vehicle: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateRecipeBook<'a> {
    pub action: VarInt,
    pub crafting_book_open: bool,
    pub crafting_filter_active: bool,
    pub smelting_book_open: bool,
    pub smelting_filter_active: bool,
    pub blast_furnace_open: bool,
    pub blast_furnace_filter_active: bool,
    pub smoker_open: bool,
    pub smoker_filter_active: bool,
    /// Recipes being added or removed.
    pub recipe_ids: Vec<&'a str>,
    /// Only present for `action == 0` (init), where it lists what is already
    /// known.
    pub already_known: Option<Vec<&'a str>>,
}

impl<'a> Decode<'a> for UpdateRecipeBook<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let action = decoder.varint()?;
        let crafting_book_open = decoder.bool()?;
        let crafting_filter_active = decoder.bool()?;
        let smelting_book_open = decoder.bool()?;
        let smelting_filter_active = decoder.bool()?;
        let blast_furnace_open = decoder.bool()?;
        let blast_furnace_filter_active = decoder.bool()?;
        let smoker_open = decoder.bool()?;
        let smoker_filter_active = decoder.bool()?;
        let recipe_ids = identifiers(decoder)?;
        // Only the "init" action carries the second array.
        let already_known = if action.get() == 0 {
            Some(identifiers(decoder)?)
        } else {
            None
        };
        Ok(Self {
            action,
            crafting_book_open,
            crafting_filter_active,
            smelting_book_open,
            smelting_filter_active,
            blast_furnace_open,
            blast_furnace_filter_active,
            smoker_open,
            smoker_filter_active,
            recipe_ids,
            already_known,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RemoveEntities {
    pub entity_ids: Vec<VarInt>,
}

impl<'a> Decode<'a> for RemoveEntities {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_ids = counted(decoder, |d| d.varint())?;
        Ok(Self { entity_ids })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct RemoveEntityEffect {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct ResourcePack<'a> {
    pub url: &'a str,
    pub hash: &'a str,
    pub forced: bool,
    pub has_prompt_message: bool,
    pub prompt_message: Option<&'a str>,
}

#[derive(Decode, Debug, Clone)]
pub struct Respawn<'a> {
    pub dimension_type: &'a str,
    pub dimension_name: &'a str,
    pub hashed_seed: i64,
    pub gamemode: u8,
    pub previous_gamemode: i8,
    pub is_debug: bool,
    pub is_flat: bool,
    pub copy_metadata: bool,
    pub has_death_location: bool,
    pub death_dimension_name: Option<&'a str>,
    pub death_location: Option<u64>,
}

#[derive(Decode, Debug, Clone)]
pub struct SetHeadRotation {
    pub entity_id: VarInt,
    pub head_yaw: u8,
}

/// One changed block inside a section: a packed `position` VarLong and the new
/// block state id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionBlock {
    pub position: VarLong,
    pub block_state: VarInt,
}

#[derive(Debug, Clone)]
pub struct UpdateSectionBlocks {
    pub chunk_section_position: i64,
    pub suppress_light_updates: bool,
    pub blocks: Vec<SectionBlock>,
}

impl<'a> Decode<'a> for UpdateSectionBlocks {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let chunk_section_position = decoder.i64()?;
        let suppress_light_updates = decoder.bool()?;
        let blocks = counted(decoder, |d| {
            Ok(SectionBlock {
                position: d.varlong()?,
                block_state: d.varint()?,
            })
        })?;
        Ok(Self {
            chunk_section_position,
            suppress_light_updates,
            blocks,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct SelectAdvancementsTab<'a> {
    pub has_id: bool,
    pub tab_id: Option<&'a str>,
}

#[derive(Decode, Debug, Clone)]
pub struct ServerData<'a> {
    pub has_motd: bool,
    pub motd: Option<&'a str>,
    pub has_icon: bool,
    pub icon: Option<&'a str>,
    pub previews_chat: bool,
    pub enforces_secure_chat: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetActionBarText<'a> {
    pub action_bar_text: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderCenter {
    pub x: f64,
    pub z: f64,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderLerpSize {
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderSize {
    pub diameter: f64,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderWarningDelay {
    pub warning_time: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBorderWarningDistance {
    pub warning_blocks: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetCamera {
    pub camera_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetHeldItem {
    pub slot: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct SetCenterChunk {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetRenderDistance {
    pub view_distance: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetDefaultSpawnPosition {
    pub location: u64,
    pub angle: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct SetDisplayChatPreview {
    pub chat_preview_setting: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct DisplayObjective<'a> {
    pub position: u8,
    pub score_name: &'a str,
}

/// One metadata entry. The value type is chosen by the entry's index, so the
/// payload is kept as a typed enum rather than raw bytes.
#[derive(Debug, Clone)]
pub struct MetadataEntry<'a> {
    pub index: u8,
    pub value: MetadataValue<'a>,
}

/// The metadata value types of protocol 760.
#[derive(Debug, Clone)]
pub enum MetadataValue<'a> {
    Byte(i8),
    VarInt(VarInt),
    Float(f32),
    String(&'a str),
    Chat(&'a str),
    OptChat(Option<&'a str>),
    Slot(Slot<'a>),
    Boolean(bool),
    Rotation([f32; 3]),
    Position(u64),
    OptPosition(Option<u64>),
    Direction(VarInt),
    OptUuid(Option<Uuid>),
    OptBlockId(Option<VarInt>),
    Nbt(NbtTag<'a>),
    Particle,
    VillagerData { kind: VarInt, profession: VarInt, level: VarInt },
    OptVarInt(Option<VarInt>),
    Pose(VarInt),
}

impl<'a> MetadataValue<'a> {
    /// Reads the value for metadata index `index`.
    fn read(decoder: &mut Decoder<'a>, index: u8) -> Result<Self, Error> {
        Ok(match index {
            0 => Self::Byte(decoder.i8()?),
            1 => Self::VarInt(decoder.varint()?),
            2 => Self::Float(decoder.f32()?),
            3 => Self::String(decoder.string()?),
            4 => Self::Chat(decoder.string()?),
            5 => Self::OptChat(optional_chat(decoder)?),
            6 => Self::Slot(Slot::decode(decoder)?),
            7 => Self::Boolean(decoder.bool()?),
            8 => Self::Rotation([decoder.f32()?, decoder.f32()?, decoder.f32()?]),
            9 => Self::Position(decoder.u64()?),
            10 => Self::OptPosition(if decoder.bool()? {
                Some(decoder.u64()?)
            } else {
                None
            }),
            11 => Self::Direction(decoder.varint()?),
            12 => Self::OptUuid(if decoder.bool()? {
                Some(decoder.uuid()?)
            } else {
                None
            }),
            13 => Self::OptBlockId(if decoder.varint()?.get() == 0 {
                None
            } else {
                Some(decoder.varint()?)
            }),
            14 => Self::Nbt(decoder.nbt()?),
            // Index 15 is a particle: its layout depends on the particle id,
            // which this proxy does not model, so it is rejected rather than
            // silently mis-parsed.
            15 => return Err(Error::Nbt("particle metadata is not supported")),
            16 => Self::VillagerData {
                kind: decoder.varint()?,
                profession: decoder.varint()?,
                level: decoder.varint()?,
            },
            17 => Self::OptVarInt(if decoder.bool()? {
                Some(decoder.varint()?)
            } else {
                None
            }),
            18 => Self::Pose(decoder.varint()?),
            other => return Err(Error::UnknownOpcode(other as i32)),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SetEntityMetadata<'a> {
    pub entity_id: VarInt,
    pub metadata: Vec<MetadataEntry<'a>>,
}

impl<'a> Decode<'a> for SetEntityMetadata<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_id = decoder.varint()?;
        let mut metadata = Vec::new();
        // Entries are terminated by a `0xFF` index byte.
        loop {
            let index = decoder.u8()?;
            if index == 0xFF {
                break;
            }
            let value = MetadataValue::read(decoder, index)?;
            metadata.push(MetadataEntry { index, value });
        }
        Ok(Self { entity_id, metadata })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct LinkEntities {
    pub attached_entity_id: i32,
    pub holding_entity_id: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct SetEntityVelocity {
    pub entity_id: VarInt,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
}

/// One equipped item. `slot` is the raw equipment-slot id (0-5).
#[derive(Debug, Clone)]
pub struct EquipmentEntry<'a> {
    pub slot: i8,
    pub item: Slot<'a>,
}

#[derive(Debug, Clone)]
pub struct SetEquipment<'a> {
    pub entity_id: VarInt,
    pub equipment: Vec<EquipmentEntry<'a>>,
}

impl<'a> Decode<'a> for SetEquipment<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_id = decoder.varint()?;
        let mut equipment = Vec::new();
        // The high bit of the slot byte means "another entry follows".
        loop {
            let raw = decoder.i8()?;
            let slot = raw & 0x7F;
            let item = Slot::decode(decoder)?;
            equipment.push(EquipmentEntry { slot, item });
            if raw & 0x80u8 as i8 == 0 {
                break;
            }
        }
        Ok(Self {
            entity_id,
            equipment,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct SetExperience {
    pub experience_bar: f32,
    pub level: VarInt,
    pub total_experience: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetHealth {
    pub health: f32,
    pub food: VarInt,
    pub food_saturation: f32,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateObjectives<'a> {
    pub objective_name: &'a str,
    pub mode: u8,
    pub objective_value: Option<&'a str>,
    pub kind: Option<VarInt>,
}

#[derive(Debug, Clone)]
pub struct SetPassengers {
    pub entity_id: VarInt,
    pub passengers: Vec<VarInt>,
}

impl<'a> Decode<'a> for SetPassengers {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_id = decoder.varint()?;
        let passengers = counted(decoder, |d| d.varint())?;
        Ok(Self {
            entity_id,
            passengers,
        })
    }
}

/// The `mode`-dependent body of [`UpdateTeams`].
#[derive(Debug, Clone)]
pub enum TeamAction<'a> {
    /// `0` — a new team is created.
    Create {
        display_name: &'a str,
        friendly_flags: u8,
        name_tag_visibility: &'a str,
        collision_rule: &'a str,
        color: VarInt,
        prefix: &'a str,
        suffix: &'a str,
        entities: Vec<&'a str>,
    },
    /// `1` — the team is removed; nothing follows.
    Remove,
    /// `2` — only the display metadata changed.
    Update {
        display_name: &'a str,
        friendly_flags: u8,
        name_tag_visibility: &'a str,
        collision_rule: &'a str,
        color: VarInt,
        prefix: &'a str,
        suffix: &'a str,
    },
    /// `3` — entities joined the team.
    AddEntities { entities: Vec<&'a str> },
    /// `4` — entities left the team.
    RemoveEntities { entities: Vec<&'a str> },
}

#[derive(Debug, Clone)]
pub struct UpdateTeams<'a> {
    pub team_name: &'a str,
    pub action: TeamAction<'a>,
}

impl<'a> Decode<'a> for UpdateTeams<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let team_name = decoder.string()?;
        let action = match decoder.u8()? {
            0 => TeamAction::Create {
                display_name: decoder.string()?,
                friendly_flags: decoder.u8()?,
                name_tag_visibility: decoder.string()?,
                collision_rule: decoder.string()?,
                color: decoder.varint()?,
                prefix: decoder.string()?,
                suffix: decoder.string()?,
                entities: identifiers(decoder)?,
            },
            1 => TeamAction::Remove,
            2 => TeamAction::Update {
                display_name: decoder.string()?,
                friendly_flags: decoder.u8()?,
                name_tag_visibility: decoder.string()?,
                collision_rule: decoder.string()?,
                color: decoder.varint()?,
                prefix: decoder.string()?,
                suffix: decoder.string()?,
            },
            3 => TeamAction::AddEntities {
                entities: identifiers(decoder)?,
            },
            4 => TeamAction::RemoveEntities {
                entities: identifiers(decoder)?,
            },
            other => return Err(Error::UnknownOpcode(other as i32)),
        };
        Ok(Self { team_name, action })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateScore<'a> {
    pub entity_name: &'a str,
    pub action: VarInt,
    pub objective_name: &'a str,
    pub value: Option<VarInt>,
}

#[derive(Decode, Debug, Clone)]
pub struct SetSimulationDistance {
    pub simulation_distance: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetSubtitleText<'a> {
    pub subtitle_text: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateTime {
    pub world_age: i64,
    pub time_of_day: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct SetTitleText<'a> {
    pub title_text: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct SetTitleAnimationTimes {
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct EntitySoundEffect {
    pub sound_id: VarInt,
    pub sound_category: VarInt,
    pub entity_id: VarInt,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct SoundEffect {
    pub sound_id: VarInt,
    pub sound_category: VarInt,
    pub effect_position_x: i32,
    pub effect_position_y: i32,
    pub effect_position_z: i32,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

#[derive(Decode, Debug, Clone)]
pub struct StopSound<'a> {
    pub flags: u8,
    pub source: Option<VarInt>,
    pub sound: Option<&'a str>,
}

#[derive(Decode, Debug, Clone)]
pub struct SystemChatMessage<'a> {
    pub json_data: &'a str,
    pub overlay: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetTabListHeaderAndFooter<'a> {
    pub header: &'a str,
    pub footer: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct TagQueryResponse<'a> {
    pub transaction_id: VarInt,
    pub nbt: NbtTag<'a>,
}

#[derive(Decode, Debug, Clone)]
pub struct PickupItem {
    pub collected_entity_id: VarInt,
    pub collector_entity_id: VarInt,
    pub pickup_item_count: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct TeleportEntity {
    pub entity_id: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

/// One advancement's progress.
#[derive(Debug, Clone)]
pub struct AdvancementProgress<'a> {
    pub id: &'a str,
    /// `None` when the advancement has no parent.
    pub parent_id: Option<&'a str>,
    /// `None` when the advancement has no display data.
    pub display: Option<AdvancementDisplay<'a>>,
    pub criteria: Vec<&'a str>,
}

/// The display block of an advancement.
#[derive(Debug, Clone)]
pub struct AdvancementDisplay<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub icon: Slot<'a>,
    pub frame_type: VarInt,
    pub flags: i32,
    /// Only present when the `0x01` flag (has background) is set.
    pub background_texture: Option<&'a str>,
    pub x_coord: f32,
    pub y_coord: f32,
}

/// One advancement that was removed or whose progress changed.
#[derive(Debug, Clone)]
pub struct AdvancementRemoval<'a> {
    pub id: &'a str,
}

#[derive(Debug, Clone)]
pub struct UpdateAdvancements<'a> {
    pub reset: bool,
    pub added: Vec<AdvancementProgress<'a>>,
    pub removed: Vec<&'a str>,
    pub progress: Vec<AdvancementProgressUpdate<'a>>,
}

/// A progress-only update: the advancement id plus its completed criteria.
#[derive(Debug, Clone)]
pub struct AdvancementProgressUpdate<'a> {
    pub id: &'a str,
    pub criteria: Vec<&'a str>,
}

impl<'a> Decode<'a> for UpdateAdvancements<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let reset = decoder.bool()?;

        let added = counted(decoder, |d| {
            let id = d.string()?;
            let parent_id = if d.bool()? { Some(d.string()?) } else { None };
            let display = if d.bool()? {
                let title = d.string()?;
                let description = d.string()?;
                let icon = Slot::decode(d)?;
                let frame_type = d.varint()?;
                let flags = d.i32()?;
                // Bit 0 of `flags` marks a background texture.
                let background_texture = if flags & 0x01 != 0 {
                    Some(d.string()?)
                } else {
                    None
                };
                let x_coord = d.f32()?;
                let y_coord = d.f32()?;
                Some(AdvancementDisplay {
                    title,
                    description,
                    icon,
                    frame_type,
                    flags,
                    background_texture,
                    x_coord,
                    y_coord,
                })
            } else {
                None
            };
            let criteria = identifiers(d)?;
            Ok(AdvancementProgress {
                id,
                parent_id,
                display,
                criteria,
            })
        })?;

        let removed = identifiers(decoder)?;

        let progress = counted(decoder, |d| {
            Ok(AdvancementProgressUpdate {
                id: d.string()?,
                criteria: identifiers(d)?,
            })
        })?;

        Ok(Self {
            reset,
            added,
            removed,
            progress,
        })
    }
}

/// One attribute modifier.
#[derive(Debug, Clone)]
pub struct AttributeModifier {
    pub uuid: Uuid,
    pub amount: f64,
    pub operation: u8,
}

/// One attribute of an entity, with its base value and modifiers.
#[derive(Debug, Clone)]
pub struct AttributeEntry<'a> {
    pub key: &'a str,
    pub value: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(Debug, Clone)]
pub struct UpdateAttributes<'a> {
    pub entity_id: VarInt,
    pub properties: Vec<AttributeEntry<'a>>,
}

impl<'a> Decode<'a> for UpdateAttributes<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let entity_id = decoder.varint()?;
        let properties = counted(decoder, |d| {
            let key = d.string()?;
            let value = d.f64()?;
            let modifiers = counted(d, |d| {
                Ok(AttributeModifier {
                    uuid: d.uuid()?,
                    amount: d.f64()?,
                    operation: d.u8()?,
                })
            })?;
            Ok(AttributeEntry {
                key,
                value,
                modifiers,
            })
        })?;
        Ok(Self {
            entity_id,
            properties,
        })
    }
}

#[derive(Decode, Debug, Clone)]
pub struct EntityEffect<'a> {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
    pub amplifier: i8,
    pub duration: VarInt,
    pub flags: u8,
    pub has_factor_data: bool,
    pub factor_data: Option<NbtTag<'a>>,
}

/// One recipe: an identifier plus an opaque, type-dependent payload.
#[derive(Debug, Clone)]
pub struct Recipe<'a> {
    pub id: &'a str,
    /// The recipe body. Its layout depends on the recipe `type`, which is a
    /// registry entry this proxy does not model, so the bytes are kept as-is.
    pub data: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct UpdateRecipes<'a> {
    pub recipes: Vec<Recipe<'a>>,
}

impl<'a> Decode<'a> for UpdateRecipes<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let recipes = counted(decoder, |d| {
            let id = d.string()?;
            // The recipe type is a VarInt registry id; the body that follows
            // has no length prefix, so it cannot be delimited without knowing
            // every recipe type. The whole packet is therefore consumed here.
            let _recipe_type = d.varint()?;
            Ok(Recipe {
                id,
                data: d.rest(),
            })
        })?;
        Ok(Self { recipes })
    }
}

/// One named tag group: an identifier and the entries it contains.
#[derive(Debug, Clone)]
pub struct TagGroup<'a> {
    pub name: &'a str,
    pub entries: Vec<VarInt>,
}

/// All tag groups of one registry type.
#[derive(Debug, Clone)]
pub struct TagRegistry<'a> {
    pub registry: &'a str,
    pub tags: Vec<TagGroup<'a>>,
}

#[derive(Debug, Clone)]
pub struct UpdateTags<'a> {
    pub registries: Vec<TagRegistry<'a>>,
}

impl<'a> Decode<'a> for UpdateTags<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let registries = counted(decoder, |d| {
            Ok(TagRegistry {
                registry: d.string()?,
                tags: counted(d, |d| {
                    Ok(TagGroup {
                        name: d.string()?,
                        entries: counted(d, |d| d.varint())?,
                    })
                })?,
            })
        })?;
        Ok(Self { registries })
    }
}
