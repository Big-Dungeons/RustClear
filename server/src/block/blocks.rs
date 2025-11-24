#![allow(unused)]

use crate::block::block_parameter::{Axis, BlockColor, ButtonDirection, LeverOrientation, RailShape, StairDirection, TorchDirection, TrapdoorDirection, VineMetadata};
use crate::block::metadata::BlockMetadata;
use crate::types::direction::{Direction, Direction3D};
use macros::blocks;

blocks! {
    #[block_toughness = 0.0]
    Air,
    #[block_toughness = 1.5, tool = Pickaxe]
    Stone {
        variants: {
            Stone,
            Granite,
            PolishedGranite,
            Diorite,
            PolishedDiorite,
            Andesite,
            PolishedAndesite,
        }
    },
    #[block_toughness = 0.6, tool = Shovel]
    Grass,
    #[block_toughness = 0.5, tool = Shovel]
    Dirt {
        variants: {
            Dirt,
            CoarseDirt,
            Podzol,
        }
    },
    #[block_toughness = 2.0, tool = Pickaxe]
    Cobblestone,
    #[block_toughness = 2.0, tool = Axe]
    BlockPlanks {
        variants: {
            OakPlanks,
            SprucePlanks,
            BirchPlanks,
            JunglePlanks,
            AcaciaPlanks,
            DarkOakPlanks,
        }
    },
    #[block_toughness = 0.0]
    BlockSapling {
        variants: {
            OakSapling,
            SpruceSapling,
            BirchSapling,
            JungleSapling,
            AcaciaSapling,
            DarkOakSapling,
        }
    },
    #[block_toughness = -1.0]
    Bedrock,
    #[block_toughness = -1.0]
    FlowingWater {
        level: u8,
    },
    #[block_toughness = -1.0]
    StillWater {
        level: u8,
    },
    #[block_toughness = -1.0]
    FlowingLava {
        level: u8,
    },
    #[block_toughness = -1.0]
    StillLava {
        level: u8,
    },
    #[block_toughness = 0.5, tool = Shovel]
    BlockSand {
        variants: {
            Sand,
            RedSand,
        }
    },
    #[block_toughness = 0.6, tool = Shovel]
    Gravel,
    #[block_toughness = 3.0, tool = Pickaxe]
    GoldOre,
    #[block_toughness = 3.0, tool = Pickaxe]
    IronOre,
    #[block_toughness = 3.0, tool = Pickaxe]
    CoalOre,
    #[block_toughness = 2.0, tool = Axe]
    BlockOldLog {
        variants: {
            OakLog,
            SpruceLog,
            BirchLog,
            JungleLog,
        },
        axis: Axis,
    },
    #[block_toughness = 0.2]
    BlockOldLeaf {
        variants: {
            OakLeaves,
            SpruceLeaves,
            BirchLeaves,
            JungleLeaves,
        },
        check_decay: bool,
        decayable: bool,
    },
    #[block_toughness = 0.6]
    BlockSponge {
        variants: {
            Sponge,
            WetSponge,
        }
    },
    #[block_toughness = 0.3]
    Glass,
    #[block_toughness = 5.0, tool = Pickaxe]
    LapisLazuliOre,
    #[block_toughness = 3.0, tool = Pickaxe]
    LapisLazuliBlock,
    #[block_toughness = 3.5, tool = Pickaxe]
    Dispenser {
        direction: Direction3D,
        triggered: bool,
    },
    #[block_toughness = 0.8, tool = Pickaxe]
    BlockSandStone {
        variants: {
            Sandstone,
            ChiseledSandstone,
            SmoothSandstone,
        }
    },
    #[block_toughness = 0.8]
    Noteblock,
    #[block_toughness = 0.2]
    Bed {
        direction: Direction,
        occupied: bool,
        part: bool,
    },
    #[block_toughness = 0.7, tool = Pickaxe]
    PoweredRail {
        shape: RailShape,
        powered: bool,
    },
    #[block_toughness = 0.7, tool = Pickaxe]
    DetectorRail {
        shape: RailShape,
        powered: bool,
    },
    #[block_toughness = 0.5, tool = Pickaxe]
    StickyPiston {
        direction: Direction3D,
        extended: bool,
    },
    #[block_toughness = 4.0]
    Web,
    #[block_toughness = 0.0]
    BlockTallgrass {
        variants: {
            DeadShrub, // ??
            TallGrass,
            Fern,
        }
    },
    #[block_toughness = 0.0]
    Deadbush,
    #[block_toughness = 0.5, tool = Pickaxe]
    Piston {
        direction: Direction3D,
        extended: bool,
    },
    #[block_toughness = 0.5, tool = Pickaxe]
    PistonHead {
        direction: Direction3D,
        sticky: bool,
    },
    #[block_toughness = 0.8]
    Wool {
        color: BlockColor
    },
    #[block_toughness = 0.5, tool = Pickaxe]
    MovingPiston {
        direction: Direction3D,
        sticky: bool,
    },
    #[block_toughness = 0.0]
    Dandelion,
    #[block_toughness = 0.0]
    BlockRedFlower {
        variants: {
            Poppy,
            BlueOrchid,
            Allium,
            AzureBluet,
            RedTulip,
            OrangeTulip,
            WhiteTulip,
            PinkTulip,
            OxeyeDaisy,
        }
    },
    #[block_toughness = 0.0]
    BrownMushroom,
    #[block_toughness = 0.0]
    RedMushroom,
    #[block_toughness = 3.0, tool = Pickaxe]
    GoldBlock,
    #[block_toughness = 5.0, tool = Pickaxe]
    IronBlock,
    #[block_toughness = 2.0, tool = Pickaxe]
    BlockDoubleSlab {
        variants: {
            DoubleStoneSlab,
            DoubleSandstoneSlab,
            DoubleWoodenSlab,
            DoubleCobblestoneSlab,
            DoubleBrickSlab,
            DoubleStoneBrickSlab,
            DoubleNetherbrickSlab,
            DoubleQuartzSlab,
        },
        seamless: bool,
    },
    #[block_toughness = 2.0, tool = Pickaxe]
    BlockHalfStoneSlab {
        variants: {
            StoneSlab,
            SandstoneSlab,
            WoodenSlab,
            CobblestoneSlab,
            BrickSlab,
            StoneBrickSlab,
            NetherbrickSlab,
            QuartzSlab,
        }
    },
    #[block_toughness = 2.0, tool = Pickaxe]
    Bricks,
    #[block_toughness = 0.0]
    Tnt,
    #[block_toughness = 1.5, tool = Axe]
    Bookshelf,
    #[block_toughness = 2.0, tool = Pickaxe]
    MossyCobblestone,
    #[block_toughness = 50.0, tool = Pickaxe]
    Obsidian,
    #[block_toughness = 0.0]
    Torch {
        direction: TorchDirection
    },
    #[block_toughness = 0.0]
    Fire,
    #[block_toughness = 5.0, tool = Pickaxe]
    MobSpawner,
    #[block_toughness = 2.0, tool = Axe]
    OakStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 2.5]
    Chest {
        direction: Direction3D
    },
    #[block_toughness = 0.0]
    Redstone {
        power: u8
    },
    #[block_toughness = 3.0, tool = Pickaxe]
    DiamondOre,
    #[block_toughness = 5.0, tool = Pickaxe]
    DiamondBlock,
    #[block_toughness = 2.5, tool = Axe]
    CraftingTable,
    #[block_toughness = 0.0]
    Wheat {
        age: u8
    },
    #[block_toughness = 0.6]
    Farmland {
        moisture: u8
    },
    #[block_toughness = 3.5, tool = Pickaxe]
    Furnace {
        facing: Direction3D
    },
    #[block_toughness = 3.5, tool = Pickaxe]
    LitFurnace {
        facing: Direction3D
    },
    #[block_toughness = 1.0, tool = Axe]
    StandingSign {
        rotation: u8
    },
    #[block_toughness = 3.0, tool = Axe]
    WoodenDoor {
        direction: Direction,
        open: bool,
        is_upper: bool,
    },
    #[block_toughness = 0.4, tool = Axe]
    Ladder {
        direction: Direction3D
    },
    #[block_toughness = 0.7, tool = Pickaxe]
    Rail {
        shape: RailShape,
    },
    #[block_toughness = 1.5, tool = Pickaxe]
    StoneStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 1.0, tool = Axe]
    WallSign {
        direction: Direction3D
    },
    #[block_toughness = 0.5]
    Lever {
        orientation: LeverOrientation,
        powered: bool,
    },
    #[block_toughness = 0.5, tool = Pickaxe]
    StonePressurePlate {
        powered: bool,
    },
    #[block_toughness = 5.0, tool = Pickaxe]
    IronDoor {
        direction: Direction,
        open: bool,
        is_upper: bool,
    },
    #[block_toughness = 0.5, tool = Axe]
    WoodenPressurePlate {
        powered: bool,
    },
    #[block_toughness = 3.0, tool = Pickaxe]
    RedstoneOre,
    #[block_toughness = 3.0, tool = Pickaxe]
    LitRedstoneOre,
    #[block_toughness = 0.0]
    UnlitRedstoneTorch {
        direction: TorchDirection,
    },
    #[block_toughness = 0.0]
    RedstoneTorch {
        direction: Direction,
    },
    #[block_toughness = 0.5, tool = Pickaxe]
    StoneButton {
        direction: ButtonDirection,
        powered: bool,
    },
    #[block_toughness = 0.1, tool = Shovel]
    SnowLayer {
        layer_amount: u8,
    },
    #[block_toughness = 0.5, tool = Pickaxe]
    Ice,
    #[block_toughness = 0.2, tool = Shovel]
    Snow,
    #[block_toughness = 0.4]
    Cactus {
        age: u8, // does cactus really need age
    },
    #[block_toughness = 0.6, tool = Shovel]
    Clay,
    #[block_toughness = 0.0]
    SugarCane {
        age: u8,
    },
    #[block_toughness = 2.0, tool = Axe]
    Jukebox {
        has_record: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    Fence,
    #[block_toughness = 1.0, tool = Axe]
    Pumpkin {
        axis: Direction
    },
    #[block_toughness = 0.4, tool = Pickaxe]
    Netherrack,
    #[block_toughness = 0.5, tool = Shovel]
    Soulsand,
    #[block_toughness = 0.3]
    Glowstone,
    #[block_toughness = -1.0]
    NetherPortal {
        axis: Axis
    },
    #[block_toughness = 1.0, tool = Axe]
    LitPumpkin {
        direction: Direction
    },
    #[block_toughness = 0.5]
    Cake {
        bites: u8,
    },
    #[block_toughness = 0.0]
    RedstoneRepeater {
        direction: Direction,
    },
    #[block_toughness = 0.0]
    PoweredRedstoneRepeater {
        direction: Direction,
    },
    #[block_toughness = 0.3]
    StainedGlass {
        color: BlockColor
    },
    #[block_toughness = 3.0, tool = Axe]
    Trapdoor {
        direction: TrapdoorDirection,
        open: bool,
        top_half: bool,
    },
    #[block_toughness = 0.75]
    BlockSilverfish {
        variants: {
            SilverfishStone,
            SilverfishCobblestone,
            SilverfishStoneBrick,
            SilverfishMossyStoneBrick,
            SilverfishCrackedStoneBrick,
            SilverfishChiseledStoneBrick,
        }
    },
    #[block_toughness = 1.5, tool = Pickaxe]
    BlockStoneBrick {
        variants: {
            StoneBricks,
            MossyStoneBricks,
            CrackedStoneBricks,
            ChiseledStoneBricks,
        }
    },
    #[block_toughness = 0.2, tool = Axe]
    BrownMushroomBlock {
        variant: u8,
    },
    #[block_toughness = 0.2, tool = Axe]
    RedMushroomBlock {
        variant: u8,
    },
    #[block_toughness = 5.0, tool = Pickaxe]
    IronBars,
    #[block_toughness = 0.3]
    GlassPane,
    #[block_toughness = 1.0, tool = Axe]
    MelonBlock,
    #[block_toughness = 0.0]
    PumpkinStem {
        age: u8,
    },
    #[block_toughness = 0.0]
    MelonStem {
        age: u8
    },
    #[block_toughness = 0.2]
    Vines {
        metadata: VineMetadata
    },
    #[block_toughness = 2.0, tool = Axe]
    FenceGate {
        direction: Direction,
        open: bool,
        powered: bool,
    },
    #[block_toughness = 2.0, tool = Pickaxe]
    BrickStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 1.5, tool = Pickaxe]
    StoneBrickStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 0.6, tool = Shovel]
    Mycelium,
    #[block_toughness = 0.0]
    Lilypad,
    #[block_toughness = 2.0, tool = Pickaxe]
    Netherbrick,
    #[block_toughness = 2.0, tool = Pickaxe]
    NetherbrickFence,
    #[block_toughness = 2.0, tool = Pickaxe]
    NetherbrickStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 0.0]
    Netherwart {
        age: u8,
    },
    #[block_toughness = 5.0, tool = Pickaxe]
    EnchantingTable,
    #[block_toughness = 0.5, tool = Pickaxe]
    BrewingStand {
        has_bottle0: bool,
        has_bottle1: bool,
        has_bottle2: bool,
    },
    #[block_toughness = 2.0, tool = Pickaxe]
    Cauldron {
        level: u8,
    },
    #[block_toughness = -1.0]
    EndPortal,
    #[block_toughness = -1.0]
    EndPortalFrame {
        direction: Direction,
        has_eye: bool,
    },
    #[block_toughness = 3.0, tool = Pickaxe]
    Endstone,
    #[block_toughness = 3.0]
    DragonEgg,
    #[block_toughness = 0.3]
    RedstoneLamp,
    #[block_toughness = 0.3]
    LitRedstoneLamp,
    #[block_toughness = 2.0, tool = Axe]
    BlockDoubleWoodSlab {
        variants: {
            DoubleOakSlab,
            DoubleSpruceSlab,
            DoubleBirchSlab,
            DoubleJungleSlab,
            DoubleAcaciaSlab,
            DoubleDarkOakSlab,
        }
    },
    #[block_toughness = 2.0, tool = Axe]
    BlockHalfWoodSlab {
        variants: {
            OakSlab,
            SpruceSlab,
            BirchSlab,
            JungleSlab,
            AcaciaSlab,
            DarkOakSlab,
        },
        top_half: bool
    },
    #[block_toughness = 0.2, tool = Axe]
    Cocoa {
        direction: Direction,
    },
    #[block_toughness = 0.8, tool = Pickaxe]
    SandstoneStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 3.0, tool = Pickaxe]
    EmeraldOre,
    #[block_toughness = 22.5, tool = Pickaxe]
    EnderChest {
        direction: Direction3D
    },
    #[block_toughness = 0.0]
    TripwireHook {
        direction: Direction,
        powered: bool,
        attached: bool,
    },
    #[block_toughness = 0.0]
    Tripwire {
        powered: bool,
        suspended: bool,
        attached: bool,
    },
    #[block_toughness = 5.0, tool = Pickaxe]
    EmeraldBlock,
    #[block_toughness = 2.0, tool = Axe]
    SpruceStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    BirchStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    JungleStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = -1.0]
    CommandBlock {
        triggered: bool,
    },
    #[block_toughness = 3.0, tool = Pickaxe]
    Beacon,
    #[block_toughness = 2.0, tool = Pickaxe]
    BlockWall {
        variants: {
            CobblestoneWall,
            MossyCobblestoneWall
        },
    },
    #[block_toughness = 0.0]
    FlowerPot {
        flower: u8,
    },
    #[block_toughness = 0.0]
    Carrots,
    #[block_toughness = 0.0]
    Potatoes,
    #[block_toughness = 0.5, tool = Axe]
    WoodenButton {
        direction: ButtonDirection,
        powered: bool,
    },
    #[block_toughness = 1.0]
    Skull {
        direction: Direction3D,
        no_drop: bool,
    },
    #[block_toughness = 5.0, tool = Pickaxe]
    Anvil {
        direction: Direction,
        // damage: u8
    },
    #[block_toughness = 2.5, tool = Axe]
    TrappedChest {
        direction: Direction3D
    },
    #[block_toughness = 0.5, tool = Pickaxe]
    GoldPressurePlate {
        power: u8,
    },
    #[block_toughness = 0.5, tool = Pickaxe]
    IronPressurePlate {
        power: u8,
    },
    #[block_toughness = 0.0]
    RedstoneComparator {
        direction: Direction,
        mode: bool,
        powered: bool,
    },
    #[block_toughness = 0.0]
    PoweredRedstoneComparator {
        direction: Direction,
        mode: bool,
        powered: bool,
    },
    #[block_toughness = 0.2]
    DaylightSensor {
        power: u8,
    },
    #[block_toughness = 5.0, tool = Pickaxe]
    RedstoneBlock,
    #[block_toughness = 3.0, tool = Pickaxe]
    QuartzOre,
    #[block_toughness = 3.0, tool = Pickaxe]
    Hopper {
        direction: Direction3D,
        enabled: bool,
    },
    #[block_toughness = 0.8, tool = Pickaxe]
    QuartzBlock {
        variants: {
            Quartz,
            ChiseledQuartz,
            PillarQuartz
        }
    },
    #[block_toughness = 0.8, tool = Pickaxe]
    QuartzStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 0.7, tool = Pickaxe]
    ActivatorRail {
        shape: RailShape,
        powered: bool,
    },
    #[block_toughness = 3.5, tool = Pickaxe]
    Dropper {
        direction: Direction3D,
        triggered: bool,
    },
    #[block_toughness = 1.25, tool = Pickaxe]
    StainedHardenedClay {
        color: BlockColor,
    },
    #[block_toughness = 0.3]
    StainedGlassPane {
        color: BlockColor
    },
    #[block_toughness = 0.2]
    BlockNewLeaf {
        variants: {
            AcaciaLeaves,
            DarkOakLeaves,
        },
        decayable: bool,
        check_decay: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    BlockNewLog {
        variants: {
            AcaciaLog,
            DarkOakLog,
        },
        axis: Axis
    },
    #[block_toughness = 2.0, tool = Axe]
    AcaciaStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    DarkOakStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 0.0]
    Slime,
    #[block_toughness = -1.0]
    Barrier,
    #[block_toughness = 5.0, tool = Pickaxe]
    IronTrapdoor {
        direction: TrapdoorDirection,
        open: bool,
        top_half: bool,
    },
    #[block_toughness = 1.5, tool = Pickaxe]
    BlockPrismarine {
        variants: {
            Prismarine,
            PrismarineBricks,
            DarkPrismarine,
        }
    },
    #[block_toughness = 0.3]
    SeaLantern,
    #[block_toughness = 0.5]
    Hay {
        axis: Axis,
    },
    #[block_toughness = 0.1]
    Carpet {
        color: BlockColor
    },
    #[block_toughness = 1.25, tool = Pickaxe]
    HardenedClay,
    #[block_toughness = 5.0, tool = Pickaxe]
    CoalBlock,
    #[block_toughness = 0.5, tool = Pickaxe]
    PackedIce,
    #[block_toughness = 0.0]
    BlockDoublePlant {
        variants: {
            Sunflower,
            Lilac,
            DoubleTallgrass,
            LargeFern,
            RoseBush,
            Peony,
        },
        top_half: bool,
    },
    #[block_toughness = 1.0, tool = Axe]
    StandingBanner {
        rotation: u8,
    },
    #[block_toughness = 1.0, tool = Axe]
    WallBanner {
        direction: Direction3D,
    },
    #[block_toughness = 0.2]
    InvertedDaylightSensor {
        power: u8,
    },
    #[block_toughness = 0.8, tool = Pickaxe]
    RedSandstoneBlock {
        variants: {
            RedSandstone,
            ChiseledRedSandstone,
            SmoothRedSandstone,
        }
    },
    #[block_toughness = 0.8, tool = Pickaxe]
    RedSandstoneStairs {
        direction: StairDirection,
        top_half: bool,
    },
    #[block_toughness = 2.0, tool = Pickaxe]
    DoubleRedSandstoneSlab {
        _placeholder0: bool, // these 3 bits are unused, but so it is encoded properly must be here
        _placeholder1: bool,
        _placeholder2: bool,
        seamless: bool,
    },
    #[block_toughness = 2.0, tool = Pickaxe]
    RedSandstoneSlab {
        _placeholder0: bool,
        _placeholder1: bool,
        _placeholder2: bool,
        top_half: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    SpruceFenceGate {
        direction: Direction,
        open: bool,
        powered: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    BirchFenceGate {
        direction: Direction,
        open: bool,
        powered: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    JungleFenceGate {
        direction: Direction,
        open: bool,
        powered: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    DarkOakFenceGate {
        direction: Direction,
        open: bool,
        powered: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    AcaciaFenceGate {
        direction: Direction,
        open: bool,
        powered: bool,
    },
    #[block_toughness = 2.0, tool = Axe]
    SpruceFence,
    #[block_toughness = 2.0, tool = Axe]
    BirchFence,
    #[block_toughness = 2.0, tool = Axe]
    JungleFence,
    #[block_toughness = 2.0, tool = Axe]
    DarkOakFence,
    #[block_toughness = 2.0, tool = Axe]
    AcaciaFence,
    #[block_toughness = 3.0, tool = Axe]
    SpruceDoor {
        direction: Direction,
        open: bool,
        is_upper: bool,
    },
    #[block_toughness = 3.0, tool = Axe]
    BirchDoor {
        direction: Direction,
        open: bool,
        is_upper: bool,
    },
    #[block_toughness = 3.0, tool = Axe]
    JungleDoor {
        direction: Direction,
        open: bool,
        is_upper: bool,
    },
    #[block_toughness = 3.0, tool = Axe]
    AcaciaDoor {
        direction: Direction,
        open: bool,
        is_upper: bool,
    },
    #[block_toughness = 3.0, tool = Axe]
    DarkOakDoor {
        direction: Direction,
        open: bool,
        is_upper: bool,
    },
}