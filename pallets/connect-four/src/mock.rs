//! Mock library for tests.
use crate::{self as pallet_connect_four, Config, Event as PalletEvent};
use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use frame_support::parameter_types;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		ConnectFour: pallet_connect_four::{Pallet, Call, Storage, Event<T>},
		CollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},

	}
);

impl pallet_randomness_collective_flip::Config for Test {}

impl system::Config for Test {
	type AccountData = ();
	type AccountId = u64;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = ConstU64<250>;
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = Event;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type Origin = Origin;
	type PalletInfo = PalletInfo;
	type SS58Prefix = ConstU16<42>;
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub const PointsForWin: u32 = 5;
	pub const PointsForLoss: u32 = 2;
	pub const PointsForDraw: u32 = 3;
}

impl pallet_connect_four::Config for Test {
	type Event = Event;
	type RandomnessSource = CollectiveFlip;
	type PointsForWin = PointsForWin;
	type PointsForLoss = PointsForLoss;
	type PointsForDraw = PointsForDraw;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities = system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
    ext.execute_with(|| System::set_block_number(1));
	ext
}
