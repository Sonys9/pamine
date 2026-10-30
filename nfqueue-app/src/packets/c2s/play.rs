//! Serverbound play packets (protocol 760 / 1.19.2).
//!
//! Packet ids follow the 1.19.2 table. Fields that are variable-length arrays or
//! depend on a preceding discriminator are kept as raw `&[u8]` slices; the
//! decoder cannot express them without a custom `Decode` impl.

use pcore::{
    Decode,
    decoder::{Uuid, VarInt},
};
use pcore_gen::DecodePacket;

#[derive(DecodePacket, Clone, Debug)]
pub enum Play<'a> {
    #[id(0x00)]
    ConfirmTeleportation(ConfirmTeleportation),
    #[id(0x01)]
    QueryBlockEntityTag(QueryBlockEntityTag),
    #[id(0x02)]
    ChangeDifficulty(ChangeDifficulty),
    #[id(0x03)]
    MessageAcknowledgment(MessageAcknowledgment<'a>),
    #[id(0x04)]
    ChatCommand(ChatCommand<'a>),
    #[id(0x05)]
    ChatMessage(ChatMessage<'a>),
    #[id(0x06)]
    ChatPreview(ChatPreview<'a>),
    #[id(0x07)]
    ClientCommand(ClientCommand),
    #[id(0x08)]
    ClientInformation(ClientInformation<'a>),
    #[id(0x09)]
    CommandSuggestionsRequest(CommandSuggestionsRequest<'a>),
    #[id(0x0A)]
    ClickContainerButton(ClickContainerButton),
    #[id(0x0B)]
    ClickContainer(ClickContainer<'a>),
    #[id(0x0C)]
    CloseContainer(CloseContainer),
    #[id(0x0D)]
    PluginMessage(PluginMessage<'a>),
    #[id(0x0E)]
    EditBook(EditBook<'a>),
    #[id(0x0F)]
    QueryEntityTag(QueryEntityTag),
    #[id(0x10)]
    Interact(Interact),
    #[id(0x11)]
    JigsawGenerate(JigsawGenerate),
    #[id(0x12)]
    KeepAlive(KeepAlive),
    #[id(0x13)]
    LockDifficulty(LockDifficulty),
    #[id(0x14)]
    SetPlayerPosition(SetPlayerPosition),
    #[id(0x15)]
    SetPlayerPositionAndRotation(SetPlayerPositionAndRotation),
    #[id(0x16)]
    SetPlayerRotation(SetPlayerRotation),
    #[id(0x17)]
    SetPlayerOnGround(SetPlayerOnGround),
    #[id(0x18)]
    MoveVehicle(MoveVehicle),
    #[id(0x19)]
    PaddleBoat(PaddleBoat),
    #[id(0x1A)]
    PickItem(PickItem),
    #[id(0x1B)]
    PlaceRecipe(PlaceRecipe<'a>),
    #[id(0x1C)]
    PlayerAbilities(PlayerAbilities),
    #[id(0x1D)]
    PlayerAction(PlayerAction),
    #[id(0x1E)]
    PlayerCommand(PlayerCommand),
    #[id(0x1F)]
    PlayerInput(PlayerInput),
    #[id(0x20)]
    Pong(Pong),
    #[id(0x21)]
    ChangeRecipeBookSettings(ChangeRecipeBookSettings),
    #[id(0x22)]
    SetSeenRecipe(SetSeenRecipe<'a>),
    #[id(0x23)]
    RenameItem(RenameItem<'a>),
    #[id(0x24)]
    ResourcePackResponse(ResourcePackResponse),
    #[id(0x25)]
    SeenAdvancements(SeenAdvancements<'a>),
    #[id(0x26)]
    SelectTrade(SelectTrade),
    #[id(0x27)]
    SetBeaconEffect(SetBeaconEffect),
    #[id(0x28)]
    SetHeldItem(SetHeldItem),
    #[id(0x29)]
    ProgramCommandBlock(ProgramCommandBlock<'a>),
    #[id(0x2A)]
    ProgramCommandBlockMinecart(ProgramCommandBlockMinecart<'a>),
    #[id(0x2B)]
    SetCreativeModeSlot(SetCreativeModeSlot<'a>),
    #[id(0x2C)]
    ProgramJigsawBlock(ProgramJigsawBlock<'a>),
    #[id(0x2D)]
    ProgramStructureBlock(ProgramStructureBlock<'a>),
    #[id(0x2E)]
    UpdateSign(UpdateSign<'a>),
    #[id(0x2F)]
    SwingArm(SwingArm),
    #[id(0x30)]
    TeleportToEntity(TeleportToEntity),
    #[id(0x31)]
    UseItemOn(UseItemOn),
    #[id(0x32)]
    UseItem(UseItem),
}

#[derive(Decode, Debug, Clone)]
pub struct ConfirmTeleportation {
    pub teleport_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct QueryBlockEntityTag {
    pub transaction_id: VarInt,
    pub location: u64,
}

#[derive(Decode, Debug, Clone)]
pub struct ChangeDifficulty {
    pub difficulty: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct MessageAcknowledgment<'a> {
    pub has_data: bool,
    /// `profile_id` + `last_signature` when `has_data` is set. Kept raw
    /// because the signature is VarInt-length-prefixed, not bool-gated.
    #[decode(rest)]
    pub data: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct ChatCommand<'a> {
    pub command: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    /// VarInt count of `(name, signature)` pairs, then `signed_preview`.
    /// Kept raw because the count is an element count.
    #[decode(rest)]
    pub argument_signatures: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct ChatMessage<'a> {
    pub message: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    /// Signature length + bytes, then `signed_preview` and the seen-message
    /// list. Kept raw because the length prefix is a VarInt, not a bool.
    #[decode(rest)]
    pub signature_and_rest: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct ChatPreview<'a> {
    pub query: i32,
    pub message: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct ClientCommand {
    pub action_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct ClientInformation<'a> {
    pub locale: &'a str,
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
    pub text: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct ClickContainerButton {
    pub window_id: u8,
    pub button_id: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct ClickContainer<'a> {
    pub window_id: u8,
    pub state_id: VarInt,
    pub slot: i16,
    pub button: i8,
    pub mode: VarInt,
    /// VarInt count of `(slot, Slot)` pairs, then the carried Slot.
    #[decode(rest)]
    pub changed_slots_and_carried: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct CloseContainer {
    pub window_id: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct PluginMessage<'a> {
    pub channel: &'a str,
    pub data: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct EditBook<'a> {
    pub slot: VarInt,
    /// VarInt count of pages, then `has_title` and the optional title.
    #[decode(rest)]
    pub entries: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct QueryEntityTag {
    pub transaction_id: VarInt,
    pub entity_id: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct Interact {
    pub entity_id: VarInt,
    pub kind: VarInt,
    pub target_x: Option<f32>,
    pub target_y: Option<f32>,
    pub target_z: Option<f32>,
    pub hand: Option<VarInt>,
    pub sneaking: bool,
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
    pub feet_y: f64,
    pub z: f64,
    pub on_ground: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetPlayerPositionAndRotation {
    pub x: f64,
    pub feet_y: f64,
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
    pub left_paddle_turning: bool,
    pub right_paddle_turning: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct PickItem {
    pub slot: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct PlaceRecipe<'a> {
    pub window_id: u8,
    pub recipe: &'a str,
    pub make_all: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct PlayerAbilities {
    pub flags: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct PlayerAction {
    pub status: VarInt,
    pub location: u64,
    pub face: u8,
    pub sequence: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct PlayerCommand {
    pub entity_id: VarInt,
    pub action_id: VarInt,
    pub jump_boost: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct PlayerInput {
    pub sideways: f32,
    pub forward: f32,
    pub flags: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct Pong {
    pub id: i32,
}

#[derive(Decode, Debug, Clone)]
pub struct ChangeRecipeBookSettings {
    pub book_id: VarInt,
    pub book_open: bool,
    pub filter_active: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetSeenRecipe<'a> {
    pub recipe_id: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct RenameItem<'a> {
    pub item_name: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct ResourcePackResponse {
    pub result: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SeenAdvancements<'a> {
    pub action: VarInt,
    pub tab_id: Option<&'a str>,
}

#[derive(Decode, Debug, Clone)]
pub struct SelectTrade {
    pub selected_slot: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct SetBeaconEffect {
    pub has_primary_effect: bool,
    pub primary_effect: Option<VarInt>,
    pub has_secondary_effect: bool,
    pub secondary_effect: Option<VarInt>,
}

#[derive(Decode, Debug, Clone)]
pub struct SetHeldItem {
    pub slot: i16,
}

#[derive(Decode, Debug, Clone)]
pub struct ProgramCommandBlock<'a> {
    pub location: u64,
    pub command: &'a str,
    pub mode: VarInt,
    pub flags: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct ProgramCommandBlockMinecart<'a> {
    pub entity_id: VarInt,
    pub command: &'a str,
    pub track_output: bool,
}

#[derive(Decode, Debug, Clone)]
pub struct SetCreativeModeSlot<'a> {
    pub slot: i16,
    /// A single Slot: `bool present + VarInt id + i8 count + NBT`.
    #[decode(rest)]
    pub clicked_item: &'a [u8],
}

#[derive(Decode, Debug, Clone)]
pub struct ProgramJigsawBlock<'a> {
    pub location: u64,
    pub name: &'a str,
    pub target: &'a str,
    pub pool: &'a str,
    pub final_state: &'a str,
    pub joint_type: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct ProgramStructureBlock<'a> {
    pub location: u64,
    pub action: VarInt,
    pub mode: VarInt,
    pub name: &'a str,
    pub offset_x: i8,
    pub offset_y: i8,
    pub offset_z: i8,
    pub size_x: i8,
    pub size_y: i8,
    pub size_z: i8,
    pub mirror: VarInt,
    pub rotation: VarInt,
    pub metadata: &'a str,
    pub integrity: f32,
    pub seed: i64,
    pub flags: u8,
}

#[derive(Decode, Debug, Clone)]
pub struct UpdateSign<'a> {
    pub location: u64,
    pub line_1: &'a str,
    pub line_2: &'a str,
    pub line_3: &'a str,
    pub line_4: &'a str,
}

#[derive(Decode, Debug, Clone)]
pub struct SwingArm {
    pub hand: VarInt,
}

#[derive(Decode, Debug, Clone)]
pub struct TeleportToEntity {
    pub target: Uuid,
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
