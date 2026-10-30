//! Clientbound play packets (protocol 760 / 1.19.2).
//!
//! Packet ids follow the 1.19.2 table. Fields that are variable-length arrays,
//! NBT payloads or depend on a preceding discriminator are kept as raw `&[u8]`
//! slices; the derive macro cannot express them without a custom `Decode` impl.

use pcore::{
    Decode,
    decoder::{NbtTag, Uuid, VarInt},
};
use pcore_gen::DecodePacket;

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
    Explosion(Explosion<'a>),
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
    RemoveEntities(RemoveEntities<'a>),
    #[id(0x3C)]
    RemoveEntityEffect(RemoveEntityEffect),
    #[id(0x3D)]
    ResourcePack(ResourcePack<'a>),
    #[id(0x3E)]
    Respawn(Respawn<'a>),
    #[id(0x3F)]
    SetHeadRotation(SetHeadRotation),
    #[id(0x40)]
    UpdateSectionBlocks(UpdateSectionBlocks<'a>),
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
    SetPassengers(SetPassengers<'a>),
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

#[derive(Decode, Debug, Clone)]
pub struct AwardStatistics<'a> {
    pub count: VarInt,
    pub statistics: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
pub struct BossBar<'a> {
    pub uuid: Uuid,
    pub action: VarInt,
    pub payload: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
pub struct CommandSuggestionsResponse<'a> {
    pub id: VarInt,
    pub start: VarInt,
    pub length: VarInt,
    pub matches: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct Commands<'a> {
    pub count: VarInt,
    pub nodes: &'a [u8],
    pub root_index: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct CloseContainer {
    pub window_id: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct SetContainerContent<'a> {
    pub window_id: u8,
    pub state_id: VarInt,
    pub count: VarInt,
    pub slots: &'a [u8],
    pub carried_item: &'a [u8],
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
    pub slot_data: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct SetCooldown {
    pub item_id: VarInt,
    pub cooldown_ticks: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct ChatSuggestions<'a> {
    pub action: VarInt,
    pub count: VarInt,
    pub entries: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct PluginMessage<'a> {
    pub channel: &'a str,
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

#[derive(Decode, Debug, Clone)]
pub struct Explosion<'a> {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub strength: f32,
    pub record_count: VarInt,
    pub records: &'a [u8],
    pub player_motion_x: f32,
    pub player_motion_y: f32,
    pub player_motion_z: f32,
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

#[derive(Decode, Debug, Clone)]
pub struct ChunkDataAndUpdateLight<'a> {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub heightmaps: NbtTag<'a>,
    pub size: VarInt,
    pub data: &'a [u8],
    pub block_entities: &'a [u8],
    pub trust_edges: bool,
    pub light_data: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
pub struct UpdateLight<'a> {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
    pub trust_edges: bool,
    pub light_data: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct Login<'a> {
    pub entity_id: i32,
    pub is_hardcore: bool,
    pub gamemode: u8,
    pub previous_gamemode: i8,
    pub dimension_count: VarInt,
    pub dimension_names: &'a [u8],
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
    pub has_death_location: bool,
    pub death_dimension_name: Option<&'a str>,
    pub death_location: Option<u64>,
}

#[derive(Decode, Debug, Clone)]
pub struct MapData<'a> {
    pub map_id: VarInt,
    pub scale: i8,
    pub locked: bool,
    pub tracking_position: bool,
    pub icons: &'a [u8],
    pub columns: u8,
    pub rows: Option<u8>,
    pub x: Option<i8>,
    pub z: Option<i8>,
    pub data: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct MerchantOffers<'a> {
    pub window_id: VarInt,
    pub size: VarInt,
    pub trades: &'a [u8],
    pub villager_level: VarInt,
    pub experience: VarInt,
    pub is_regular_villager: bool,
    pub can_restock: bool,
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

#[derive(Decode, Debug, Clone)]
pub struct PlayerChatMessage<'a> {
    pub sender: Uuid,
    pub index: VarInt,
    pub signature: Option<&'a [u8]>,
    pub message: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    pub previous_messages: &'a [u8],
    pub unsigned_content: Option<&'a str>,
    pub filter_type: VarInt,
    pub network_target: VarInt,
    pub network_name: &'a str,
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

#[derive(Decode, Debug, Clone)]
pub struct PlayerInfo<'a> {
    pub actions: u8,
    pub number_of_players: VarInt,
    pub players: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
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
    pub recipe_ids: &'a [u8],
    pub recipe_ids_2: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct RemoveEntities<'a> {
    pub count: VarInt,
    pub entity_ids: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
pub struct UpdateSectionBlocks<'a> {
    pub chunk_section_position: i64,
    pub suppress_light_updates: bool,
    pub blocks_array_size: VarInt,
    pub blocks: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
pub struct SetEntityMetadata<'a> {
    pub entity_id: VarInt,
    pub metadata: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
pub struct SetEquipment<'a> {
    pub entity_id: VarInt,
    pub equipment: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
pub struct SetPassengers<'a> {
    pub entity_id: VarInt,
    pub passenger_count: VarInt,
    pub passengers: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateTeams<'a> {
    pub team_name: &'a str,
    pub mode: u8,
    pub payload: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
pub struct UpdateAdvancements<'a> {
    pub reset: bool,
    pub mapping_size: VarInt,
    pub advancement_mapping: &'a [u8],
    pub list_size: VarInt,
    pub identifiers: &'a [u8],
    pub progress_size: VarInt,
    pub progress_mapping: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateAttributes<'a> {
    pub entity_id: VarInt,
    pub number_of_properties: VarInt,
    pub properties: &'a [u8],
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

#[derive(Decode, Debug, Clone)]
pub struct UpdateRecipes<'a> {
    pub num_recipes: VarInt,
    pub recipes: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateTags<'a> {
    pub length_of_the_array: VarInt,
    pub tags: &'a [u8],
}
