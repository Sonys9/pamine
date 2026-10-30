//! Serverbound play packets (protocol 760 / 1.19.2).
//!
//! Packet ids follow the 1.19.2 table. The packet bodies are grouped into
//! submodules by subject; this module only holds the [`Play`] dispatch enum and
//! re-exports every body so the variants can name them directly.

pub mod block;
pub mod chat;
pub mod client;
pub mod container;
pub mod misc;
pub mod movement;

pub use block::*;
pub use chat::*;
pub use client::*;
pub use container::*;
pub use misc::*;
pub use movement::*;

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
