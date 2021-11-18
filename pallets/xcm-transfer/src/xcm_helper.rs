pub use self::xcm_helper::*;

pub mod xcm_helper {
	use crate::common::XTransferAsset;
	use cumulus_primitives_core::ParaId;
	use frame_support::pallet_prelude::*;
	use sp_runtime::traits::CheckedConversion;
	use sp_std::{
		convert::{Into, TryFrom, TryInto},
		marker::PhantomData,
		result,
	};
	use xcm::latest::{
		prelude::*, AssetId::Concrete, Fungibility::Fungible, MultiAsset, MultiLocation,
	};
	use xcm_executor::traits::{
		Error as XcmError, FilterAssetLocation, MatchesFungible, MatchesFungibles,
	};

	const LOG_TARGET: &str = "xcm-helper";

	pub struct NativeAssetMatcher<C>(PhantomData<C>);
	impl<C: NativeAssetChecker, B: TryFrom<u128>> MatchesFungible<B> for NativeAssetMatcher<C> {
		fn matches_fungible(a: &MultiAsset) -> Option<B> {
			log::trace!(
				target: LOG_TARGET,
				"NativeAssetMatcher check fungible {:?}.",
				a.clone(),
			);
			match (&a.id, &a.fun) {
				(Concrete(_), Fungible(ref amount)) if C::is_native_asset(a) => {
					CheckedConversion::checked_from(*amount)
				}
				_ => None,
			}
		}
	}

	pub struct ConcreteAssetsMatcher<AssetId, Balance>(PhantomData<(AssetId, Balance)>);
	impl<AssetId: Clone + From<XTransferAsset>, Balance: Clone + From<u128>>
		MatchesFungibles<AssetId, Balance> for ConcreteAssetsMatcher<AssetId, Balance>
	{
		fn matches_fungibles(a: &MultiAsset) -> result::Result<(AssetId, Balance), XcmError> {
			log::trace!(
				target: LOG_TARGET,
				"ConcreteAssetsMatcher check fungible {:?}.",
				a.clone(),
			);
			let (&amount, location) = match (&a.fun, &a.id) {
				(Fungible(ref amount), Concrete(ref id)) => (amount, id),
				_ => return Err(XcmError::AssetNotFound),
			};
			let xtransfer_asset: XTransferAsset = location
			.clone()
			.try_into()
			.map_err(|_| XcmError::AssetIdConversionFailed)?;

			let amount = amount
				.try_into()
				.map_err(|_| XcmError::AmountToBalanceConversionFailed)?;
			Ok((xtransfer_asset.into(), amount))
		}
	}

	// We only trust the origin to send us assets that they identify as their
	// sovereign assets.
	pub struct AssetOriginFilter;
	impl FilterAssetLocation for AssetOriginFilter {
		fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
			if let Some(ref id) = ConcrateAsset::origin(asset) {
				if id == origin {
					return true;
				}
			}
			false
		}
	}

	pub struct NativeAssetFilter<T>(PhantomData<T>);
	impl<T: Get<ParaId>> NativeAssetFilter<T> {
		pub fn is_native_asset_id(id: &MultiLocation) -> bool {
			let native_locations = [
				MultiLocation::here(),
				(1, X1(Parachain(T::get().into()))).into(),
			];
			native_locations.contains(id)
		}
	}

	pub trait NativeAssetChecker {
		fn is_native_asset(asset: &MultiAsset) -> bool;
	}
	impl<T: Get<ParaId>> NativeAssetChecker for NativeAssetFilter<T> {
		fn is_native_asset(asset: &MultiAsset) -> bool {
			match (&asset.id, &asset.fun) {
				// So far our native asset is concrete
				(Concrete(ref id), Fungible(_)) if Self::is_native_asset_id(id) => true,
				_ => false,
			}
		}
	}

	pub struct ConcrateAsset;
	impl ConcrateAsset {
		pub fn id(asset: &MultiAsset) -> Option<MultiLocation> {
			match (&asset.id, &asset.fun) {
				// So far our native asset is concrete
				(Concrete(id), Fungible(_)) => Some(id.clone()),
				_ => None,
			}
		}

		pub fn origin(asset: &MultiAsset) -> Option<MultiLocation> {
			Self::id(asset).and_then(|id| {
				match (id.parents, id.first_interior()) {
					// sibling parachain
					(1, Some(Parachain(id))) => Some(MultiLocation::new(1, X1(Parachain(*id)))),
					// parent
					(1, _) => Some(MultiLocation::parent()),
					// children parachain
					(0, Some(Parachain(id))) => Some(MultiLocation::new(0, X1(Parachain(*id)))),
					// local: (0, Here)
					(0, None) => Some(id),
					_ => None,
				}
			})
		}
	}
}
