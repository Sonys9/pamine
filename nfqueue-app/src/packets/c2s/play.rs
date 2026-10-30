//! Serverbound play packets (protocol 760 / 1.19.2).
//!
//! Packet ids follow the 1.19.2 table. Fields whose layout depends on a
//! preceding discriminator get a hand-written [`Decode`] impl; everything else
//! uses the derive macro.

use pcore::{
    Decode,
    decoder::{Decoder, MAX_ELEMENTS, Slot, Uuid, VarInt},
    errors::Error,
};
use pcore_gen::DecodePacket;

/// Reads an "Optional Byte Array": a VarInt length where `0` means absent.
fn optional_bytes<'a>(decoder: &mut Decoder<'a>) -> Result<Option<&'a [u8]>, Error> {
    let bytes = decoder.slice()?;
    Ok(if bytes.is_empty() { None } else { Some(bytes) })
}

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

#[derive(Debug, Clone)]
pub struct MessageAcknowledgment<'a> {
    pub message_count: VarInt,
    pub has_data: bool,
    /// Only present when `has_data` is set.
    pub profile_id: Option<Uuid>,
    /// Only present when `has_data` is set; `None` when the length is `0`.
    pub last_signature: Option<&'a [u8]>,
}

impl<'a> Decode<'a> for MessageAcknowledgment<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let message_count = decoder.varint()?;
        let has_data = decoder.bool()?;
        let (profile_id, last_signature) = if has_data {
            (Some(decoder.uuid()?), optional_bytes(decoder)?)
        } else {
            (None, None)
        };
        Ok(Self {
            message_count,
            has_data,
            profile_id,
            last_signature,
        })
    }
}

/// One `(argument name, signature)` pair of a signed chat command.
#[derive(Debug, Clone)]
pub struct ArgumentSignature<'a> {
    pub name: &'a str,
    pub signature: &'a [u8],
}

/// A previously seen chat message, referenced by its signature and id.
#[derive(Debug, Clone)]
pub struct LastSeenMessage<'a> {
    pub signature: &'a [u8],
    pub message_id: VarInt,
}

/// The most recently received chat message.
#[derive(Debug, Clone)]
pub struct LastReceived<'a> {
    pub message_id: VarInt,
    pub signature: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct ChatCommand<'a> {
    pub command: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    pub argument_signatures: Vec<ArgumentSignature<'a>>,
    pub signed_preview: bool,
    pub last_seen_messages: Vec<LastSeenMessage<'a>>,
    pub last_received: Option<LastReceived<'a>>,
}

impl<'a> Decode<'a> for ChatCommand<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let command = decoder.string()?;
        let timestamp = decoder.i64()?;
        let salt = decoder.i64()?;
        let argument_signatures = counted(decoder, |d| {
            Ok(ArgumentSignature {
                name: d.string()?,
                signature: d.slice()?,
            })
        })?;
        let signed_preview = decoder.bool()?;
        let last_seen_messages = counted(decoder, |d| {
            Ok(LastSeenMessage {
                signature: d.slice()?,
                message_id: d.varint()?,
            })
        })?;
        let last_received = if decoder.bool()? {
            Some(LastReceived {
                message_id: decoder.varint()?,
                signature: decoder.slice()?,
            })
        } else {
            None
        };
        Ok(Self {
            command,
            timestamp,
            salt,
            argument_signatures,
            signed_preview,
            last_seen_messages,
            last_received,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage<'a> {
    pub message: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    /// `None` when the signature length prefix is `0`.
    pub signature: Option<&'a [u8]>,
    pub signed_preview: bool,
    pub last_seen_messages: Vec<LastSeenMessage<'a>>,
    pub last_received: Option<LastReceived<'a>>,
}

impl<'a> Decode<'a> for ChatMessage<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let message = decoder.string()?;
        let timestamp = decoder.i64()?;
        let salt = decoder.i64()?;
        let signature = optional_bytes(decoder)?;
        let signed_preview = decoder.bool()?;
        let last_seen_messages = counted(decoder, |d| {
            Ok(LastSeenMessage {
                signature: d.slice()?,
                message_id: d.varint()?,
            })
        })?;
        let last_received = if decoder.bool()? {
            Some(LastReceived {
                message_id: decoder.varint()?,
                signature: decoder.slice()?,
            })
        } else {
            None
        };
        Ok(Self {
            message,
            timestamp,
            salt,
            signature,
            signed_preview,
            last_seen_messages,
            last_received,
        })
    }
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

#[derive(Debug, Clone)]
pub struct ClickContainer<'a> {
    pub window_id: u8,
    pub state_id: VarInt,
    pub slot: i16,
    pub button: i8,
    pub mode: VarInt,
    /// `(slot number, Slot)` pairs the client changed.
    pub changed_slots: Vec<(i16, Slot<'a>)>,
    /// The item still held by the cursor after the click.
    pub carried_item: Slot<'a>,
}

impl<'a> Decode<'a> for ClickContainer<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let window_id = decoder.u8()?;
        let state_id = decoder.varint()?;
        let slot = decoder.i16()?;
        let button = decoder.i8()?;
        let mode = decoder.varint()?;

        // The count is a VarInt, and each entry is a bare i16 followed by a Slot.
        let count = decoder.varint()?.get();
        if count < 0 {
            return Err(Error::Read(0));
        }
        let mut changed_slots = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let index = decoder.i16()?;
            changed_slots.push((index, Slot::decode(decoder)?));
        }
        let carried_item = Slot::decode(decoder)?;

        Ok(Self {
            window_id,
            state_id,
            slot,
            button,
            mode,
            changed_slots,
            carried_item,
        })
    }
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

#[derive(Debug, Clone)]
pub struct EditBook<'a> {
    pub slot: VarInt,
    pub pages: Vec<&'a str>,
    /// `None` when the client cleared the title.
    pub title: Option<&'a str>,
}

impl<'a> Decode<'a> for EditBook<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self, Error> {
        let slot = decoder.varint()?;
        let pages = counted(decoder, |d| d.string())?;
        let title = if decoder.bool()? {
            Some(decoder.string()?)
        } else {
            None
        };
        Ok(Self { slot, pages, title })
    }
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
    pub clicked_item: Slot<'a>,
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
