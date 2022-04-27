// Copyright 2022 Capsule Corp (France) SAS.
// This file is part of Ternoa.

// Ternoa is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Ternoa is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Ternoa.  If not, see <http://www.gnu.org/licenses/>.

use super::mock::*;
use frame_support::{assert_noop, assert_ok, bounded_vec, error::BadOrigin};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use ternoa_common::traits::NFTExt;

use crate::{tests::mock, Error, Event as NFTsEvent, IPFSReference, NFTData, NFTSeriesDetails};

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

mod create {
	use super::*;

	#[test]
	fn create() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let data = NFTData::new_default(ALICE, bounded_vec![1], vec![50]);
			let alice_balance = Balances::free_balance(ALICE);

			// Create NFT with new serie id while there is no series already registered
			let ok = NFT::create(
				alice.clone(),
				data.ipfs_reference.clone(),
				Some(data.series_id.clone()),
			);
			assert_ok!(ok);
			let nft_id = NFT::nft_id_generator() - 1;

			// Final state checks
			let nft_series_details = Some(NFTSeriesDetails::new(ALICE, true));
			assert_eq!(NFT::series_id_generator(), 0);
			assert_eq!(NFT::series(&data.series_id), nft_series_details);
			assert_eq!(NFT::data(nft_id), Some(data.clone()));
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFT::nft_mint_fee());

			// Events checks
			let event = NFTsEvent::NFTCreated {
				nft_id,
				owner: data.owner,
				series_id: data.series_id,
				ipfs_reference: data.ipfs_reference,
				mint_fee: NFT::nft_mint_fee(),
			};
			let event = Event::NFT(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn create_without_series() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let owner = ALICE;
			let ipfs_reference: IPFSReference<Test> = bounded_vec![1];
			let alice_balance = Balances::free_balance(ALICE);

			// Create NFT with new serie id while there is no series already registered
			let ok = NFT::create(origin(ALICE), ipfs_reference.clone(), None);
			assert_ok!(ok);
			let nft_id = NFT::nft_id_generator() - 1;

			// Final state checks
			let data = NFT::data(nft_id);
			assert!(data.is_some());
			assert_eq!(data.as_ref().unwrap().owner, owner);
			assert_eq!(data.as_ref().unwrap().ipfs_reference, ipfs_reference);
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFT::nft_mint_fee());

			// Events checks
			let event = NFTsEvent::NFTCreated {
				nft_id,
				owner: data.as_ref().unwrap().owner,
				series_id: data.as_ref().unwrap().series_id.clone(),
				ipfs_reference: data.as_ref().unwrap().ipfs_reference.clone(),
				mint_fee: NFT::nft_mint_fee(),
			};
			let event = Event::NFT(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 1)]).execute_with(|| {
			// Should fail and storage should remain empty
			let ok = NFT::create(origin(ALICE), bounded_vec![1], None);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn not_the_series_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100), (BOB, 100)]).execute_with(|| {
			let series_id = Some(vec![50]);
			let ok = NFT::create(origin(ALICE), bounded_vec![50], series_id.clone());
			assert_ok!(ok);

			// Should fail and storage should remain empty
			assert_noop!(
				NFT::create(origin(BOB), bounded_vec![1], series_id),
				Error::<Test>::NotTheSeriesOwner
			);
			assert_eq!(Balances::free_balance(BOB), 100);
		})
	}

	#[test]
	fn cannot_create_nft_with_completed_series() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);

			let series_id = Some(vec![51]);
			let ok = NFT::create(alice.clone(), bounded_vec![50], series_id.clone());
			assert_ok!(ok);
			let ok = NFT::finish_series(alice.clone(), series_id.clone().unwrap());
			assert_ok!(ok);

			// Should fail and storage should remain empty
			assert_noop!(
				NFT::create(alice, bounded_vec![1], series_id.clone()),
				Error::<Test>::CannotCreateNFTsWithCompletedSeries
			);
		})
	}
}

mod transfer {
	use super::*;

	#[test]
	fn transfer() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let nft_id = ALICE_NFT_ID;

			let alice: mock::Origin = origin(ALICE);
			let ok = NFT::finish_series(alice.clone(), vec![ALICE_SERIES_ID]);
			assert_ok!(ok);
			let nft = NFT::data(nft_id).unwrap();

			// NFT owner and creator check
			assert_eq!(nft.owner, ALICE);
			assert_eq!(nft.creator, ALICE);

			// Transfer nft ownership from ALICE to BOB
			let ok = NFT::transfer(alice.clone(), nft_id, BOB);
			assert_ok!(ok);

			// Final state checks
			let nft = NFT::data(nft_id).unwrap();
			assert_eq!(nft.owner, BOB);
			assert_eq!(nft.creator, ALICE);

			// Events checks
			let event = NFTsEvent::NFTTransferred { nft_id, old_owner: ALICE, new_owner: BOB };
			let event = Event::NFT(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Try to transfer with an unknown nft id
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::transfer(origin(ALICE), INVALID_NFT_ID, BOB),
				Error::<Test>::NFTNotFound
			);
		})
	}

	#[test]
	fn cannot_transfer_nfts_in_uncompleted_series() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Try to transfer an nft that is part of an uncompleted serie
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::transfer(origin(ALICE), ALICE_NFT_ID, BOB),
				Error::<Test>::CannotTransferNFTsInUncompletedSeries
			);
		})
	}

	#[test]
	fn cannot_transfer_nfts_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_listed_for_sale(ALICE_NFT_ID, true);
			assert_ok!(ok);

			// Try to transfer an nft that is listed for sale
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::transfer(origin(ALICE), ALICE_NFT_ID, BOB),
				Error::<Test>::CannotTransferNFTsListedForSale
			);
		})
	}

	#[test]
	fn cannot_transfer_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_converted_to_capsule(ALICE_NFT_ID, true);
			assert_ok!(ok);

			// Try to transfer an nft that is converted to capsule
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::transfer(origin(ALICE), ALICE_NFT_ID, BOB),
				Error::<Test>::CannotTransferCapsules
			);
		})
	}

	#[test]
	fn cannot_transfer_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_in_transmission(ALICE_NFT_ID, true);
			assert_ok!(ok);

			// Try to transfer an nft that is in transmission
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::transfer(origin(ALICE), ALICE_NFT_ID, BOB),
				Error::<Test>::CannotTransferNFTsInTransmission
			);
		})
	}

	#[test]
	fn cannot_transfer_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_viewer(ALICE_NFT_ID, Some(BOB));
			assert_ok!(ok);

			// Try to transfer a delegated nft
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::transfer(origin(ALICE), ALICE_NFT_ID, BOB),
				Error::<Test>::CannotTransferDelegatedNFTs
			);
		})
	}
}

mod burn {
	use super::*;

	#[test]
	fn burn() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			// NFT check
			assert_eq!(NFT::data(ALICE_NFT_ID).is_some(), true);

			// Burning the nft
			let ok = NFT::burn(origin(ALICE), ALICE_NFT_ID);
			assert_ok!(ok);

			// Final state checks
			assert_eq!(NFT::data(ALICE_NFT_ID).is_some(), false);

			// Events checks
			let event = NFTsEvent::NFTBurned { nft_id: ALICE_NFT_ID };
			let event = Event::NFT(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Try to burn an unknown nft
			// Should fail and storage should remain empty
			assert_noop!(NFT::burn(origin(ALICE), INVALID_NFT_ID), Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100), (BOB, 100)]).execute_with(|| {
			// Try to burn an nft but is not the owner
			// Should fail and storage should remain empty
			assert_noop!(NFT::burn(origin(ALICE), BOB_NFT_ID), Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_burn_nfts_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_listed_for_sale(ALICE_NFT_ID, true);
			assert_ok!(ok);

			// Try to burn an nft that is listed for sale
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::burn(origin(ALICE), ALICE_NFT_ID),
				Error::<Test>::CannotBurnNFTsListedForSale
			);
		})
	}

	#[test]
	fn cannot_burn_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_converted_to_capsule(ALICE_NFT_ID, true);
			assert_ok!(ok);

			// Try to burn an nft that is converted to capsule
			// Should fail and storage should remain empty
			assert_noop!(NFT::burn(origin(ALICE), ALICE_NFT_ID), Error::<Test>::CannotBurnCapsules);
		})
	}

	#[test]
	fn cannot_burn_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_viewer(ALICE_NFT_ID, Some(BOB));
			assert_ok!(ok);

			// Try to burn a delegated nft
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::burn(origin(ALICE), ALICE_NFT_ID),
				Error::<Test>::CannotBurnDelegatedNFTs
			);
		})
	}
}

mod delegate {
	use super::*;

	#[test]
	fn delegate() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let viewer = Some(BOB);
			let mut expected_data = NFT::data(ALICE_NFT_ID).unwrap();
			expected_data.is_delegated = true;

			// Delegating nft to another account
			let ok = NFT::delegate(origin(ALICE), ALICE_NFT_ID, viewer);
			assert_ok!(ok);

			// Final state checks
			assert_eq!(NFT::data(ALICE_NFT_ID), Some(expected_data));
			assert_eq!(NFT::delegated_nfts(ALICE_NFT_ID), Some(BOB));

			// Events checks
			let event = NFTsEvent::NFTDelegated { nft_id: ALICE_NFT_ID, viewer };
			let event = Event::NFT(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn delegate_to_none() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let alice = origin(ALICE);
			let viewer = Some(BOB);

			// Delegating nft to another account
			let ok = NFT::delegate(alice.clone(), ALICE_NFT_ID, viewer);
			assert_ok!(ok);

			// state checks
			assert_eq!(NFT::data(ALICE_NFT_ID).as_ref().unwrap().is_delegated, true);
			assert_eq!(NFT::delegated_nfts(ALICE_NFT_ID), Some(BOB));

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFT(NFTsEvent::NFTDelegated { nft_id: ALICE_NFT_ID, viewer })
			);

			// Delegating nft to None (canceling delegation)
			let ok = NFT::delegate(alice, ALICE_NFT_ID, None);
			assert_ok!(ok);

			// Final state checks
			assert_eq!(NFT::data(ALICE_NFT_ID).as_ref().unwrap().is_delegated, false);
			assert_eq!(NFT::delegated_nfts(ALICE_NFT_ID), None);

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFT(NFTsEvent::NFTDelegated { nft_id: ALICE_NFT_ID, viewer: None })
			);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![]).execute_with(|| {
			// Try to delegate an unknown nft
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::delegate(origin(ALICE), INVALID_NFT_ID, None),
				Error::<Test>::NFTNotFound
			);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Try to delegate an nft but caller is not the owner
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::delegate(origin(BOB), ALICE_NFT_ID, None),
				Error::<Test>::NotTheNFTOwner
			);
		})
	}

	#[test]
	fn cannot_delegate_nfts_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_listed_for_sale(ALICE_NFT_ID, true);
			assert_ok!(ok);

			// Try to delegate an nft that is listed for sale
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::delegate(origin(ALICE), ALICE_NFT_ID, None),
				Error::<Test>::CannotDelegateNFTsListedForSale
			);
		})
	}

	#[test]
	fn cannot_delegate_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_converted_to_capsule(ALICE_NFT_ID, true);
			assert_ok!(ok);

			// Try to delegate an nft that has been converted to capsule
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::delegate(origin(ALICE), ALICE_NFT_ID, None),
				Error::<Test>::CannotDelegateCapsules
			);
		})
	}

	#[test]
	fn cannot_delegate_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let ok = NFT::set_in_transmission(ALICE_NFT_ID, true);
			assert_ok!(ok);

			// Try to delegate an nft that is in transmission
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::delegate(origin(ALICE), ALICE_NFT_ID, None),
				Error::<Test>::CannotDelegateNFTsInTransmission
			);
		})
	}

	#[test]
	fn cannot_delegate_nfts_to_yourself() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Try to delegate an nft to yourself
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::delegate(origin(ALICE), ALICE_NFT_ID, Some(ALICE)),
				Error::<Test>::CannotDelegateNFTsToYourself
			);
		})
	}
}

mod finish_series {
	use super::*;

	#[test]
	fn finish_series() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let series_id = vec![ALICE_SERIES_ID];
			assert_eq!(NFT::series(series_id.clone()).unwrap().draft, true);

			// Finish the serie
			let ok = NFT::finish_series(alice.clone(), series_id.clone());
			assert_ok!(ok);

			// Final state checks
			assert_eq!(NFT::series(series_id.clone()).unwrap().draft, false);

			// Events checks
			let event = NFTsEvent::SeriesFinished { series_id };
			let event = Event::NFT(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn series_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Try to finish unknown serie
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::finish_series(origin(ALICE), vec![123]),
				Error::<Test>::SeriesNotFound
			);
		})
	}

	#[test]
	fn not_the_series_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100), (BOB, 100)]).execute_with(|| {
			// Try to finish serie as no owner
			// Should fail and storage should remain empty
			assert_noop!(
				NFT::finish_series(origin(ALICE), vec![BOB_SERIES_ID]),
				Error::<Test>::NotTheSeriesOwner
			);
		})
	}
}

mod set_nft_mint_fee {
	use super::*;

	#[test]
	fn set_nft_mint_fee() {
		ExtBuilder::new_build(vec![]).execute_with(|| {
			let old_mint_fee = NFT::nft_mint_fee();
			let new_mint_fee = 654u64;
			assert_eq!(NFT::nft_mint_fee(), old_mint_fee);

			// Change the mint fee
			let ok = NFT::set_nft_mint_fee(root(), new_mint_fee);
			assert_ok!(ok);

			// Final state checks
			assert_eq!(NFT::nft_mint_fee(), new_mint_fee);

			// Events checks
			let event = NFTsEvent::NFTMintFeeUpdated { fee: new_mint_fee };
			let event = Event::NFT(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			// Try to change nft mint fee as not root
			// Should fail and storage should remain empty
			assert_noop!(NFT::set_nft_mint_fee(origin(ALICE), 654), BadOrigin);
		})
	}
}