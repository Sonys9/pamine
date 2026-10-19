use pcore::{Decode, decoder::VarInt};
use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Play<'a> {
    #[id(0x00)] SpawnEntity(SpawnEntity),
    #[id(0x01)] SpawnExperienceOrb(SpawnExperienceOrb),
    #[id(0x02)] SpawnPlayer(SpawnPlayer),
    #[id(0x03)] EntityAnimation(EntityAnimation),
    #[id(0x04)] AwardStatistics(AwardStatistics),
    #[id(0x05)] AcknowledgeBlockChange(AcknowledgeBlockChange),
    #[id(0x06)] SetBlockDestroyStage(SetBlockDestroyStage),
    #[id(0x07)] BlockEntityData(BlockEntityData),
    #[id(0x08)] BlockAction(BlockAction),
    #[id(0x09)] BlockUpdate(BlockUpdate),
    #[id(0x0A)] BossBar(BossBar<'a>),
    #[id(0x0B)] ChangeDifficulty(ChangeDifficulty),
    #[id(0x0C)] ChatPreview(ChatPreview<'a>),
    #[id(0x0D)] ClearTitles(ClearTitles),
    #[id(0x0E)] CommandSuggestionsResponse(CommandSuggestionsResponse),
    #[id(0x0F)] Commands(Commands<'a>),
    #[id(0x10)] CloseContainer(CloseContainer),
    #[id(0x11)] SetContainerContent(SetContainerContent<'a>),
    #[id(0x12)] SetContainerProperty(SetContainerProperty),
    #[id(0x13)] SetContainerSlot(SetContainerSlot<'a>),
    #[id(0x14)] SetCooldown(SetCooldown),
    #[id(0x15)] ChatSuggestions(ChatSuggestions),
    #[id(0x16)] PluginMessage(PluginMessage<'a>),
    #[id(0x17)] CustomSoundEffect(CustomSoundEffect<'a>),
    #[id(0x18)] HideMessage(HideMessage<'a>),
    #[id(0x19)] Disconnect(Disconnect<'a>),
    #[id(0x1A)] EntityEvent(EntityEvent),
    #[id(0x1B)] Explosion(Explosion),
    #[id(0x1C)] UnloadChunk(UnloadChunk),
    #[id(0x1D)] GameEvent(GameEvent),
    #[id(0x1E)] OpenHorseScreen(OpenHorseScreen),
    #[id(0x1F)] InitializeWorldBorder(InitializeWorldBorder),
    #[id(0x20)] KeepAlive(KeepAlive),
    #[id(0x21)] ChunkDataAndupdateLight(ChunkDataAndupdateLight<'a>),
    #[id(0x22)] WorldEvent(WorldEvent),
    #[id(0x23)] Particle(Particle<'a>),
    #[id(0x24)] UpdateLight(UpdateLight<'a>),
    #[id(0x25)] Login(Login),
    #[id(0x26)] MapData(MapData<'a>),
    #[id(0x27)] MerchantOffers(MerchantOffers<'a>),
    #[id(0x28)] UpdateEntityPosition(UpdateEntityPosition),
    #[id(0x29)] UpdateEntityPositionAndRotation(UpdateEntityPositionAndRotation),
    #[id(0x2A)] UpdateEntityRotation(UpdateEntityRotation),
    #[id(0x2B)] MoveVehicle(MoveVehicle),
    #[id(0x2C)] OpenBook(OpenBook),
    #[id(0x2D)] OpenScreen(OpenScreen<'a>),
    #[id(0x2E)] OpenSignEditor(OpenSignEditor),
    #[id(0x2F)] Ping(Ping),
    #[id(0x30)] PlaceGhostRecipe(PlaceGhostRecipe<'a>),
    #[id(0x31)] PlayerAbilities(PlayerAbilities),
    #[id(0x32)] MessageHeader(MessageHeader<'a>),
    #[id(0x33)] PlayerChat(PlayerChat<'a>),
    #[id(0x34)] EndCombat(EndCombat),
    #[id(0x35)] EnterCombat(EnterCombat),
    #[id(0x36)] CombatDeath(CombatDeath<'a>),
    #[id(0x37)] DisguisedChat(DisguisedChat<'a>),
    #[id(0x38)] PlayerInfoRemove(PlayerInfoRemove),
    #[id(0x39)] PlayerInfoUpdate(PlayerInfoUpdate),
    #[id(0x3A)] LookAt(LookAt),
    #[id(0x3B)] SynchronizePlayerPosition(SynchronizePlayerPosition),
    #[id(0x3C)] UpdateRecipeBook(UpdateRecipeBook),
    #[id(0x3D)] RemoveEntities(RemoveEntities),
    #[id(0x3E)] RemoveEntityEffect(RemoveEntityEffect),
    #[id(0x3F)] ResourcePackSend(ResourcePackSend<'a>),
    #[id(0x40)] Respawn(Respawn<'a>),
    #[id(0x41)] SetHeadRotation(SetHeadRotation),
    #[id(0x42)] UpdateSectionBlocks(UpdateSectionBlocks),
    #[id(0x43)] SelectAdvancementsTab(SelectAdvancementsTab<'a>),
    #[id(0x44)] ServerData(ServerData<'a>),
    #[id(0x45)] SetActionBarText(SetActionBarText<'a>),
    #[id(0x46)] SetBorderCenter(SetBorderCenter),
    #[id(0x47)] SetBorderLerpSize(SetBorderLerpSize),
    #[id(0x48)] SetBorderSize(SetBorderSize),
    #[id(0x49)] SetBorderWarningDelay(SetBorderWarningDelay),
    #[id(0x4A)] SetBorderWarningDistance(SetBorderWarningDistance),
    #[id(0x4B)] SetCamera(SetCamera),
    #[id(0x4C)] SetHeldItem(SetHeldItem),
    #[id(0x4D)] SetCenterChunk(SetCenterChunk),
    #[id(0x4E)] SetRenderDistance(SetRenderDistance),
    #[id(0x4F)] SetDefaultSpawnPosition(SetDefaultSpawnPosition),
    #[id(0x50)] DisplayObjective(DisplayObjective<'a>),
    #[id(0x51)] SetEntityMetadata(SetEntityMetadata<'a>),
    #[id(0x52)] LinkEntities(LinkEntities),
    #[id(0x53)] SetEntityVelocity(SetEntityVelocity),
    #[id(0x54)] SetEquipment(SetEquipment<'a>),
    #[id(0x55)] SetExperience(SetExperience),
    #[id(0x56)] SetHealth(SetHealth),
    #[id(0x57)] SetObjectives(SetObjectives<'a>),
    #[id(0x58)] SetPassengers(SetPassengers),
    #[id(0x59)] UpdateTeams(UpdateTeams<'a>),
    #[id(0x5A)] UpdateScore(UpdateScore<'a>),
    #[id(0x5B)] SetSimulationDistance(SetSimulationDistance),
    #[id(0x5C)] SetSubtitleText(SetSubtitleText<'a>),
    #[id(0x5D)] UpdateTime(UpdateTime),
    #[id(0x5E)] SetTitleText(SetTitleText<'a>),
    #[id(0x5F)] SetTitleTimes(SetTitleTimes),
    #[id(0x60)] EntitySoundEffect(EntitySoundEffect),
    #[id(0x61)] SoundEffect(SoundEffect),
    #[id(0x62)] StopSound(StopSound<'a>),
    #[id(0x63)] SystemChat(SystemChat<'a>),
    #[id(0x64)] SetTabListHeaderAndFooter(SetTabListHeaderAndFooter<'a>),
    #[id(0x65)] TagQueryResponse(TagQueryResponse),
    #[id(0x66)] PickupItem(PickupItem),
    #[id(0x67)] TeleportEntity(TeleportEntity),
    #[id(0x68)] UpdateAdvancements(UpdateAdvancements),
    #[id(0x69)] UpdateAttributes(UpdateAttributes<'a>),
    #[id(0x6A)] UpdateMobEffect(UpdateMobEffect),
}

#[derive(Decode, Debug, Clone)] pub struct SpawnEntity { pub entity_id: VarInt, pub entity_uuid: pcore::decoder::Uuid, pub type_id: VarInt, pub x: f64, pub y: f64, pub z: f64, pub pitch: i8, pub yaw: i8, pub head_yaw: i8, pub data: VarInt, pub velocity_x: i16, pub velocity_y: i16, pub velocity_z: i16 }
#[derive(Decode, Debug, Clone)] pub struct SpawnExperienceOrb { pub entity_id: VarInt, pub x: f64, pub y: f64, pub z: f64, pub count: i16 }
#[derive(Decode, Debug, Clone)] pub struct SpawnPlayer { pub entity_id: VarInt, pub player_uuid: pcore::decoder::Uuid, pub x: f64, pub y: f64, pub z: f64, pub yaw: i8, pub pitch: i8 }
#[derive(Decode, Debug, Clone)] pub struct EntityAnimation { pub entity_id: VarInt, pub animation: u8 }
#[derive(Decode, Debug, Clone)] pub struct AwardStatistics { pub count: VarInt }
#[derive(Decode, Debug, Clone)] pub struct AcknowledgeBlockChange { pub sequence: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SetBlockDestroyStage { pub entity_id: VarInt, pub location: u64, pub destroy_stage: i8 }
#[derive(Decode, Debug, Clone)] pub struct BlockEntityData { pub location: u64, pub action_type: VarInt, pub nbt_data: u64 }
#[derive(Decode, Debug, Clone)] pub struct BlockAction { pub location: u64, pub action_id: i8, pub action_param: i8, pub block_type: VarInt }
#[derive(Decode, Debug, Clone)] pub struct BlockUpdate { pub location: u64, pub block_id: VarInt }
#[derive(Decode, Debug, Clone)] pub struct BossBar<'a> { pub uuid: pcore::decoder::Uuid, pub action: VarInt, pub title: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct ChangeDifficulty { pub difficulty: i8, pub difficulty_locked: bool }
#[derive(Decode, Debug, Clone)] pub struct ChatPreview<'a> { pub query_id: i32, pub content: Option<&'a [u8]> }
#[derive(Decode, Debug, Clone)] pub struct ClearTitles { pub reset_times: bool }
#[derive(Decode, Debug, Clone)] pub struct CommandSuggestionsResponse { pub id: VarInt, pub start: VarInt, pub length: VarInt, pub matches: VarInt }
#[derive(Decode, Debug, Clone)] pub struct Commands<'a> { pub count: VarInt, pub raw_data: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct CloseContainer { pub window_id: i8 }
#[derive(Decode, Debug, Clone)] pub struct SetContainerContent<'a> { pub window_id: i8, pub state_id: VarInt, pub count: VarInt, pub data: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct SetContainerProperty { pub window_id: i8, pub property: i16, pub value: i16 }
#[derive(Decode, Debug, Clone)] pub struct SetContainerSlot<'a> { pub window_id: i8, pub state_id: VarInt, pub slot: i16, pub slot_data: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct SetCooldown { pub item_id: VarInt, pub cooldown_ticks: VarInt }
#[derive(Decode, Debug, Clone)] pub struct ChatSuggestions { pub action: VarInt, pub entries: VarInt }
#[derive(Decode, Debug, Clone)] pub struct PluginMessage<'a> { pub channel: &'a [u8], pub data: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct CustomSoundEffect<'a> { pub sound_name: &'a [u8], pub sound_category: VarInt, pub fixed_x: i32, pub fixed_y: i32, pub fixed_z: i32, pub volume: f32, pub pitch: f32, pub seed: u64 }
#[derive(Decode, Debug, Clone)] pub struct HideMessage<'a> { pub signature: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct Disconnect<'a> { pub reason: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct EntityEvent { pub entity_id: i32, pub entity_status: i8 }
#[derive(Decode, Debug, Clone)] pub struct Explosion { pub x: f32, pub y: f32, pub z: f32, pub radius: f32, pub record_count: i32 }
#[derive(Decode, Debug, Clone)] pub struct UnloadChunk { pub chunk_x: i32, pub chunk_z: i32 }
#[derive(Decode, Debug, Clone)] pub struct GameEvent { pub event: u8, pub value: f32 }
#[derive(Decode, Debug, Clone)] pub struct OpenHorseScreen { pub window_id: i8, pub slot_count: VarInt, pub entity_id: i32 }
#[derive(Decode, Debug, Clone)] pub struct InitializeWorldBorder { pub x: f64, pub z: f64, pub old_diameter: f64, pub new_diameter: f64, pub speed: u64, pub portal_teleport_boundary: VarInt, pub warning_blocks: VarInt, pub warning_time: VarInt }
#[derive(Decode, Debug, Clone)] pub struct KeepAlive { pub keep_alive_id: i64 }
#[derive(Decode, Debug, Clone)] pub struct ChunkDataAndupdateLight<'a> { pub chunk_x: i32, pub chunk_z: i32, pub data: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct WorldEvent { pub event: i32, pub location: u64, pub data: i32, pub disable_relative_volume: bool }
#[derive(Decode, Debug, Clone)] pub struct Particle<'a> { pub particle_id: i32, pub long_distance: bool, pub x: f64, pub y: f64, pub z: f64, pub offset_x: f32, pub offset_y: f32, pub offset_z: f32, pub max_speed: f32, pub count: i32, pub data: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct UpdateLight<'a> { pub chunk_x: VarInt, pub chunk_z: VarInt, pub data: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct Login { pub entity_id: i32, pub hardcore: bool, pub gamemode: u8, pub previous_gamemode: i8, pub dimension_count: VarInt }
#[derive(Decode, Debug, Clone)] pub struct MapData<'a> { pub map_id: VarInt, pub scale: i8, pub locked: bool, pub data: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct MerchantOffers<'a> { pub window_id: VarInt, pub data: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct UpdateEntityPosition { pub entity_id: VarInt, pub delta_x: i16, pub delta_y: i16, pub delta_z: i16, pub on_ground: bool }
#[derive(Decode, Debug, Clone)] pub struct UpdateEntityPositionAndRotation { pub entity_id: VarInt, pub delta_x: i16, pub delta_y: i16, pub delta_z: i16, pub yaw: i8, pub pitch: i8, pub on_ground: bool }
#[derive(Decode, Debug, Clone)] pub struct UpdateEntityRotation { pub entity_id: VarInt, pub yaw: i8, pub pitch: i8, pub on_ground: bool }
#[derive(Decode, Debug, Clone)] pub struct MoveVehicle { pub x: f64, pub y: f64, pub z: f64, pub yaw: f32, pub pitch: f32 }
#[derive(Decode, Debug, Clone)] pub struct OpenBook { pub hand: VarInt }
#[derive(Decode, Debug, Clone)] pub struct OpenScreen<'a> { pub window_id: VarInt, pub window_type: VarInt, pub window_title: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct OpenSignEditor { pub location: u64, pub is_front_text: bool }
#[derive(Decode, Debug, Clone)] pub struct Ping { pub id: i32 }
#[derive(Decode, Debug, Clone)] pub struct PlaceGhostRecipe<'a> { pub window_id: i8, pub recipe: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct PlayerAbilities { pub flags: i8, pub flying_speed: f32, pub fov_modifier: f32 }
#[derive(Decode, Debug, Clone)] pub struct MessageHeader<'a> { pub sender: pcore::decoder::Uuid, pub signature: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct PlayerChat<'a> { pub sender: pcore::decoder::Uuid, pub index: VarInt, pub signature: Option<&'a [u8]>, pub message: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct EndCombat { pub duration: VarInt }
#[derive(Decode, Debug, Clone)] pub struct EnterCombat {}
#[derive(Decode, Debug, Clone)] pub struct CombatDeath<'a> { pub player_id: VarInt, pub message: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct DisguisedChat<'a> { pub message: &'a [u8], pub chat_type: VarInt, pub sender_name: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct PlayerInfoRemove { pub count: VarInt }
#[derive(Decode, Debug, Clone)] pub struct PlayerInfoUpdate { pub actions: u8, pub count: VarInt }
#[derive(Decode, Debug, Clone)] pub struct LookAt { pub anchor: VarInt, pub x: f64, pub y: f64, pub z: f64, pub is_entity: bool }
#[derive(Decode, Debug, Clone)] pub struct SynchronizePlayerPosition { pub x: f64, pub y: f64, pub z: f64, pub yaw: f32, pub pitch: f32, pub flags: i8, pub teleport_id: VarInt }
#[derive(Decode, Debug, Clone)] pub struct UpdateRecipeBook { pub action: VarInt, pub filtering_craftable: bool, pub smelting_filter_active: bool }
#[derive(Decode, Debug, Clone)] pub struct RemoveEntities { pub count: VarInt }
#[derive(Decode, Debug, Clone)] pub struct RemoveEntityEffect { pub entity_id: VarInt, pub effect_id: VarInt }
#[derive(Decode, Debug, Clone)] pub struct ResourcePackSend<'a> { pub url: &'a [u8], pub hash: &'a [u8], pub forced: bool, pub prompt_message: Option<&'a [u8]> }
#[derive(Decode, Debug, Clone)] pub struct Respawn<'a> { pub dimension_type: VarInt, pub dimension_name: &'a [u8], pub hashed_seed: u64, pub gamemode: u8, pub previous_gamemode: i8, pub is_debug: bool, pub is_flat: bool, pub copy_metadata: bool }
#[derive(Decode, Debug, Clone)] pub struct SetHeadRotation { pub entity_id: VarInt, pub head_yaw: i8 }
#[derive(Decode, Debug, Clone)] pub struct UpdateSectionBlocks { pub chunk_section_position: u64, pub blocks: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SelectAdvancementsTab<'a> { pub tab_id: Option<&'a [u8]> }
#[derive(Decode, Debug, Clone)] pub struct ServerData<'a> { pub motd: &'a [u8], pub icon: Option<&'a [u8]>, pub enforces_secure_chat: bool }
#[derive(Decode, Debug, Clone)] pub struct SetActionBarText<'a> { pub text: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct SetBorderCenter { pub x: f64, pub z: f64 }
#[derive(Decode, Debug, Clone)] pub struct SetBorderLerpSize { pub old_diameter: f64, pub new_diameter: f64, pub speed: u64 }
#[derive(Decode, Debug, Clone)] pub struct SetBorderSize { pub diameter: f64 }
#[derive(Decode, Debug, Clone)] pub struct SetBorderWarningDelay { pub warning_delay: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SetBorderWarningDistance { pub warning_distance: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SetCamera { pub camera_id: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SetHeldItem { pub slot: i8 }
#[derive(Decode, Debug, Clone)] pub struct SetCenterChunk { pub chunk_x: VarInt, pub chunk_z: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SetRenderDistance { pub view_distance: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SetDefaultSpawnPosition { pub location: u64, pub angle: f32 }
#[derive(Decode, Debug, Clone)] pub struct DisplayObjective<'a> { pub position: i8, pub score_name: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct SetEntityMetadata<'a> { pub entity_id: VarInt, pub metadata: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct LinkEntities { pub attached_entity_id: VarInt, pub holding_entity_id: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SetEntityVelocity { pub entity_id: VarInt, pub velocity_x: i16, pub velocity_y: i16, pub velocity_z: i16 }
#[derive(Decode, Debug, Clone)] pub struct SetEquipment<'a> { pub entity_id: VarInt, pub equipment: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct SetExperience { pub experience_bar: f32, pub total_experience: VarInt, pub level: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SetHealth { pub health: f32, pub food: VarInt, pub food_saturation: f32 }
#[derive(Decode, Debug, Clone)] pub struct SetObjectives<'a> { pub objective_name: &'a [u8], pub mode: i8 }
#[derive(Decode, Debug, Clone)] pub struct SetPassengers { pub entity_id: VarInt, pub passengers: VarInt }
#[derive(Decode, Debug, Clone)] pub struct UpdateTeams<'a> { pub team_name: &'a [u8], pub mode: i8 }
#[derive(Decode, Debug, Clone)] pub struct UpdateScore<'a> { pub entity_name: &'a [u8], pub action: i8, pub objective_name: &'a [u8], pub value: Option<VarInt> }
#[derive(Decode, Debug, Clone)] pub struct SetSimulationDistance { pub simulation_distance: VarInt }
#[derive(Decode, Debug, Clone)] pub struct SetSubtitleText<'a> { pub text: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct UpdateTime { pub world_age: i64, pub time_of_day: i64 }
#[derive(Decode, Debug, Clone)] pub struct SetTitleText<'a> { pub text: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct SetTitleTimes { pub fade_in: i32, pub stay: i32, pub fade_out: i32 }
#[derive(Decode, Debug, Clone)] pub struct EntitySoundEffect { pub sound_id: VarInt, pub sound_category: VarInt, pub entity_id: VarInt, pub volume: f32, pub pitch: f32, pub seed: u64 }
#[derive(Decode, Debug, Clone)] pub struct SoundEffect { pub sound_id: VarInt, pub sound_category: VarInt, pub x: i32, pub y: i32, pub z: i32, pub volume: f32, pub pitch: f32, pub seed: u64 }
#[derive(Decode, Debug, Clone)] pub struct StopSound<'a> { pub flags: i8, pub source: Option<VarInt>, pub sound: Option<&'a [u8]> }
#[derive(Decode, Debug, Clone)] pub struct SystemChat<'a> { pub content: &'a [u8], pub is_action_bar: bool }
#[derive(Decode, Debug, Clone)] pub struct SetTabListHeaderAndFooter<'a> { pub header: &'a [u8], pub footer: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct TagQueryResponse { pub transaction_id: VarInt, pub nbt: u64 }
#[derive(Decode, Debug, Clone)] pub struct PickupItem { pub collected_entity_id: VarInt, pub collector_entity_id: VarInt, pub pickup_item_count: VarInt }
#[derive(Decode, Debug, Clone)] pub struct TeleportEntity { pub entity_id: VarInt, pub x: f64, pub y: f64, pub z: f64, pub yaw: i8, pub pitch: i8, pub on_ground: bool }
#[derive(Decode, Debug, Clone)] pub struct UpdateAdvancements { pub reset: bool, pub mapping_size: VarInt }
#[derive(Decode, Debug, Clone)] pub struct UpdateAttributes<'a> { pub entity_id: VarInt, pub properties: &'a [u8] }
#[derive(Decode, Debug, Clone)] pub struct UpdateMobEffect { pub entity_id: VarInt, pub effect_id: VarInt, pub amplifier: i8, pub duration: VarInt, pub flags: i8 }