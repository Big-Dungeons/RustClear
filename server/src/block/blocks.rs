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

// block_macro! {

//     // In case something needs to be changed,
//     // each field must be either, u8, bool, or implement BlockMetadata
//     // keep in mind field,
//     // order does matter and the macro needs to generate a function that matches vanilla

//     /// This is an implementation of every block in minecraft 1.8.9. Including their block states.
//     /// Methods are generated with a proc macro.
//     ///
//     /// Implements [From] to get a block from u16.
//     /// You can also get an u16 using get_block_state_id.
//     #[derive(Debug, Eq, PartialEq, Copy, Clone)]
//     pub enum Blocks {
//         Air,
//         Stone {
//             variant: u8,
//         },
//         Grass,
//         Dirt {
//             variant: u8,
//         },
//         Cobblestone,
//         WoodPlank {
//             variant: u8
//         },
//         Sapling {
//             variant: u8
//         },
//         Bedrock,
//         FlowingWater {
//             level: u8
//         },
//         StillWater {
//             level: u8
//         },
//         FlowingLava {
//             level: u8
//         },
//         Lava {
//             level: u8
//         },
//         Sand {
//             variant: u8,
//         },
//         Gravel,
//         GoldOre,
//         IronOre,
//         CoalOre,
//         Log {
//             variant: u2,
//             axis: Axis,
//         },
//         Leaf {
//             variant: u2,
//             check_decay: bool,
//             decayable: bool,
//         },
//         Sponge {
//             wet: bool
//         },
//         Glass,
//         LapisLazuliOre,
//         LapisLazuliBlock,
//         Dispenser {
//             direction: Direction,
//             triggered: bool,
//         },
//         Sandstone {
//             variant: u8
//         },
//         NoteBlock,
//         Bed {
//             direction: HorizontalDirection,
//             occupied: bool,
//             part: bool,
//         },
//         PoweredRail {
//             shape: u3,
//             powered: bool,
//         },
//         DetectorRail {
//             shape: u3,
//             powered: bool,
//         },
//         StickyPiston {
//             direction: Direction,
//             extended: bool,
//         },
//         Web,
//         Tallgrass {
//             variant: u8,
//         },
//         Deadbush,
//         Piston {
//             direction: Direction,
//             extended: bool,
//         },
//         PistonHead {
//             direction: Direction,
//             sticky: bool,
//         },
//         Wool {
//             color: u8
//         },
//         MovingPiston {
//             direction: Direction,
//             sticky: bool,
//         },
//         YellowFlower,
//         RedFlower {
//             variant: u8
//         },
//         BrownMushroom,
//         RedMushroom,
//         GoldBlock,
//         IronBlock,
//         DoubleStoneSlab {
//             variant: u3,
//             seamless: bool,
//         },
//         StoneSlab {
//             variant: u3,
//             top_half: bool,
//         },
//         BrickBlock,
//         Tnt,
//         Bookshelf,
//         MossyCobblestone,
//         Obsidian,
//         Torch {
//             direction: TorchDirection,
//         },
//         Fire,
//         MobSpawner,
//         OakStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         Chest {
//             direction: Direction,
//         },
//         Redstone {
//             power: u8
//         },
//         DiamondOre,
//         DiamondBlock,
//         CraftingTable,
//         Wheat {
//             age: u8
//         },
//         Farmland {
//             moisture: u8
//         },
//         Furnace {
//             facing: Direction
//         },
//         LitFurnace {
//             facing: Direction
//         },
//         StandingSign {
//             rotation: u8
//         },
//         WoodenDoor {
//             direction: HorizontalDirection,
//             open: bool,
//             is_upper: bool,
//         },
//         Ladder {
//             direction: Direction,
//         },
//         Rail {
//             shape: u8
//         },
//         StoneStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         WallSign {
//             direction: Direction,
//         },
//         Lever {
//             orientation: LeverOrientation,
//             powered: bool
//         },
//         StonePressurePlate {
//             powered: bool
//         },
//         IronDoor {
//             direction: HorizontalDirection,
//             open: bool,
//             is_upper: bool,
//         },
//         WoodenPressurePlate {
//             powered: bool
//         },
//         RedstoneOre,
//         LitRedstoneOre,
//         UnlitRedstoneTorch {
//             direction: TorchDirection,
//         },
//         RedstoneTorch {
//             direction: TorchDirection,
//         },
//         StoneButton {
//             direction: ButtonDirection,
//             powered: bool,
//         },
//         SnowLayer {
//             layer_amount: u8
//         },
//         Ice,
//         Snow,
//         Cactus {
//             age: u8
//         },
//         Clay,
//         SugarCane {
//             age: u8,
//         },
//         Jukebox {
//             has_record: bool
//         },
//         Fence,
//         Pumpkin {
//             direction: HorizontalDirection
//         },
//         Netherrack,
//         SoulSand,
//         GlowStone,
//         Portal {
//             axis: Axis
//         },
//         LitPumpkin {
//             direction: HorizontalDirection
//         },
//         Cake {
//             bites: u8
//         },
//         RedstoneRepeater {
//             direction: HorizontalDirection,
//             delay: u2
//         },
//         PoweredRedstoneRepeater {
//             direction: HorizontalDirection,
//             delay: u2
//         },
//         StainedGlass {
//             color: u8
//         },
//         Trapdoor {
//             direction: TrapdoorDirection,
//             open: bool,
//             top_half: bool,
//         },
//         SilverfishBlock {
//             variant: u8
//         },
//         StoneBrick {
//             variant: u8
//         },
//         BrownMushroomBlock {
//             variant: u8
//         },
//         RedMushroomBlock {
//             variant: u8
//         },
//         IronBars,
//         GlassPane,
//         MelonBlock,
//         PumpkinStem {
//             age: u8
//         },
//         MelonStem {
//             age: u8
//         },
//         Vine {
//             metadata: VineMetadata
//         },
//         FenceGate {
//             direction: HorizontalDirection,
//             open: bool,
//             powered: bool,
//         },
//         BrickStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         StoneBrickStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         Mycelium,
//         Lilypad,
//         Netherbrick,
//         NetherbrickFence,
//         NetherbrickStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         Netherwart {
//             age: u8
//         },
//         EnchantingTable,
//         BrewingStand {
//             has_bottle0: bool,
//             has_bottle1: bool,
//             has_bottle2: bool,
//         },
//         Cauldron {
//             level: u8
//         },
//         EndPortal,
//         EndPortalFrame {
//             direction: HorizontalDirection,
//             has_eye: bool
//         },
//         Endstone,
//         DragonEgg,
//         RedstoneLamp,
//         LitRedstoneLamp,
//         DoubleWoodenSlab {
//             variant: u3,
//         },
//         WoodenSlab {
//             variant: u3,
//             top_half: bool
//         },
//         Cocoa {
//             direction: HorizontalDirection,
//             age: u2
//         },
//         SandstoneStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         EmeraldOre,
//         EnderChest {
//             direction: Direction,
//         },
//         TripwireHook {
//             direction: HorizontalDirection,
//             powered: bool,
//             attached: bool,
//         },
//         Tripwire {
//             powered: bool,
//             suspended: bool,
//             attached: bool,
//             disarmed: bool,
//         },
//         EmeraldBlock,
//         SpruceStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         BirchStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         JungleStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         CommandBlock {
//             triggered: bool,
//         },
//         Beacon,
//         CobblestoneWalls {
//             variant: u8
//         },
//         FlowerPot {
//             flower: u8
//         },
//         Carrots,
//         Potatoes,
//         WoodenButton {
//             direction: ButtonDirection,
//             powered: bool,
//         },
//         Skull {
//             direction: Direction,
//             no_drop: bool,
//         },
//         Anvil {
//             direction: HorizontalDirection,
//             damage: u2,
//         },
//         TrappedChest {
//             direction: Direction,
//         },
//         GoldPressurePlate {
//             power: u8
//         },
//         IronPressurePlate {
//             power: u8
//         },
//         RedstoneComparator {
//             direction: HorizontalDirection,
//             mode: bool,
//             powered: bool,
//         },
//         PoweredRedstoneComparator {
//             direction: HorizontalDirection,
//             mode: bool,
//             powered: bool,
//         },
//         DaylightSensor {
//             power: u8
//         },
//         RedstoneBlock,
//         QuartzOre,
//         Hopper {
//             direction: Direction,
//             enabled: bool,
//         },
//         QuartzBlock {
//             variant: u8
//         },
//         QuartzStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         ActivatorRail {
//             shape: u3,
//             powered: bool,
//         },
//         Dropper {
//             direction: Direction,
//             triggered: bool,
//         },
//         StainedHardenedClay {
//             color: u8
//         },
//         StainedGlassPane {
//             color: u8
//         },
//         // i think mojang couldnt fit all in 4 bits
//         NewLeaf {
//             variant: u2,
//             decayable: bool,
//             check_decay: bool,
//         },
//         NewLog {
//             variant: u2,
//             axis: Axis,
//         },
//         AcaciaStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         DarkOakStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         Slime,
//         Barrier,
//         IronTrapdoor {
//             direction: TrapdoorDirection,
//             open: bool,
//             top_half: bool,
//         },
//         Prismarine {
//             variant: u8
//         },
//         SeaLantern,
//         Hay {
//             axis: Axis
//         },
//         Carpet {
//             color: u8
//         },
//         HardenedClay,
//         CoalBlock,
//         PackedIce,
//         DoublePlant {
//             metadata: u8,
//         },
//         StandingBanner {
//             rotation: u8
//         },
//         WallBanner {
//             direction: Direction,
//         },
//         InvertedDaylightSensor {
//             power: u8
//         },
//         RedSandstone {
//             variant: u8
//         },
//         RedSandstoneStairs {
//             direction: StairDirection,
//             top_half: bool,
//         },
//         NewDoubleStoneSlab {
//             variant: u3,
//             seamless: bool,
//         },
//         NewStoneSlab {
//             variant: u3,
//             top_half: bool,
//         },
//         SpruceFenceGate {
//             direction: HorizontalDirection,
//             open: bool,
//             powered: bool,
//         },
//         BirchFenceGate {
//             direction: HorizontalDirection,
//             open: bool,
//             powered: bool,
//         },
//         JungleFenceGate {
//             direction: HorizontalDirection,
//             open: bool,
//             powered: bool,
//         },
//         DarkOakFenceGate {
//             direction: HorizontalDirection,
//             open: bool,
//             powered: bool,
//         },
//         AcaciaFenceGate {
//             direction: HorizontalDirection,
//             open: bool,
//             powered: bool,
//         },
//         SpruceFence,
//         BirchFence,
//         JungleFence,
//         DarkOakFence,
//         AcaicaFence,
//         SpruceDoor {
//             direction: HorizontalDirection,
//             open: bool,
//             is_upper: bool,
//         },
//         BirchDoor {
//             direction: HorizontalDirection,
//             open: bool,
//             is_upper: bool,
//         },
//         JungleDoor {
//             direction: HorizontalDirection,
//             open: bool,
//             is_upper: bool,
//         },
//         AcaicaDoor {
//             direction: HorizontalDirection,
//             open: bool,
//             is_upper: bool,
//         },
//         DarkOakDoor {
//             direction: HorizontalDirection,
//             open: bool,
//             is_upper: bool,
//         },
//     }
// }