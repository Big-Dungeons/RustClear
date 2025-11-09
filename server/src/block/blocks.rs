#![allow(unused)]

use crate::block::block_parameter::{Axis, ButtonDirection, HorizontalDirection, LeverOrientation, StairDirection, TorchDirection, TrapdoorDirection, VineMetadata};
use crate::block::metadata::{u2, u3};
use crate::types::direction::Direction;
use macros::block_macro;

block_macro! {

    // In case something needs to be changed,
    // each field must be either, u8, bool, or implement BlockMetadata
    // keep in mind field,
    // order does matter and the macro needs to generate a function that matches vanilla

    /// This is an implementation of every block in minecraft 1.8.9. Including their block states.
    /// Methods are generated with a proc macro.
    ///
    /// Implements [From] to get a block from u16.
    /// You can also get an u16 using get_block_state_id.
    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum Blocks {
        Air,
        Stone {
            variant: u8,
        },
        Grass,
        Dirt {
            variant: u8,
        },
        Cobblestone,
        WoodPlank {
            variant: u8
        },
        Sapling {
            variant: u8
        },
        Bedrock,
        FlowingWater {
            level: u8
        },
        StillWater {
            level: u8
        },
        FlowingLava {
            level: u8
        },
        Lava {
            level: u8
        },
        Sand {
            variant: u8,
        },
        Gravel,
        GoldOre,
        IronOre,
        CoalOre,
        Log {
            variant: u2,
            axis: Axis,
        },
        Leaf {
            variant: u2,
            check_decay: bool,
            decayable: bool,
        },
        Sponge {
            wet: bool
        },
        Glass,
        LapisLazuliOre,
        LapisLazuliBlock,
        Dispenser {
            direction: Direction,
            triggered: bool,
        },
        Sandstone {
            variant: u8
        },
        NoteBlock,
        Bed {
            direction: HorizontalDirection,
            occupied: bool,
            part: bool,
        },
        PoweredRail {
            shape: u3,
            powered: bool,
        },
        DetectorRail {
            shape: u3,
            powered: bool,
        },
        StickyPiston {
            direction: Direction,
            extended: bool,
        },
        Web,
        Tallgrass {
            variant: u8,
        },
        Deadbush,
        Piston {
            direction: Direction,
            extended: bool,
        },
        PistonHead {
            direction: Direction,
            sticky: bool,
        },
        Wool {
            color: u8
        },
        MovingPiston {
            direction: Direction,
            sticky: bool,
        },
        YellowFlower,
        RedFlower {
            variant: u8
        },
        BrownMushroom,
        RedMushroom,
        GoldBlock,
        IronBlock,
        DoubleStoneSlab {
            variant: u3,
            seamless: bool,
        },
        StoneSlab {
            variant: u3,
            top_half: bool,
        },
        BrickBlock,
        Tnt,
        Bookshelf,
        MossyCobblestone,
        Obsidian,
        Torch {
            direction: TorchDirection,
        },
        Fire,
        MobSpawner,
        OakStairs {
            direction: StairDirection,
            top_half: bool,
        },
        Chest {
            direction: Direction,
        },
        Redstone {
            power: u8
        },
        DiamondOre,
        DiamondBlock,
        CraftingTable,
        Wheat {
            age: u8
        },
        Farmland {
            moisture: u8
        },
        Furnace {
            facing: Direction
        },
        LitFurnace {
            facing: Direction
        },
        StandingSign {
            rotation: u8
        },
        WoodenDoor {
            direction: HorizontalDirection,
            open: bool,
            is_upper: bool,
        },
        Ladder {
            direction: Direction,
        },
        Rail {
            shape: u8
        },
        StoneStairs {
            direction: StairDirection,
            top_half: bool,
        },
        WallSign {
            direction: Direction,
        },
        Lever {
            orientation: LeverOrientation,
            powered: bool
        },
        StonePressurePlate {
            powered: bool
        },
        IronDoor {
            direction: HorizontalDirection,
            open: bool,
            is_upper: bool,
        },
        WoodenPressurePlate {
            powered: bool
        },
        RedstoneOre,
        LitRedstoneOre,
        UnlitRedstoneTorch {
            direction: TorchDirection,
        },
        RedstoneTorch {
            direction: TorchDirection,
        },
        StoneButton {
            direction: ButtonDirection,
            powered: bool,
        },
        SnowLayer {
            layer_amount: u8
        },
        Ice,
        Snow,
        Cactus {
            age: u8
        },
        Clay,
        SugarCane {
            age: u8,
        },
        Jukebox {
            has_record: bool
        },
        Fence,
        Pumpkin {
            direction: HorizontalDirection
        },
        Netherrack,
        SoulSand,
        GlowStone,
        Portal {
            axis: Axis
        },
        LitPumpkin {
            direction: HorizontalDirection
        },
        Cake {
            bites: u8
        },
        RedstoneRepeater {
            direction: HorizontalDirection,
            delay: u2
        },
        PoweredRedstoneRepeater {
            direction: HorizontalDirection,
            delay: u2
        },
        StainedGlass {
            color: u8
        },
        Trapdoor {
            direction: TrapdoorDirection,
            open: bool,
            top_half: bool,
        },
        SilverfishBlock {
            variant: u8
        },
        StoneBrick {
            variant: u8
        },
        BrownMushroomBlock {
            variant: u8
        },
        RedMushroomBlock {
            variant: u8
        },
        IronBars,
        GlassPane,
        MelonBlock,
        PumpkinStem {
            age: u8
        },
        MelonStem {
            age: u8
        },
        Vine {
            metadata: VineMetadata
        },
        FenceGate {
            direction: HorizontalDirection,
            open: bool,
            powered: bool,
        },
        BrickStairs {
            direction: StairDirection,
            top_half: bool,
        },
        StoneBrickStairs {
            direction: StairDirection,
            top_half: bool,
        },
        Mycelium,
        Lilypad,
        Netherbrick,
        NetherbrickFence,
        NetherbrickStairs {
            direction: StairDirection,
            top_half: bool,
        },
        Netherwart {
            age: u8
        },
        EnchantingTable,
        BrewingStand {
            has_bottle0: bool,
            has_bottle1: bool,
            has_bottle2: bool,
        },
        Cauldron {
            level: u8
        },
        EndPortal,
        EndPortalFrame {
            direction: HorizontalDirection,
            has_eye: bool
        },
        Endstone,
        DragonEgg,
        RedstoneLamp,
        LitRedstoneLamp,
        DoubleWoodenSlab {
            variant: u3,
        },
        WoodenSlab {
            variant: u3,
            top_half: bool
        },
        Cocoa {
            direction: HorizontalDirection,
            age: u2
        },
        SandstoneStairs {
            direction: StairDirection,
            top_half: bool,
        },
        EmeraldOre,
        EnderChest {
            direction: Direction,
        },
        TripwireHook {
            direction: HorizontalDirection,
            powered: bool,
            attached: bool,
        },
        Tripwire {
            powered: bool,
            suspended: bool,
            attached: bool,
            disarmed: bool,
        },
        EmeraldBlock,
        SpruceStairs {
            direction: StairDirection,
            top_half: bool,
        },
        BirchStairs {
            direction: StairDirection,
            top_half: bool,
        },
        JungleStairs {
            direction: StairDirection,
            top_half: bool,
        },
        CommandBlock {
            triggered: bool,
        },
        Beacon,
        CobblestoneWalls {
            variant: u8
        },
        FlowerPot {
            flower: u8
        },
        Carrots,
        Potatoes,
        WoodenButton {
            direction: ButtonDirection,
            powered: bool,
        },
        Skull {
            direction: Direction,
            no_drop: bool,
        },
        Anvil {
            direction: HorizontalDirection,
            damage: u2,
        },
        TrappedChest {
            direction: Direction,
        },
        GoldPressurePlate {
            power: u8
        },
        IronPressurePlate {
            power: u8
        },
        RedstoneComparator {
            direction: HorizontalDirection,
            mode: bool,
            powered: bool,
        },
        PoweredRedstoneComparator {
            direction: HorizontalDirection,
            mode: bool,
            powered: bool,
        },
        DaylightSensor {
            power: u8
        },
        RedstoneBlock,
        QuartzOre,
        Hopper {
            direction: Direction,
            enabled: bool,
        },
        QuartzBlock {
            variant: u8
        },
        QuartzStairs {
            direction: StairDirection,
            top_half: bool,
        },
        ActivatorRail {
            shape: u3,
            powered: bool,
        },
        Dropper {
            direction: Direction,
            triggered: bool,
        },
        StainedHardenedClay {
            color: u8
        },
        StainedGlassPane {
            color: u8
        },
        // i think mojang couldnt fit all in 4 bits
        NewLeaf {
            variant: u2,
            decayable: bool,
            check_decay: bool,
        },
        NewLog {
            variant: u2,
            axis: Axis,
        },
        AcaciaStairs {
            direction: StairDirection,
            top_half: bool,
        },
        DarkOakStairs {
            direction: StairDirection,
            top_half: bool,
        },
        Slime,
        Barrier,
        IronTrapdoor {
            direction: TrapdoorDirection,
            open: bool,
            top_half: bool,
        },
        Prismarine {
            variant: u8
        },
        SeaLantern,
        Hay {
            axis: Axis
        },
        Carpet {
            color: u8
        },
        HardenedClay,
        CoalBlock,
        PackedIce,
        DoublePlant {
            metadata: u8,
        },
        StandingBanner {
            rotation: u8
        },
        WallBanner {
            direction: Direction,
        },
        InvertedDaylightSensor {
            power: u8
        },
        RedSandstone {
            variant: u8
        },
        RedSandstoneStairs {
            direction: StairDirection,
            top_half: bool,
        },
        NewDoubleStoneSlab {
            variant: u3,
            seamless: bool,
        },
        NewStoneSlab {
            variant: u3,
            top_half: bool,
        },
        SpruceFenceGate {
            direction: HorizontalDirection,
            open: bool,
            powered: bool,
        },
        BirchFenceGate {
            direction: HorizontalDirection,
            open: bool,
            powered: bool,
        },
        JungleFenceGate {
            direction: HorizontalDirection,
            open: bool,
            powered: bool,
        },
        DarkOakFenceGate {
            direction: HorizontalDirection,
            open: bool,
            powered: bool,
        },
        AcaciaFenceGate {
            direction: HorizontalDirection,
            open: bool,
            powered: bool,
        },
        SpruceFence,
        BirchFence,
        JungleFence,
        DarkOakFence,
        AcaicaFence,
        SpruceDoor {
            direction: HorizontalDirection,
            open: bool,
            is_upper: bool,
        },
        BirchDoor {
            direction: HorizontalDirection,
            open: bool,
            is_upper: bool,
        },
        JungleDoor {
            direction: HorizontalDirection,
            open: bool,
            is_upper: bool,
        },
        AcaicaDoor {
            direction: HorizontalDirection,
            open: bool,
            is_upper: bool,
        },
        DarkOakDoor {
            direction: HorizontalDirection,
            open: bool,
            is_upper: bool,
        },
    }
}