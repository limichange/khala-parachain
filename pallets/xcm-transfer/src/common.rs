pub use self::common::*;

pub mod common {
	use codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use sp_std::{
		convert::{From, TryFrom, TryInto},
		result,
	};
	use xcm::latest::MultiLocation;

	#[derive(Clone, Decode, Encode, Eq, PartialEq, Ord, PartialOrd, Debug, TypeInfo)]
	pub enum XTransferAsset {
		ParachainAsset(MultiLocation),
		SolochainAsset([u8; 32]),
	}

	impl TryFrom<MultiLocation> for XTransferAsset {
		type Error = ();
		fn try_from(x: MultiLocation) -> result::Result<Self, ()> {
			Ok(XTransferAsset::ParachainAsset(x))
		}
	}

	impl TryFrom<XTransferAsset> for MultiLocation {
		type Error = ();
		fn try_from(x: XTransferAsset) -> result::Result<Self, ()> {
			match x {
				XTransferAsset::ParachainAsset(location) => Ok(location),
				_ => Err(()),
			}
		}
	}

	impl TryFrom<[u8; 32]> for XTransferAsset {
		type Error = ();
		fn try_from(x: [u8; 32]) -> result::Result<Self, ()> {
			Ok(XTransferAsset::SolochainAsset(x))
		}
	}

	pub type XTransferAssetId = u128;
	impl From<XTransferAsset> for XTransferAssetId {
		fn from(x: XTransferAsset) -> Self {
			match x {
				XTransferAsset::ParachainAsset(location) => u128::from_le_bytes(
					crate::hashing::blake2_256(&location.encode())
						.split_at(16)
						.1
						.try_into()
						.unwrap(),
				),
				XTransferAsset::SolochainAsset(v) => {
					u128::from_le_bytes(v.split_at(16).1.try_into().unwrap())
				}
			}
		}
	}
}
