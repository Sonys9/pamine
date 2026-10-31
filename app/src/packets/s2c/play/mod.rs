//! Clientbound play packets (protocol 760 / 1.19.2).
//!
//! Packet ids follow the 1.19.2 table. The packet bodies are grouped into
//! submodules by subject; this module only holds the [`Play`] dispatch enum and
//! re-exports every body so the variants can name them directly.

pub mod advancement;
pub mod blocks;
pub mod chat;
pub mod chunk;
pub mod command;
pub mod entity;
pub mod inventory;
pub mod misc;
pub mod particle;
pub mod player;
pub mod scoreboard;
pub mod sound;
pub mod ui;
pub mod world;

pub use advancement::*;
pub use blocks::*;
pub use chat::*;
pub use chunk::*;
pub use command::*;
pub use entity::*;
pub use inventory::*;
pub use misc::*;
pub use player::*;
pub use scoreboard::*;
pub use sound::*;
pub use ui::*;
pub use world::*;

use pcore_gen::{DecodePacket, EncodePacket};

#[derive(DecodePacket, EncodePacket, Clone, Debug)]
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
