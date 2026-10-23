use pcore::{Decode, decoder::VarInt};
use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Play<'a> {
    #[id(0x00)]
    TeleportConfirm(TeleportConfirm),
    #[id(0x01)]
    QueryBlockEntityNbt(QueryBlockEntityNbt),
    #[id(0x02)]
    ChatCommand(ChatCommand<'a>),
    #[id(0x03)]
    ChatMessage(ChatMessage<'a>),
    #[id(0x04)]
    ChatPreview(ChatPreview<'a>),
    #[id(0x05)]
    ClientCommand(ClientCommand),
    #[id(0x06)]
    ClientSettings(ClientSettings<'a>),
    #[id(0x07)]
    CommandSuggestionsRequest(CommandSuggestionsRequest<'a>),
    #[id(0x08)]
    ClickContainerButton(ClickContainerButton),
    #[id(0x09)]
    ClickContainer(ClickContainer<'a>),
    #[id(0x0A)]
    CloseContainer(CloseContainer),
    #[id(0x0B)]
    PluginMessage(PluginMessage<'a>),
    #[id(0x0C)]
    EditBook(EditBook<'a>),
    #[id(0x0D)]
    QueryEntityNbt(QueryEntityNbt),
    #[id(0x0E)]
    Interact(Interact),
    #[id(0x0F)]
    JigsawGenerate(JigsawGenerate),
    #[id(0x10)]
    KeepAlive(KeepAlive),
    #[id(0x11)]
    LockDifficulty(LockDifficulty),
    #[id(0x12)]
    SetPlayerPosition(SetPlayerPosition),
    #[id(0x13)]
    SetPlayerPositionAndRotation(SetPlayerPositionAndRotation),
    #[id(0x14)]
    SetPlayerRotation(SetPlayerRotation),
    #[id(0x15)]
    SetPlayerOnGround(SetPlayerOnGround),
    #[id(0x16)]
    MoveVehicle(MoveVehicle),
    #[id(0x17)]
    PaddleBoat(PaddleBoat),
    #[id(0x18)]
    PickItem(PickItem),
    #[id(0x19)]
    PlaceRecipe(PlaceRecipe),
    #[id(0x1A)]
    PlayerAbilities(PlayerAbilities),
    #[id(0x1B)]
    PlayerDigging(PlayerDigging),
    #[id(0x1C)]
    PlayerAction(PlayerAction),
    #[id(0x1D)]
    Pong(Pong),
    #[id(0x1E)]
    RecipeBookChangeSettings(RecipeBookChangeSettings),
    #[id(0x1F)]
    RenameItem(RenameItem<'a>),
    #[id(0x20)]
    ResourcePackStatus(ResourcePackStatus),
    #[id(0x21)]
    SetSeenAdvancements(SetSeenAdvancements),
    #[id(0x22)]
    SelectTrade(SelectTrade),
    #[id(0x23)]
    SetBeaconEffect(SetBeaconEffect),
    #[id(0x24)]
    HeldItemChange(HeldItemChange),
    #[id(0x25)]
    UpdateCommandBlock(UpdateCommandBlock<'a>),
    #[id(0x26)]
    UpdateCommandBlockMinecart(UpdateCommandBlockMinecart<'a>),
    #[id(0x27)]
    CreativeInventoryAction(CreativeInventoryAction<'a>),
    #[id(0x28)]
    UpdateJigsawBlock(UpdateJigsawBlock<'a>),
    #[id(0x29)]
    UpdateStructureBlock(UpdateStructureBlock<'a>),
    #[id(0x2A)]
    UpdateSign(UpdateSign<'a>),
    #[id(0x2B)]
    SwingArm(SwingArm),
    #[id(0x2C)]
    TeleportToEntity(TeleportToEntity),
    #[id(0x2D)]
    UseItemOn(UseItemOn),
    #[id(0x2E)]
    UseItem(UseItem),
}

#[derive(Decode, Debug, Clone)]
pub struct TeleportConfirm {
    pub teleport_id: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct QueryBlockEntityNbt {
    pub transaction_id: VarInt,
    pub location: u64,
}
#[derive(Decode, Debug, Clone)]
pub struct ChatCommand<'a> {
    pub command: &'a [u8],
    pub timestamp: u64,
    pub salt: u64,
    pub argument_signatures: VarInt,
    pub signed_preview: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct ChatMessage<'a> {
    pub message: &'a [u8],
    pub timestamp: u64,
    pub salt: u64,
    pub signature: Option<&'a [u8]>,
    pub signed_preview: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct ChatPreview<'a> {
    pub query_id: i32,
    pub query: &'a [u8],
}
#[derive(Decode, Debug, Clone)]
pub struct ClientCommand {
    pub action_id: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct ClientSettings<'a> {
    pub locale: &'a [u8],
    pub view_distance: i8,
    pub chat_mode: VarInt,
    pub chat_colors: bool,
    pub displayed_skin_parts: u8,
    pub main_hand: VarInt,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct CommandSuggestionsRequest<'a> {
    pub transaction_id: VarInt,
    pub text: &'a [u8],
}
#[derive(Decode, Debug, Clone)]
pub struct ClickContainerButton {
    pub window_id: i8,
    pub button_id: i8,
}
#[derive(Decode, Debug, Clone)]
pub struct ClickContainer<'a> {
    pub window_id: i8,
    pub state_id: VarInt,
    pub slot_idx: i16,
    pub button: i8,
    pub mode: VarInt,
    pub data: &'a [u8],
}
#[derive(Decode, Debug, Clone)]
pub struct CloseContainer {
    pub window_id: i8,
}
#[derive(Decode, Debug, Clone)]
pub struct PluginMessage<'a> {
    pub channel: &'a [u8],
    pub data: &'a [u8],
}
#[derive(Decode, Debug, Clone)]
pub struct EditBook<'a> {
    pub slot: VarInt,
    pub pages: VarInt,
    pub title: Option<&'a [u8]>,
}
#[derive(Decode, Debug, Clone)]
pub struct QueryEntityNbt {
    pub transaction_id: VarInt,
    pub entity_id: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct Interact {
    pub entity_id: VarInt,
    pub target_action: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct JigsawGenerate {
    pub location: u64,
    pub levels: VarInt,
    pub keep_jigsaws: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct KeepAlive {
    pub keep_alive_id: i64,
}
#[derive(Decode, Debug, Clone)]
pub struct LockDifficulty {
    pub locked: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct SetPlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct SetPlayerPositionAndRotation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct SetPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct SetPlayerOnGround {
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
pub struct PaddleBoat {
    pub left_paddle_down: bool,
    pub right_paddle_down: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct PickItem {
    pub slot: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct PlaceRecipe {
    pub window_id: i8,
    pub recipe: VarInt,
    pub make_all: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct PlayerAbilities {
    pub flags: i8,
}
#[derive(Decode, Debug, Clone)]
pub struct PlayerDigging {
    pub status: VarInt,
    pub location: u64,
    pub face: i8,
}
#[derive(Decode, Debug, Clone)]
pub struct PlayerAction {
    pub action_id: VarInt,
    pub data: VarInt,
    pub sequence: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct Pong {
    pub id: i32,
}
#[derive(Decode, Debug, Clone)]
pub struct RecipeBookChangeSettings {
    pub book_id: VarInt,
    pub book_open: bool,
    pub filter_active: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct RenameItem<'a> {
    pub item_name: &'a [u8],
}
#[derive(Decode, Debug, Clone)]
pub struct ResourcePackStatus {
    pub result: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct SetSeenAdvancements {
    pub action: VarInt,
    pub tab_id: Option<u64>,
}
#[derive(Decode, Debug, Clone)]
pub struct SelectTrade {
    pub selected_slot: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct SetBeaconEffect {
    pub primary_effect: Option<VarInt>,
    pub secondary_effect: Option<VarInt>,
}
#[derive(Decode, Debug, Clone)]
pub struct HeldItemChange {
    pub slot: i16,
}
#[derive(Decode, Debug, Clone)]
pub struct UpdateCommandBlock<'a> {
    pub location: u64,
    pub command: &'a [u8],
    pub mode: VarInt,
    pub flags: i8,
}
#[derive(Decode, Debug, Clone)]
pub struct UpdateCommandBlockMinecart<'a> {
    pub entity_id: VarInt,
    pub command: &'a [u8],
    pub track_output: bool,
}
#[derive(Decode, Debug, Clone)]
pub struct CreativeInventoryAction<'a> {
    pub slot: i16,
    pub clicked_item: &'a [u8],
}
#[derive(Decode, Debug, Clone)]
pub struct UpdateJigsawBlock<'a> {
    pub location: u64,
    pub name: &'a [u8],
    pub target: &'a [u8],
    pub pool: &'a [u8],
    pub final_state: &'a [u8],
    pub joint: &'a [u8],
}
#[derive(Decode, Debug, Clone)]
pub struct UpdateStructureBlock<'a> {
    pub location: u64,
    pub action: VarInt,
    pub mode: VarInt,
    pub name: &'a [u8],
    pub offset_x: i8,
    pub offset_y: i8,
    pub offset_z: i8,
    pub size_x: i8,
    pub size_y: i8,
    pub size_z: i8,
    pub mirror: VarInt,
    pub rotation: VarInt,
    pub metadata: &'a [u8],
    pub integrity: f32,
    pub seed: i64,
    pub flags: i8,
}
#[derive(Decode, Debug, Clone)]
pub struct UpdateSign<'a> {
    pub location: u64,
    pub line_1: &'a [u8],
    pub line_2: &'a [u8],
    pub line_3: &'a [u8],
    pub line_4: &'a [u8],
}
#[derive(Decode, Debug, Clone)]
pub struct SwingArm {
    pub hand: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct TeleportToEntity {
    pub target: pcore::decoder::Uuid,
}
#[derive(Decode, Debug, Clone)]
pub struct UseItemOn {
    pub hand: VarInt,
    pub location: u64,
    pub face: VarInt,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
    pub inside_block: bool,
    pub sequence: VarInt,
}
#[derive(Decode, Debug, Clone)]
pub struct UseItem {
    pub hand: VarInt,
    pub sequence: VarInt,
}
