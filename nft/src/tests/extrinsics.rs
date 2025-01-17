// Copyright 2023 Capsule Corp (France) SAS.
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
use crate::{tests::mock, Collection, CollectionId, Error, Event as NFTsEvent, NFTData, NFTId};
use frame_support::{assert_noop, assert_ok, error::BadOrigin, BoundedVec};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use primitives::{nfts::NFTState, tee::ClusterId};
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::NFTExt;

const ALICE_NFT_ID: NFTId = 0;
const BOB_NFT_ID: NFTId = 1;
const ALICE_COLLECTION_ID: CollectionId = 0;
const BOB_COLLECTION_ID: CollectionId = 1;
const INVALID_ID: NFTId = 1001;
const PERCENT_100: Permill = Permill::from_parts(1000000);
const PERCENT_80: Permill = Permill::from_parts(800000);
const PERCENT_0: Permill = Permill::from_parts(0);

fn origin(account: u64) -> mock::RuntimeOrigin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::RuntimeOrigin {
	RawOrigin::Root.into()
}

fn prepare_tests() {
	let alice: mock::RuntimeOrigin = origin(ALICE);
	let bob: mock::RuntimeOrigin = origin(BOB);

	//Create alice NFT.
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_100, None, false).unwrap();

	// Create alice collection.
	NFT::create_collection(alice, BoundedVec::default(), None).unwrap();

	//Create bob NFT.
	NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_100, None, false).unwrap();

	// Create bob collection.
	NFT::create_collection(bob, BoundedVec::default(), None).unwrap();

	assert_eq!(NFT::nfts(ALICE_NFT_ID).is_some(), true);
	assert_eq!(NFT::nfts(BOB_NFT_ID).is_some(), true);

	assert_eq!(NFT::collections(ALICE_COLLECTION_ID).is_some(), true);
	assert_eq!(NFT::collections(BOB_COLLECTION_ID).is_some(), true);
}

fn prepare_tee_for_tests() {
	let alice: mock::RuntimeOrigin = origin(ALICE);
	let bob: mock::RuntimeOrigin = origin(BOB);
	let charlie: mock::RuntimeOrigin = origin(CHARLIE);

	assert_ok!(TEE::register_enclave(alice.clone(), ALICE_ENCLAVE, BoundedVec::default()));
	assert_ok!(TEE::register_enclave(bob.clone(), BOB_ENCLAVE, BoundedVec::default()));
	assert_ok!(TEE::register_enclave(charlie.clone(), CHARLIE_ENCLAVE, BoundedVec::default()));

	let cluster_id: ClusterId = 0;
	let second_cluster_id: ClusterId = 1;
	assert_ok!(TEE::create_cluster(root()));
	assert_ok!(TEE::create_cluster(root()));

	assert_ok!(TEE::assign_enclave(root(), ALICE, cluster_id));
	assert_ok!(TEE::assign_enclave(root(), BOB, cluster_id));
	assert_ok!(TEE::assign_enclave(root(), CHARLIE, second_cluster_id));
}

mod create_nft {
	use super::*;

	#[test]
	fn create_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);
			let data = NFTData::new_default(ALICE, BoundedVec::default(), PERCENT_100, None, false);

			// Create NFT without a collection.
			NFT::create_nft(
				alice,
				data.offchain_data.clone(),
				data.royalty,
				data.collection_id,
				data.state.is_soulbound,
			)
			.unwrap();
			let nft_id = NFT::get_next_nft_id() - 1;

			// Final state checks.
			let nft = NFT::nfts(nft_id);
			assert_eq!(nft, Some(data.clone()));
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFT::nft_mint_fee());

			// Events checks.
			let event = NFTsEvent::NFTCreated {
				nft_id,
				owner: data.owner,
				offchain_data: data.offchain_data,
				royalty: data.royalty,
				collection_id: data.collection_id,
				is_soulbound: data.state.is_soulbound,
				mint_fee: NFT::nft_mint_fee(),
			};
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn create_nft_with_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);
			let data = NFTData::new_default(
				ALICE,
				BoundedVec::default(),
				PERCENT_100,
				Some(ALICE_COLLECTION_ID),
				false,
			);

			// Create NFT with a collection.
			NFT::create_nft(
				alice,
				data.offchain_data.clone(),
				data.royalty,
				data.collection_id,
				data.state.is_soulbound,
			)
			.unwrap();
			let nft_id = NFT::get_next_nft_id() - 1;

			// Final state checks.
			let nft = NFT::nfts(nft_id);
			assert_eq!(nft, Some(data.clone()));
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFT::nft_mint_fee());
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).unwrap().nfts.contains(&nft_id), true);

			// Events checks.
			let event = NFTsEvent::NFTCreated {
				nft_id,
				owner: data.owner,
				offchain_data: data.offchain_data,
				royalty: data.royalty,
				collection_id: data.collection_id,
				is_soulbound: data.state.is_soulbound,
				mint_fee: NFT::nft_mint_fee(),
			};
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 1)]).execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Should fail and storage should remain empty.
			let err = NFT::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false);
			assert_noop!(err, BalanceError::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to add Alice's NFT to Bob's collection.
			let err = NFT::create_nft(
				alice,
				BoundedVec::default(),
				PERCENT_0,
				Some(BOB_COLLECTION_ID),
				false,
			);

			// Should fail because Bob is not the collection owner.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}

	#[test]
	fn collection_is_closed() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Close alice's collection.
			NFT::close_collection(alice.clone(), ALICE_COLLECTION_ID).unwrap();

			// Add an NFT to this collection.
			let err = NFT::create_nft(
				alice,
				BoundedVec::default(),
				PERCENT_0,
				Some(ALICE_COLLECTION_ID),
				false,
			);

			// Should fail because collection is close.
			assert_noop!(err, Error::<Test>::CollectionIsClosed);
		})
	}

	#[test]
	fn collection_has_reached_max() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Add CollectionSizeLimit NFTs to Alice's collection.
			for _i in 0..CollectionSizeLimit::get() {
				NFT::create_nft(
					alice.clone(),
					BoundedVec::default(),
					PERCENT_0,
					Some(ALICE_COLLECTION_ID),
					false,
				)
				.unwrap();
			}

			// Add another nft to the collection.
			let err = NFT::create_nft(
				alice,
				BoundedVec::default(),
				PERCENT_0,
				Some(ALICE_COLLECTION_ID),
				false,
			);

			// Should fail because collection has reached maximum value.
			assert_noop!(err, Error::<Test>::CollectionHasReachedLimit);
		})
	}

	#[test]
	fn collection_has_reached_limit() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Create a collection with 1 as limit.
			NFT::create_collection(alice.clone(), BoundedVec::default(), Some(1)).unwrap();
			let collection_id = NFT::get_next_collection_id() - 1;

			// Add nft to the collection.
			NFT::create_nft(
				alice.clone(),
				BoundedVec::default(),
				PERCENT_0,
				Some(collection_id),
				false,
			)
			.unwrap();

			// Adding another nft to the collection.
			let err = NFT::create_nft(
				alice,
				BoundedVec::default(),
				PERCENT_0,
				Some(collection_id),
				false,
			);
			// Should fail because collection has reached limit.
			assert_noop!(err, Error::<Test>::CollectionHasReachedLimit);
		})
	}

	#[test]
	fn keep_alive() {
		ExtBuilder::new_build(vec![(ALICE, 2 * NFT_MINT_FEE), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);

			// Try to create an NFT.
			let err = NFT::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false);

			// Should fail because Alice's account must stay alive.
			assert_noop!(err, BalanceError::<Test>::KeepAlive);
			// Alice's balance should not have been changed
			assert_eq!(Balances::free_balance(ALICE), alice_balance);
		})
	}
}

mod burn_nft {

	use super::*;

	#[test]
	fn burn_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Burning the nft.
			let ok = NFT::burn_nft(alice, ALICE_NFT_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID).is_some(), false);

			// Events checks.
			let event = NFTsEvent::NFTBurned { nft_id: ALICE_NFT_ID };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn burn_nft_in_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let expected_collection = NFT::collections(ALICE_COLLECTION_ID).unwrap();
			// Add alice's NFT to her collection.
			NFT::add_nft_to_collection(alice.clone(), ALICE_NFT_ID, ALICE_COLLECTION_ID).unwrap();
			// Burning the nft.
			let ok = NFT::burn_nft(alice, ALICE_NFT_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID).is_some(), false);
			assert_eq!(
				NFT::collections(ALICE_COLLECTION_ID).unwrap().nfts,
				expected_collection.nfts
			);

			// Events checks.
			let event = NFTsEvent::NFTBurned { nft_id: ALICE_NFT_ID };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn burn_synced_secret_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Add a secret to Alice's NFT.
			NFT::add_secret(alice.clone(), ALICE_NFT_ID, offchain_data.clone()).unwrap();

			// Set listed to true for Alice's NFT.
			let nft_state =
				NFTState::new(false, false, true, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			assert_eq!(NFT::secret_nfts_offchain_data(ALICE_NFT_ID).unwrap(), offchain_data);

			// Burning the nft.
			let ok = NFT::burn_nft(alice, ALICE_NFT_ID);
			assert_ok!(ok);

			// Final state checks.
			assert!(NFT::nfts(ALICE_NFT_ID).is_none());
			assert!(NFT::secret_nfts_offchain_data(ALICE_NFT_ID).is_none());

			// Events checks.
			let event = NFTsEvent::NFTBurned { nft_id: ALICE_NFT_ID };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn burn_syncing_secret_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
				let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
				// Add a secret to Alice's NFT.
				NFT::add_secret(alice.clone(), ALICE_NFT_ID, offchain_data.clone()).unwrap();

				NFT::add_secret_shard(alice_enclave, ALICE_NFT_ID).unwrap();

				assert_eq!(NFT::secret_nfts_offchain_data(ALICE_NFT_ID).unwrap(), offchain_data);
				assert_eq!(NFT::secret_nfts_shards_count(ALICE_NFT_ID).unwrap().len(), 1);

				// Burning the nft.
				let ok = NFT::burn_nft(alice, ALICE_NFT_ID);
				assert_ok!(ok);

				// Final state checks.
				assert!(NFT::nfts(ALICE_NFT_ID).is_none());
				assert!(NFT::secret_nfts_offchain_data(ALICE_NFT_ID).is_none());
				assert!(NFT::secret_nfts_shards_count(ALICE_NFT_ID).is_none());

				// Events checks.
				let event = NFTsEvent::NFTBurned { nft_id: ALICE_NFT_ID };
				let event = RuntimeEvent::NFT(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn burn_synced_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Conver Alice's NFT to Capsule.
			NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, offchain_data.clone()).unwrap();

			// Set listed to true for Alice's NFT.
			let nft_state =
				NFTState::new(true, false, false, false, false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			assert_eq!(NFT::capsule_offchain_data(ALICE_NFT_ID).unwrap(), offchain_data);

			// Burning the nft.
			let ok = NFT::burn_nft(alice, ALICE_NFT_ID);
			assert_ok!(ok);

			// Final state checks.
			assert!(NFT::nfts(ALICE_NFT_ID).is_none());
			assert!(NFT::capsule_offchain_data(ALICE_NFT_ID).is_none());

			// Events checks.
			let event = NFTsEvent::NFTBurned { nft_id: ALICE_NFT_ID };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn burn_syncing_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
				let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
				// Add a secret to Alice's NFT.
				NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, offchain_data.clone())
					.unwrap();

				NFT::add_capsule_shard(alice_enclave, ALICE_NFT_ID).unwrap();

				assert_eq!(NFT::capsule_offchain_data(ALICE_NFT_ID).unwrap(), offchain_data);
				assert_eq!(NFT::capsules_shards_count(ALICE_NFT_ID).unwrap().len(), 1);

				// Burning the nft.
				let ok = NFT::burn_nft(alice, ALICE_NFT_ID);
				assert_ok!(ok);

				// Final state checks.
				assert!(NFT::nfts(ALICE_NFT_ID).is_none());
				assert!(NFT::capsule_offchain_data(ALICE_NFT_ID).is_none());
				assert!(NFT::capsules_shards_count(ALICE_NFT_ID).is_none());

				// Events checks.
				let event = NFTsEvent::NFTBurned { nft_id: ALICE_NFT_ID };
				let event = RuntimeEvent::NFT(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			// Burning an nft.
			let err = NFT::burn_nft(origin(ALICE), ALICE_NFT_ID);
			// Should fail because NFT was not created.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Burning an nft.
			let err = NFT::burn_nft(origin(BOB), ALICE_NFT_ID);
			// Should fail because BOB is not the owner of alice's NFT.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_burn_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set listed to true for Alice's NFT.
			let nft_state =
				NFTState::new(false, true, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Burning an nft.
			let err = NFT::burn_nft(origin(ALICE), ALICE_NFT_ID);
			// Should fail because NFT is listed for sale.
			assert_noop!(err, Error::<Test>::CannotBurnListedNFTs);
		})
	}

	#[test]
	fn cannot_burn_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set delegated to true for Alice's NFT.
			NFT::delegate_nft(origin(ALICE), ALICE_NFT_ID, Some(BOB)).unwrap();
			// Burning an nft.
			let err = NFT::burn_nft(origin(ALICE), ALICE_NFT_ID);
			// Should fail because NFT is delegated.
			assert_noop!(err, Error::<Test>::CannotBurnDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_burn_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set capsule to true for Alice's NFT.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Burning an nft.
			let err = NFT::burn_nft(origin(ALICE), ALICE_NFT_ID);
			// Should fail because NFT is capsule.
			assert_noop!(err, Error::<Test>::CannotBurnRentedNFTs);
		})
	}

	#[test]
	fn cannot_burn_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set transmission to true for Alice's NFT.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, false, false, true);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Burning an nft.
			let err = NFT::burn_nft(origin(ALICE), ALICE_NFT_ID);
			// Should fail because NFT is in transmission.
			assert_noop!(err, Error::<Test>::CannotBurnNFTsInTransmission);
		})
	}
}

mod transfer_nft {
	use super::*;

	#[test]
	fn transfer_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Transfer nft ownership from ALICE to BOB.
			let ok = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			assert_ok!(ok);

			// Final state checks.
			let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
			assert_eq!(nft.owner, BOB);
			assert_eq!(nft.creator, ALICE);

			// Events checks.
			let event =
				NFTsEvent::NFTTransferred { nft_id: ALICE_NFT_ID, sender: ALICE, recipient: BOB };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Try to transfer with an unknown NFT id.
			let err = NFT::transfer_nft(alice, INVALID_ID, BOB);
			// Should fail because NFT does not exist.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Try to transfer an unowned NFT.
			let err = NFT::transfer_nft(alice, BOB_NFT_ID, BOB);
			// Should fail because Alice is not the NFT owner.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_transfer_nfts_to_yourself() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Try to transfer to current owner.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, ALICE);
			// Should fail because alice is owner and recipient.
			assert_noop!(err, Error::<Test>::CannotTransferNFTsToYourself);
		})
	}

	#[test]
	fn cannot_transfer_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set NFT to listed.
			let nft_state =
				NFTState::new(false, true, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Try to transfer.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			// Should fail because NFT is listed.
			assert_noop!(err, Error::<Test>::CannotTransferListedNFTs);
		})
	}

	#[test]
	fn cannot_transfer_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set NFT to delegated.
			NFT::delegate_nft(origin(ALICE), ALICE_NFT_ID, Some(BOB)).unwrap();
			// Try to transfer.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			// Should fail because NFT is delegated.
			assert_noop!(err, Error::<Test>::CannotTransferDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_transfer_not_created_soulbound_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Create soulbound NFTs.
			let ok = NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, true);
			assert_ok!(ok);
			let nft_id = NFT::get_next_nft_id() - 1;
			let mut nft = NFT::get_nft(nft_id).unwrap();
			nft.creator = BOB;
			NFT::set_nft(nft_id, nft).unwrap();

			// Try to transfer.
			let err = NFT::transfer_nft(alice, nft_id, BOB);
			// Should fail because NFT is soulbound.
			assert_noop!(err, Error::<Test>::CannotTransferNotCreatedSoulboundNFTs);
		})
	}

	#[test]
	fn cannot_transfer_not_synced_secret_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set NFT to unsynced secret.
			let nft_state =
				NFTState::new(false, false, true, false, false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Try to transfer.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			// Should fail because NFT is secret and secret is not synced.
			assert_noop!(err, Error::<Test>::CannotTransferNotSyncedSecretNFTs);
		})
	}

	#[test]
	fn cannot_transfer_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set NFT to listed.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Try to transfer.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			// Should fail because NFT is listed.
			assert_noop!(err, Error::<Test>::CannotTransferRentedNFTs);
		})
	}

	#[test]
	fn cannot_transfer_not_synced_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set NFT to capsule / capsule syncing.
			let nft_state =
				NFTState::new(true, false, false, false, false, false, false, true, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Try to transfer.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			// Should fail because NFT is capsule and not synced.
			assert_noop!(err, Error::<Test>::CannotTransferNotSyncedCapsules);
		})
	}

	#[test]
	fn cannot_transfer_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set NFT to in transmission.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, false, false, true);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Try to transfer.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			// Should fail because NFT is in transmission.
			assert_noop!(err, Error::<Test>::CannotTransferNFTsInTransmission);
		})
	}
}

mod delegate_nft {
	use super::*;

	#[test]
	fn delegate_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Expected data.
			let mut expected_data = NFT::nfts(ALICE_NFT_ID).unwrap();
			expected_data.state.is_delegated = true;
			// Delegating NFT to another account.
			let ok = NFT::delegate_nft(alice, ALICE_NFT_ID, Some(BOB));
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID), Some(expected_data));
			assert_eq!(NFT::delegated_nfts(ALICE_NFT_ID), Some(BOB));

			// Events checks.
			let event = NFTsEvent::NFTDelegated { nft_id: ALICE_NFT_ID, recipient: Some(BOB) };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn delegate_nft_to_none() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Expected data.
			let mut expected_data = NFT::nfts(ALICE_NFT_ID).unwrap();
			expected_data.state.is_delegated = false;
			// Delegating NFT to another account.
			NFT::delegate_nft(alice.clone(), ALICE_NFT_ID, Some(BOB)).unwrap();
			// Delegate NFT to none.
			let ok = NFT::delegate_nft(alice, ALICE_NFT_ID, None);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID), Some(expected_data));
			assert_eq!(NFT::delegated_nfts(ALICE_NFT_ID), None);

			// Events checks.
			let event = NFTsEvent::NFTDelegated { nft_id: ALICE_NFT_ID, recipient: None };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Delegating unexisting NFT.
			let err = NFT::delegate_nft(alice, INVALID_ID, None);
			// Should fail because NFT does not exist.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Delegating unowned NFT.
			let err = NFT::delegate_nft(alice, BOB_NFT_ID, None);
			// Should fail because NFT is not owned by Alice.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_delegate_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set alice's NFT to listed.
			let nft_state =
				NFTState::new(false, true, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Delegate listed NFT.
			let err = NFT::delegate_nft(alice, ALICE_NFT_ID, None);
			// Should fail because NFT is listed.
			assert_noop!(err, Error::<Test>::CannotDelegateListedNFTs);
		})
	}

	#[test]
	fn cannot_delegate_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set alice's NFT to capsule.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Delegate capsule NFT.
			let err = NFT::delegate_nft(alice, ALICE_NFT_ID, None);
			// Should fail because NFT is capsule.
			assert_noop!(err, Error::<Test>::CannotDelegateRentedNFTs);
		})
	}

	#[test]
	fn cannot_delegate_syncing_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set alice's NFT to capsule.
			let nft_state =
				NFTState::new(false, false, false, false, false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Delegate capsule NFT.
			let err = NFT::delegate_nft(alice, ALICE_NFT_ID, None);
			// Should fail because NFT is secret and syncing.
			assert_noop!(err, Error::<Test>::CannotDelegateSyncingNFTs);
		})
	}

	#[test]
	fn cannot_delegate_syncing_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set alice's NFT to capsule / syncing.
			let nft_state =
				NFTState::new(true, false, false, false, false, false, false, true, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Delegate capsule NFT.
			let err = NFT::delegate_nft(alice, ALICE_NFT_ID, None);
			// Should fail because NFT is capsule and syncing.
			assert_noop!(err, Error::<Test>::CannotDelegateSyncingCapsules);
		})
	}

	#[test]
	fn cannot_delegate_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set alice's NFT to in transmission.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, false, false, true);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Delegate capsule NFT.
			let err = NFT::delegate_nft(alice, ALICE_NFT_ID, None);
			// Should fail because NFT is in transmission.
			assert_noop!(err, Error::<Test>::CannotDelegateNFTsInTransmission);
		})
	}
}

mod set_royalty {
	use super::*;

	#[test]
	fn set_royalty() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Expected data.
			let mut expected_data = NFT::nfts(ALICE_NFT_ID).unwrap();
			expected_data.royalty = PERCENT_80;
			// Set royalty.
			let ok = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID), Some(expected_data));

			// Events checks.
			let event = NFTsEvent::NFTRoyaltySet { nft_id: ALICE_NFT_ID, royalty: PERCENT_80 };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set royalty.
			let err = NFT::set_royalty(alice, INVALID_ID, PERCENT_80);
			// Should failt because NFT does not exist.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set royalty.
			let err = NFT::set_royalty(alice, BOB_NFT_ID, PERCENT_80);
			// Should failt because Alice is not the owner of Bob's NFT.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn not_the_creator() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);
			// Transfer Bob's NFT to Alice.
			NFT::transfer_nft(bob, BOB_NFT_ID, ALICE).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, BOB_NFT_ID, PERCENT_80);
			// Should failt because Alice is not the creator of Bob's NFT.
			assert_noop!(err, Error::<Test>::NotTheNFTCreator);
		})
	}

	#[test]
	fn cannot_set_royalty_for_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set Alice's NFT to listed.
			let nft_state =
				NFTState::new(false, true, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			// Should fail because you cannot set royalty for listed NFTs.
			assert_noop!(err, Error::<Test>::CannotSetRoyaltyForListedNFTs);
		})
	}

	#[test]
	fn cannot_set_royalty_for_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set Alice's NFT to delegated.
			NFT::delegate_nft(origin(ALICE), ALICE_NFT_ID, Some(BOB)).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			// Should fail because you cannot set royalty for delegated NFTs.
			assert_noop!(err, Error::<Test>::CannotSetRoyaltyForDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_set_royalty_for_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set Alice's NFT to capsule.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			// Should fail because you cannot set royalty for capsule NFTs.
			assert_noop!(err, Error::<Test>::CannotSetRoyaltyForRentedNFTs);
		})
	}

	#[test]
	fn cannot_set_royalty_for_syncing_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set Alice's NFT to capsule.
			let nft_state =
				NFTState::new(false, false, false, false, false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			// Should fail because you cannot set royalty for capsule NFTs.
			assert_noop!(err, Error::<Test>::CannotSetRoyaltyForSyncingNFTs);
		})
	}

	#[test]
	fn cannot_set_royalty_for_syncing_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set Alice's NFT to capsule and syncing.
			let nft_state =
				NFTState::new(true, false, false, false, false, false, false, true, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			// Should fail because you cannot set royalty for not synced capsule NFTs.
			assert_noop!(err, Error::<Test>::CannotSetRoyaltyForSyncingCapsules);
		})
	}

	#[test]
	fn cannot_set_royalty_for_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Set Alice's NFT to in transmission.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, false, false, true);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			// Should fail because you cannot set royalty for nft in transmission.
			assert_noop!(err, Error::<Test>::CannotSetRoyaltyForNFTsInTransmission);
		})
	}
}

mod set_nft_mint_fee {
	use super::*;

	#[test]
	fn set_nft_mint_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set new mint fee.
			let ok = NFT::set_nft_mint_fee(root(), 20);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nft_mint_fee(), 20);

			// Events checks.
			let event = NFTsEvent::NFTMintFeeSet { fee: 20 };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			// Try to change nft mint fee as not root.
			let err = NFT::set_nft_mint_fee(origin(ALICE), 20);
			// Should fail because Alice is not the root.
			assert_noop!(err, BadOrigin);
		})
	}
}

mod create_collection {
	use super::*;

	#[test]
	fn create_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let data = Collection::new(ALICE, BoundedVec::default(), Some(5));
			// Create collection.
			let ok = NFT::create_collection(alice, data.offchain_data.clone(), data.limit);
			assert_ok!(ok);
			let collection_id = NFT::get_next_collection_id() - 1;

			// Final state checks.
			let collection = NFT::collections(collection_id);
			assert_eq!(collection, Some(data.clone()));

			// Events checks.
			let event = NFTsEvent::CollectionCreated {
				collection_id,
				owner: data.owner,
				offchain_data: data.offchain_data,
				limit: data.limit,
			};
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_limit_is_too_high() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let collection_limit = CollectionSizeLimit::get() + 1;
			// Create NFT without a collection.
			let err = NFT::create_collection(alice, BoundedVec::default(), Some(collection_limit));
			// Should fail because max + 1 is not a valid limit.
			assert_noop!(err, Error::<Test>::CollectionLimitExceededMaximumAllowed);
		})
	}
}

mod burn_collection {
	use super::*;

	#[test]
	fn burn_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Burn collection.
			let ok = NFT::burn_collection(alice, ALICE_COLLECTION_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).is_some(), false);

			// Events checks.
			let event = NFTsEvent::CollectionBurned { collection_id: ALICE_COLLECTION_ID };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Burn invalid collection.
			let err = NFT::burn_collection(alice, INVALID_ID);
			// Should fail because collection does not exist.
			assert_noop!(err, Error::<Test>::CollectionNotFound);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Burn Bob's collection from Alice's account.
			let err = NFT::burn_collection(alice, BOB_COLLECTION_ID);
			// Should fail because Alice is not the collection owner.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}

	#[test]
	fn collection_is_not_empty() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Add Alice's NFT to her collection.
			NFT::add_nft_to_collection(alice.clone(), ALICE_NFT_ID, ALICE_COLLECTION_ID).unwrap();
			// Burn non empty collection.
			let err = NFT::burn_collection(alice, ALICE_COLLECTION_ID);
			// Should fail because collection is not empty.
			assert_noop!(err, Error::<Test>::CollectionIsNotEmpty);
		})
	}
}

mod close_collection {
	use super::*;

	#[test]
	fn close_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Close collection.
			let ok = NFT::close_collection(alice, ALICE_COLLECTION_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).unwrap().is_closed, true);

			// Events checks.
			let event = NFTsEvent::CollectionClosed { collection_id: ALICE_COLLECTION_ID };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Close invalid collection.
			let err = NFT::close_collection(alice, INVALID_ID);
			// Should fail because collection does not exist.
			assert_noop!(err, Error::<Test>::CollectionNotFound);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Close invalid collection.
			let err = NFT::close_collection(alice, BOB_COLLECTION_ID);
			// Should fail because Alice is not the owner of the collection.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}
}

mod limit_collection {
	use super::*;

	#[test]
	fn limit_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Limit collection.
			let ok = NFT::limit_collection(alice, ALICE_COLLECTION_ID, 1);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).unwrap().limit, Some(1));

			// Events checks.
			let event =
				NFTsEvent::CollectionLimited { collection_id: ALICE_COLLECTION_ID, limit: 1 };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Limit invalid collection.
			let err = NFT::limit_collection(alice, INVALID_ID, 1);
			// Should fail because the collection does not exist.
			assert_noop!(err, Error::<Test>::CollectionNotFound);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Limit unowned collection.
			let err = NFT::limit_collection(alice, BOB_COLLECTION_ID, 1);
			// Should fail because Alice is not the collection owner.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}

	#[test]
	fn collection_limit_already_set() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Limit once.
			let ok = NFT::limit_collection(alice.clone(), ALICE_COLLECTION_ID, 1);
			assert_ok!(ok);
			// Limit again.
			let err = NFT::limit_collection(alice, ALICE_COLLECTION_ID, 2);
			// Should fail because the collection limit is already set.
			assert_noop!(err, Error::<Test>::CollectionLimitAlreadySet);
		})
	}

	#[test]
	fn collection_is_closed() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Close collection.
			let ok = NFT::close_collection(alice.clone(), ALICE_COLLECTION_ID);
			assert_ok!(ok);
			// Limit.
			let err = NFT::limit_collection(alice, ALICE_COLLECTION_ID, 1);
			// Should fail because the collection is closed.
			assert_noop!(err, Error::<Test>::CollectionIsClosed);
		})
	}

	#[test]
	fn collection_nfts_number_greater_than_limit() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Add Alice's NFT to her collection.
			let ok = NFT::add_nft_to_collection(alice.clone(), ALICE_NFT_ID, ALICE_COLLECTION_ID);
			assert_ok!(ok);
			// Create a second nft for alice.
			let ok = NFT::create_nft(
				alice.clone(),
				BoundedVec::default(),
				PERCENT_100,
				Some(ALICE_COLLECTION_ID),
				false,
			);
			assert_ok!(ok);
			// Limit collection with value 1.
			let err = NFT::limit_collection(alice, ALICE_COLLECTION_ID, 1);
			// Should fail because the selected limit is lower than the number of NFTs currently in
			// the collection.
			assert_noop!(err, Error::<Test>::CollectionHasTooManyNFTs);
		})
	}

	#[test]
	fn collection_limit_is_too_high() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let limit = CollectionSizeLimit::get() + 1;
			// Limit again.
			let err = NFT::limit_collection(alice, ALICE_COLLECTION_ID, limit);
			// Should fail because the selected limit is greater than the size limit from config.
			assert_noop!(err, Error::<Test>::CollectionLimitExceededMaximumAllowed);
		})
	}
}

mod add_nft_to_collection {
	use super::*;

	#[test]
	fn add_nft_to_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let mut expected_collection = NFT::collections(ALICE_COLLECTION_ID).unwrap();
			expected_collection.nfts.try_push(ALICE_COLLECTION_ID).unwrap();
			// Add Alice's NFT to her collection.
			let ok = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, ALICE_COLLECTION_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID).unwrap().collection_id, Some(ALICE_COLLECTION_ID));
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).unwrap(), expected_collection);

			// Events checks.
			let event = NFTsEvent::NFTAddedToCollection {
				nft_id: ALICE_NFT_ID,
				collection_id: ALICE_COLLECTION_ID,
			};
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Add Alice's NFT to invalid collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, INVALID_ID);
			// Should fail because collection does not exist.
			assert_noop!(err, Error::<Test>::CollectionNotFound);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Add Alice's NFT to Bob's collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, BOB_COLLECTION_ID);
			// Should fail because collection belong to Bob.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}

	#[test]
	fn collection_is_closed() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Close Alice's collection.
			NFT::close_collection(alice.clone(), ALICE_COLLECTION_ID).unwrap();
			// Add Alice's NFT to Bob's collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, ALICE_COLLECTION_ID);
			// Should fail because collection belong to Bob.
			assert_noop!(err, Error::<Test>::CollectionIsClosed);
		})
	}

	#[test]
	fn collection_has_reached_max() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Add CollectionSizeLimit NFTs to Alice's collection.
			for _i in 0..CollectionSizeLimit::get() {
				NFT::create_nft(
					alice.clone(),
					BoundedVec::default(),
					PERCENT_0,
					Some(ALICE_COLLECTION_ID),
					false,
				)
				.unwrap();
			}
			// Add another nft to the collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, ALICE_COLLECTION_ID);
			// Should fail because collection has reached maximum value.
			assert_noop!(err, Error::<Test>::CollectionHasReachedLimit);
		})
	}

	#[test]
	fn collection_has_reached_limit() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let limit = 5;
			// Set limit to Alice's NFT.
			NFT::limit_collection(alice.clone(), ALICE_COLLECTION_ID, limit).unwrap();
			// Add CollectionSizeLimit NFTs to Alice's collection.
			for _i in 0..limit {
				NFT::create_nft(
					alice.clone(),
					BoundedVec::default(),
					PERCENT_0,
					Some(ALICE_COLLECTION_ID),
					false,
				)
				.unwrap();
			}
			// Add another nft to the collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, ALICE_COLLECTION_ID);
			// Should fail because collection has reached limit value.
			assert_noop!(err, Error::<Test>::CollectionHasReachedLimit);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Add invalid NFT to the collection.
			let err = NFT::add_nft_to_collection(alice, INVALID_ID, ALICE_COLLECTION_ID);
			// Should fail because NFT does not exist.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Add unowned NFT in collection.
			let err = NFT::add_nft_to_collection(alice, BOB_NFT_ID, ALICE_COLLECTION_ID);
			// Should fail because the NFT does not belong to Alice.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn nft_belong_to_a_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Add NFT in collection.
			let ok = NFT::add_nft_to_collection(alice.clone(), ALICE_NFT_ID, ALICE_COLLECTION_ID);
			assert_ok!(ok);
			// Create new collection.
			let ok = NFT::create_collection(alice.clone(), BoundedVec::default(), None);
			assert_ok!(ok);
			let collection_id = NFT::get_next_collection_id() - 1;
			// Add NFT to the new collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, collection_id);
			// Should fail because the NFT already belong to an other collection.
			assert_noop!(err, Error::<Test>::NFTBelongToACollection);
		})
	}
}

mod add_secret {
	use super::*;

	#[test]
	fn add_secret() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Add a secret to Alice's NFT.
			let ok = NFT::add_secret(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_ok!(ok);

			// Final state checks.
			let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
			let secret_offchain_data = NFT::secret_nfts_offchain_data(ALICE_NFT_ID).unwrap();
			assert_eq!(nft.state.is_secret, true);
			assert_eq!(nft.state.is_syncing_secret, true);
			assert_eq!(secret_offchain_data, offchain_data.clone());

			// Events checks.
			let event = NFTsEvent::SecretAddedToNFT { nft_id: ALICE_NFT_ID, offchain_data };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Add a secret to Alice's NFT.
			let err = NFT::add_secret(alice, INVALID_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Add a secret to Alice's NFT.
			let err = NFT::add_secret(alice, BOB_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_add_secret_to_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Set Alice's NFT to listed
			let nft_state =
				NFTState::new(false, true, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Add a secret to Alice's NFT.
			let err = NFT::add_secret(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotAddSecretToListedNFTs);
		})
	}

	#[test]
	fn cannot_add_secret_to_secret_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Set Alice's NFT to secret
			let nft_state =
				NFTState::new(false, false, true, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Add a secret to Alice's NFT.
			let err = NFT::add_secret(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotAddSecretToSecretNFTs);
		})
	}

	#[test]
	fn cannot_add_secret_to_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Set Alice's NFT to rented
			let nft_state =
				NFTState::new(false, false, false, false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Add a secret to Alice's NFT.
			let err = NFT::add_secret(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotAddSecretToRentedNFTs);
		})
	}

	#[test]
	fn cannot_add_secret_to_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Set Alice's NFT to delegated
			let nft_state =
				NFTState::new(false, false, false, true, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Add a secret to Alice's NFT.
			let err = NFT::add_secret(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotAddSecretToDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_add_secret_to_syncing_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Set Alice's NFT to syncing capsule
			let nft_state =
				NFTState::new(true, false, false, false, false, false, false, true, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Add a secret to Alice's NFT.
			let err = NFT::add_secret(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotAddSecretToSyncingCapsules);
		})
	}

	#[test]
	fn cannot_add_secret_to_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Set Alice's NFT to in transmission
			let nft_state =
				NFTState::new(false, false, false, false, false, false, false, false, true);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Add a secret to Alice's NFT.
			let err = NFT::add_secret(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotAddSecretToNFTsInTransmission);
		})
	}

	#[test]
	fn not_enough_balance() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();

			Balances::set_balance(RuntimeOrigin::root(), ALICE, 0, 0).unwrap();

			// Add a secret to Alice's NFT.
			let err = NFT::add_secret(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

mod create_secret_nft {
	use super::*;

	#[test]
	fn create_secret_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			let mut data =
				NFTData::new_default(ALICE, BoundedVec::default(), PERCENT_100, None, false);
			data.state.is_secret = true;
			data.state.is_syncing_secret = true;

			// Create NFT without a collection.
			NFT::create_secret_nft(
				alice,
				data.offchain_data.clone(),
				offchain_data.clone(),
				data.royalty,
				data.collection_id,
				data.state.is_soulbound,
			)
			.unwrap();
			let nft_id = NFT::get_next_nft_id() - 1;

			// Final state checks.
			let nft = NFT::nfts(nft_id);
			let secret_offchain_data = NFT::secret_nfts_offchain_data(nft_id).unwrap();
			assert_eq!(nft, Some(data.clone()));
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance - NFT::nft_mint_fee() - NFT::secret_nft_mint_fee()
			);
			assert_eq!(secret_offchain_data, offchain_data.clone());

			// Events checks.
			let event = RuntimeEvent::NFT(NFTsEvent::NFTCreated {
				nft_id,
				owner: data.owner,
				offchain_data: data.offchain_data,
				royalty: data.royalty,
				collection_id: data.collection_id,
				is_soulbound: data.state.is_soulbound,
				mint_fee: NFT::nft_mint_fee(),
			});
			System::assert_has_event(event);
			let event = RuntimeEvent::NFT(NFTsEvent::SecretAddedToNFT { nft_id, offchain_data });
			System::assert_last_event(event);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, NFT_MINT_FEE + 1)]).execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Should fail and storage should remain empty.
			let err = NFT::create_secret_nft(
				alice,
				BoundedVec::default(),
				BoundedVec::default(),
				PERCENT_0,
				None,
				false,
			);
			assert_noop!(err, Error::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn keep_alive() {
		ExtBuilder::new_build(vec![(ALICE, 2 * NFT_MINT_FEE + SECRET_NFT_MINT_FEE), (BOB, 1000)])
			.execute_with(|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_balance = Balances::free_balance(ALICE);

				// Try to create an NFT.
				let err = NFT::create_secret_nft(
					alice,
					BoundedVec::default(),
					BoundedVec::default(),
					PERCENT_0,
					None,
					false,
				);

				// Should fail because Alice's account must stay alive.
				assert_noop!(err, BalanceError::<Test>::KeepAlive);
				// Alice's balance should not have been changed
				assert_eq!(Balances::free_balance(ALICE), alice_balance);
			})
	}
}

mod add_secret_shard {
	use super::*;

	#[test]
	fn add_secret_shard() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);

				// Add a secret to Alice's NFT.
				NFT::add_secret(alice, ALICE_NFT_ID, BoundedVec::default()).unwrap();

				NFT::add_secret_shard(alice_enclave, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let shards = NFT::secret_nfts_shards_count(ALICE_NFT_ID).unwrap();
				assert_eq!(nft.state.is_secret, true);
				assert_eq!(nft.state.is_syncing_secret, true);
				assert_eq!(shards.len(), 1);
				assert!(shards.contains(&(0, ALICE)));

				// Events checks.
				let event = NFTsEvent::ShardAdded { nft_id: ALICE_NFT_ID, enclave: ALICE_ENCLAVE };
				let event = RuntimeEvent::NFT(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn add_last_secret_shard() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
				let bob_enclave: mock::RuntimeOrigin = origin(BOB_ENCLAVE);

				// Add a secret to Alice's NFT.
				NFT::add_secret(alice, ALICE_NFT_ID, BoundedVec::default()).unwrap();

				NFT::add_secret_shard(alice_enclave, ALICE_NFT_ID).unwrap();
				NFT::add_secret_shard(bob_enclave, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let shards = NFT::secret_nfts_shards_count(ALICE_NFT_ID);
				assert_eq!(nft.state.is_secret, true);
				assert_eq!(nft.state.is_syncing_secret, false);
				assert_eq!(shards, None);

				// Events checks.
				let event = RuntimeEvent::NFT(NFTsEvent::ShardAdded {
					nft_id: ALICE_NFT_ID,
					enclave: ALICE_ENCLAVE,
				});
				let final_event =
					RuntimeEvent::NFT(NFTsEvent::SecretNFTSynced { nft_id: ALICE_NFT_ID });
				System::assert_has_event(event);
				System::assert_last_event(final_event);
			},
		)
	}

	#[test]
	fn not_a_registered_enclave() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Add a secret to Alice's NFT.
			let ok = NFT::add_secret(alice.clone(), ALICE_NFT_ID, offchain_data.clone());
			assert_ok!(ok);

			let err = NFT::add_secret_shard(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::NotARegisteredEnclave);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			prepare_tee_for_tests();
			let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
			let err = NFT::add_secret_shard(alice_enclave, INVALID_ID);
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn share_not_from_valid_cluster() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
				let charlie_enclave: mock::RuntimeOrigin = origin(CHARLIE_ENCLAVE);

				// Add a secret to Alice's NFT.
				NFT::add_secret(alice, ALICE_NFT_ID, BoundedVec::default()).unwrap();

				NFT::add_secret_shard(alice_enclave, ALICE_NFT_ID).unwrap();

				let err = NFT::add_secret_shard(charlie_enclave, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::ShareNotFromValidCluster);
			},
		)
	}

	#[test]
	fn nft_is_not_secret() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();

				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);

				let err = NFT::add_secret_shard(alice_enclave, ALICE_NFT_ID);

				// Should fail because Alice's NFT is not a secret NFT.
				assert_noop!(err, Error::<Test>::NFTIsNotSecret);
			},
		)
	}

	#[test]
	fn nft_already_synced() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();

				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
				let bob_enclave: mock::RuntimeOrigin = origin(BOB_ENCLAVE);

				// Add a secret to Alice's NFT.
				NFT::add_secret(alice.clone(), ALICE_NFT_ID, BoundedVec::default()).unwrap();

				NFT::add_secret_shard(alice_enclave.clone(), ALICE_NFT_ID).unwrap();
				NFT::add_secret_shard(bob_enclave, ALICE_NFT_ID).unwrap();

				let err = NFT::add_secret_shard(alice_enclave, ALICE_NFT_ID);

				// Should fail because Alice's secret NFT is already synced.
				assert_noop!(err, Error::<Test>::NFTAlreadySynced);
			},
		)
	}

	#[test]
	fn enclave_already_added_shard() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();

				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);

				// Add a secret to Alice's NFT.
				NFT::add_secret(alice, ALICE_NFT_ID, BoundedVec::default()).unwrap();

				NFT::add_secret_shard(alice_enclave.clone(), ALICE_NFT_ID).unwrap();

				let err = NFT::add_secret_shard(alice_enclave, ALICE_NFT_ID);

				// Should fail because enclave has already added shard.
				assert_noop!(err, Error::<Test>::EnclaveAlreadyAddedShard);
			},
		)
	}
}

mod set_secret_nft_mint_fee {
	use super::*;

	#[test]
	fn set_secret_nft_mint_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set new secret nft mint fee.
			let ok = NFT::set_secret_nft_mint_fee(root(), 150);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::secret_nft_mint_fee(), 150);

			// Events checks.
			let event = NFTsEvent::SecretNFTMintFeeSet { fee: 150 };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			// Try to change secret nft mint fee as not root.
			let err = NFT::set_secret_nft_mint_fee(origin(ALICE), 150);
			// Should fail because Alice is not the root.
			assert_noop!(err, BadOrigin);
		})
	}
}

mod convert_to_capsule {
	use super::*;

	#[test]
	fn convert_to_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Convert Alice's NFT to capsule.
			let ok = NFT::convert_to_capsule(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_ok!(ok);

			// Final state checks.
			let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
			let capsule_offchain_data = NFT::capsule_offchain_data(ALICE_NFT_ID).unwrap();
			assert_eq!(nft.state.is_capsule, true);
			assert_eq!(nft.state.is_syncing_capsule, true);
			assert_eq!(capsule_offchain_data, offchain_data.clone());

			// Events checks.
			let event = NFTsEvent::NFTConvertedToCapsule { nft_id: ALICE_NFT_ID, offchain_data };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Convert Alice's NFT to capsule.
			let err = NFT::convert_to_capsule(alice, INVALID_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			// Convert Alice's NFT to capsule.
			let err = NFT::convert_to_capsule(alice, BOB_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_convert_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();

			// Change NFT State
			let nft_state =
				NFTState::new(false, true, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Convert Alice's NFT to capsule.
			let err = NFT::convert_to_capsule(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotConvertListedNFTs);
		})
	}

	#[test]
	fn cannot_convert_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();

			// Change NFT State
			let nft_state =
				NFTState::new(true, false, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Convert Alice's NFT to capsule.
			let err = NFT::convert_to_capsule(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotConvertCapsules);
		})
	}

	#[test]
	fn cannot_convert_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();

			// Change NFT State
			let nft_state =
				NFTState::new(false, false, false, false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Convert Alice's NFT to capsule.
			let err = NFT::convert_to_capsule(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotConvertRentedNFTs);
		})
	}

	#[test]
	fn cannot_convert_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();

			// Change NFT State
			let nft_state =
				NFTState::new(false, false, false, true, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Convert Alice's NFT to capsule.
			let err = NFT::convert_to_capsule(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotConvertDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_convert_syncing_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();

			// Change NFT State
			let nft_state =
				NFTState::new(false, false, false, false, false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Convert Alice's NFT to capsule.
			let err = NFT::convert_to_capsule(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotConvertSyncingNFTs);
		})
	}

	#[test]
	fn cannot_convert_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();

			// Change NFT State
			let nft_state =
				NFTState::new(false, false, false, false, false, false, false, false, true);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Convert Alice's NFT to capsule.
			let err = NFT::convert_to_capsule(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, Error::<Test>::CannotConvertNFTsInTransmission);
		})
	}

	#[test]
	fn not_enough_balance() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();

			Balances::set_balance(RuntimeOrigin::root(), ALICE, 0, 0).unwrap();

			// Add a secret to Alice's NFT.
			let err = NFT::convert_to_capsule(alice, ALICE_NFT_ID, offchain_data.clone());
			assert_noop!(err, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

mod create_capsule {
	use super::*;

	#[test]
	fn create_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();
			let mut data =
				NFTData::new_default(ALICE, BoundedVec::default(), PERCENT_100, None, false);
			data.state.is_capsule = true;
			data.state.is_syncing_capsule = true;

			// Create NFT without a collection.
			NFT::create_capsule(
				alice,
				data.offchain_data.clone(),
				offchain_data.clone(),
				data.royalty,
				data.collection_id,
				data.state.is_soulbound,
			)
			.unwrap();
			let nft_id = NFT::get_next_nft_id() - 1;

			// Final state checks.
			let nft = NFT::nfts(nft_id);
			let capsule_offchain_data = NFT::capsule_offchain_data(nft_id).unwrap();
			assert_eq!(nft, Some(data.clone()));
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance - NFT::nft_mint_fee() - NFT::capsule_mint_fee()
			);
			assert_eq!(capsule_offchain_data, offchain_data.clone());

			// Events checks.
			let event = RuntimeEvent::NFT(NFTsEvent::NFTCreated {
				nft_id,
				owner: data.owner,
				offchain_data: data.offchain_data,
				royalty: data.royalty,
				collection_id: data.collection_id,
				is_soulbound: data.state.is_soulbound,
				mint_fee: NFT::nft_mint_fee(),
			});
			System::assert_has_event(event);
			let event =
				RuntimeEvent::NFT(NFTsEvent::NFTConvertedToCapsule { nft_id, offchain_data });
			System::assert_last_event(event);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, NFT_MINT_FEE + 1)]).execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Should fail and storage should remain empty.
			let err = NFT::create_capsule(
				alice,
				BoundedVec::default(),
				BoundedVec::default(),
				PERCENT_0,
				None,
				false,
			);
			assert_noop!(err, Error::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn keep_alive() {
		ExtBuilder::new_build(vec![(ALICE, 2 * NFT_MINT_FEE + CAPSULE_MINT_FEE), (BOB, 1000)])
			.execute_with(|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_balance = Balances::free_balance(ALICE);

				// Try to create an NFT.
				let err = NFT::create_capsule(
					alice,
					BoundedVec::default(),
					BoundedVec::default(),
					PERCENT_0,
					None,
					false,
				);

				// Should fail because Alice's account must stay alive.
				assert_noop!(err, BalanceError::<Test>::KeepAlive);
				// Alice's balance should not have been changed
				assert_eq!(Balances::free_balance(ALICE), alice_balance);
			})
	}
}

// TODO: add back when we can revert capsule
// mod revert_capsule {
// 	use super::*;

// 	#[test]
// 	fn revert_capsule() {
// 		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
// 			prepare_tests();
// 			prepare_tee_for_tests();
// 			let alice: mock::RuntimeOrigin = origin(ALICE);
// 			let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
// 			// Convert Alice's NFT to capsule.
// 			let ok = NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default());
// 			assert_ok!(ok);

// 			// Change NFT State
// 			let nft_state =
// 				NFTState::new(true, false, false, false, false, false, false, true, false);
// 			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
// 			NFT::add_capsule_shard(alice_enclave, ALICE_NFT_ID).unwrap();

// 			// Revert capsule
// 			let ok = NFT::revert_capsule(alice, ALICE_NFT_ID);
// 			assert_ok!(ok);

// 			// Final state checks.
// 			let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
// 			assert_eq!(nft.state.is_capsule, false);
// 			assert_eq!(nft.state.is_syncing_capsule, false);
// 			assert_eq!(NFT::capsule_offchain_data(ALICE_NFT_ID), None);
// 			assert_eq!(NFT::capsules_shards_count(ALICE_NFT_ID), None);

// 			// Events checks.
// 			let event = NFTsEvent::CapsuleReverted { nft_id: ALICE_NFT_ID };
// 			let event = RuntimeEvent::NFT(event);
// 			System::assert_last_event(event);
// 		})
// 	}

// 	#[test]
// 	fn nft_not_found() {
// 		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
// 			prepare_tests();
// 			let alice: mock::RuntimeOrigin = origin(ALICE);

// 			// Revert capsule
// 			let err = NFT::revert_capsule(alice, INVALID_ID);
// 			assert_noop!(err, Error::<Test>::NFTNotFound);
// 		})
// 	}

// 	#[test]
// 	fn not_the_nft_owner() {
// 		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
// 			prepare_tests();
// 			let alice: mock::RuntimeOrigin = origin(ALICE);

// 			// Revert capsule
// 			let err = NFT::revert_capsule(alice, BOB_NFT_ID);
// 			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
// 		})
// 	}

// 	#[test]
// 	fn nft_is_not_capsule() {
// 		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
// 			prepare_tests();
// 			let alice: mock::RuntimeOrigin = origin(ALICE);

// 			// Revert capsule
// 			let err = NFT::revert_capsule(alice, ALICE_NFT_ID);
// 			assert_noop!(err, Error::<Test>::NFTIsNotCapsule);
// 		})
// 	}

// 	#[test]
// 	fn cannot_revert_listed_nfts() {
// 		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
// 			prepare_tests();
// 			let alice: mock::RuntimeOrigin = origin(ALICE);

// 			// Change NFT State
// 			let nft_state =
// 				NFTState::new(true, true, false, false, false, false, false, false, false);
// 			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

// 			// Revert capsule
// 			let err = NFT::revert_capsule(alice, ALICE_NFT_ID);
// 			assert_noop!(err, Error::<Test>::CannotRevertListedNFTs);
// 		})
// 	}

// 	#[test]
// 	fn cannot_revert_rented_nfts() {
// 		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
// 			prepare_tests();
// 			let alice: mock::RuntimeOrigin = origin(ALICE);

// 			// Change NFT State
// 			let nft_state =
// 				NFTState::new(true, false, false, false, false, false, true, false, false);
// 			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

// 			// Revert capsule
// 			let err = NFT::revert_capsule(alice, ALICE_NFT_ID);
// 			assert_noop!(err, Error::<Test>::CannotRevertRentedNFTs);
// 		})
// 	}

// 	#[test]
// 	fn cannot_revert_delegated_nfts() {
// 		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
// 			prepare_tests();
// 			let alice: mock::RuntimeOrigin = origin(ALICE);

// 			// Change NFT State
// 			let nft_state =
// 				NFTState::new(true, false, false, true, false, false, false, false, false);
// 			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

// 			// Revert capsule
// 			let err = NFT::revert_capsule(alice, ALICE_NFT_ID);
// 			assert_noop!(err, Error::<Test>::CannotRevertDelegatedNFTs);
// 		})
// 	}

// 	#[test]
// 	fn cannot_revert_syncing_nfts() {
// 		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
// 			prepare_tests();
// 			let alice: mock::RuntimeOrigin = origin(ALICE);

// 			// Change NFT State
// 			let nft_state =
// 				NFTState::new(true, false, false, false, false, true, false, false, false);
// 			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

// 			// Revert capsule
// 			let err = NFT::revert_capsule(alice, ALICE_NFT_ID);
// 			assert_noop!(err, Error::<Test>::CannotRevertSyncingNFTs);
// 		})
// 	}

// 	#[test]
// 	fn cannot_revert_nfts_in_transmission() {
// 		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
// 			prepare_tests();
// 			let alice: mock::RuntimeOrigin = origin(ALICE);

// 			// Change NFT State
// 			let nft_state =
// 				NFTState::new(true, false, false, false, false, false, false, false, true);
// 			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

// 			// Revert capsule
// 			let err = NFT::revert_capsule(alice, ALICE_NFT_ID);
// 			assert_noop!(err, Error::<Test>::CannotRevertNFTsInTransmission);
// 		})
// 	}
// }

mod set_capsule_offchaindata {
	use super::*;

	#[test]
	fn set_capsule_offchaindata() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Convert Alice's NFT to capsule.
			let ok = NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default());
			assert_ok!(ok);

			// Change NFT State.
			let nft_state =
				NFTState::new(true, false, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Set capsule offchain data.
			let ok = NFT::set_capsule_offchaindata(alice, ALICE_NFT_ID, BoundedVec::default());
			assert_ok!(ok);

			// Final state checks.
			let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
			assert_eq!(nft.state.is_capsule, true);
			assert_eq!(nft.state.is_syncing_capsule, false);
			assert_eq!(NFT::capsule_offchain_data(ALICE_NFT_ID), Some(BoundedVec::default()));

			// Events checks.
			let event = NFTsEvent::CapsuleOffchainDataSet {
				nft_id: ALICE_NFT_ID,
				offchain_data: BoundedVec::default(),
			};
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Set capsule offchain data.
			let err = NFT::set_capsule_offchaindata(alice, INVALID_ID, BoundedVec::default());
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Set capsule offchain data.
			let err = NFT::set_capsule_offchaindata(alice, BOB_NFT_ID, BoundedVec::default());
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn nft_is_not_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Set capsule offchain data.
			let err = NFT::set_capsule_offchaindata(alice, ALICE_NFT_ID, BoundedVec::default());
			assert_noop!(err, Error::<Test>::NFTIsNotCapsule);
		})
	}

	#[test]
	fn cannot_set_offchain_data_for_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Change NFT State
			let nft_state =
				NFTState::new(true, true, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Set capsule offchain data.
			let err = NFT::set_capsule_offchaindata(alice, ALICE_NFT_ID, BoundedVec::default());
			assert_noop!(err, Error::<Test>::CannotSetOffchainDataForListedNFTs);
		})
	}

	#[test]
	fn cannot_set_offchain_data_for_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Change NFT State
			let nft_state =
				NFTState::new(true, false, false, false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Set capsule offchain data.
			let err = NFT::set_capsule_offchaindata(alice, ALICE_NFT_ID, BoundedVec::default());
			assert_noop!(err, Error::<Test>::CannotSetOffchainDataForRentedNFTs);
		})
	}

	#[test]
	fn cannot_set_offchain_data_for_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Change NFT State
			let nft_state =
				NFTState::new(true, false, false, true, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Set capsule offchain data.
			let err = NFT::set_capsule_offchaindata(alice, ALICE_NFT_ID, BoundedVec::default());
			assert_noop!(err, Error::<Test>::CannotSetOffchainDataForDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_set_offchain_data_for_syncing_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Change NFT State
			let nft_state =
				NFTState::new(true, false, false, false, false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Set capsule offchain data.
			let err = NFT::set_capsule_offchaindata(alice, ALICE_NFT_ID, BoundedVec::default());
			assert_noop!(err, Error::<Test>::CannotSetOffchainDataForSyncingNFTs);
		})
	}

	#[test]
	fn cannot_set_offchain_data_for_syncing_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Change NFT State
			let nft_state =
				NFTState::new(true, false, false, false, false, false, false, true, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

			// Set capsule offchain data.
			let err = NFT::set_capsule_offchaindata(alice, ALICE_NFT_ID, BoundedVec::default());
			assert_noop!(err, Error::<Test>::CannotSetOffchainDataForSyncingCapsules);
		})
	}
}

mod set_capsule_mint_fee {
	use super::*;

	#[test]
	fn set_capsule_mint_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set new capsule mint fee.
			let ok = NFT::set_capsule_mint_fee(root(), 150);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::capsule_mint_fee(), 150);

			// Events checks.
			let event = NFTsEvent::CapsuleMintFeeSet { fee: 150 };
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			// Try to change capsule mint fee as not root.
			let err = NFT::set_capsule_mint_fee(origin(ALICE), 150);
			// Should fail because Alice is not the root.
			assert_noop!(err, BadOrigin);
		})
	}
}

mod add_capsule_shard {
	use super::*;

	#[test]
	fn add_capsule_shard() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);

				// Add a secret to Alice's NFT.
				NFT::convert_to_capsule(alice, ALICE_NFT_ID, BoundedVec::default()).unwrap();

				NFT::add_capsule_shard(alice_enclave, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let shards = NFT::capsules_shards_count(ALICE_NFT_ID).unwrap();
				assert_eq!(nft.state.is_capsule, true);
				assert_eq!(nft.state.is_syncing_capsule, true);
				assert_eq!(shards.len(), 1);
				assert!(shards.contains(&(0, ALICE)));

				// Events checks.
				let event =
					NFTsEvent::CapsuleShardAdded { nft_id: ALICE_NFT_ID, enclave: ALICE_ENCLAVE };
				let event = RuntimeEvent::NFT(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn add_last_secret_shard() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
				let bob_enclave: mock::RuntimeOrigin = origin(BOB_ENCLAVE);

				// Convert Alice's NFT to capsule.
				NFT::convert_to_capsule(alice, ALICE_NFT_ID, BoundedVec::default()).unwrap();

				NFT::add_capsule_shard(alice_enclave, ALICE_NFT_ID).unwrap();
				NFT::add_capsule_shard(bob_enclave, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let shards = NFT::capsules_shards_count(ALICE_NFT_ID);
				assert_eq!(nft.state.is_capsule, true);
				assert_eq!(nft.state.is_syncing_capsule, false);
				assert_eq!(shards, None);

				// Events checks.
				let event = RuntimeEvent::NFT(NFTsEvent::CapsuleShardAdded {
					nft_id: ALICE_NFT_ID,
					enclave: ALICE_ENCLAVE,
				});
				let final_event =
					RuntimeEvent::NFT(NFTsEvent::CapsuleSynced { nft_id: ALICE_NFT_ID });
				System::assert_has_event(event);
				System::assert_last_event(final_event);
			},
		)
	}

	#[test]
	fn not_a_registered_enclave() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: BoundedVec<u8, NFTOffchainDataLimit> = BoundedVec::default();

			// Convert Alice's NFT to capsule.
			let ok = NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, offchain_data.clone());
			assert_ok!(ok);

			let err = NFT::add_capsule_shard(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::NotARegisteredEnclave);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			prepare_tee_for_tests();
			let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
			let err = NFT::add_capsule_shard(alice_enclave, INVALID_ID);
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn nft_is_not_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();

				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);

				let err = NFT::add_capsule_shard(alice_enclave, ALICE_NFT_ID);

				// Should fail because Alice's NFT is not a capsule.
				assert_noop!(err, Error::<Test>::NFTIsNotCapsule);
			},
		)
	}

	#[test]
	fn nft_already_synced() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();

				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
				let bob_enclave: mock::RuntimeOrigin = origin(BOB_ENCLAVE);

				// Convert Alice's NFT to capsule.
				NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default())
					.unwrap();

				NFT::add_capsule_shard(alice_enclave.clone(), ALICE_NFT_ID).unwrap();
				NFT::add_capsule_shard(bob_enclave, ALICE_NFT_ID).unwrap();

				let err = NFT::add_capsule_shard(alice_enclave, ALICE_NFT_ID);

				// Should fail because Alice's capsule is already synced.
				assert_noop!(err, Error::<Test>::NFTAlreadySynced);
			},
		)
	}

	#[test]
	fn share_not_from_valid_cluster() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);
				let charlie_enclave: mock::RuntimeOrigin = origin(CHARLIE_ENCLAVE);

				// Convert Alice's NFT to capsule.
				NFT::convert_to_capsule(alice, ALICE_NFT_ID, BoundedVec::default()).unwrap();

				NFT::add_capsule_shard(alice_enclave, ALICE_NFT_ID).unwrap();

				let err = NFT::add_capsule_shard(charlie_enclave, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::ShareNotFromValidCluster);
			},
		)
	}

	#[test]
	fn enclave_already_added_shard() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				prepare_tee_for_tests();

				let alice: mock::RuntimeOrigin = origin(ALICE);
				let alice_enclave: mock::RuntimeOrigin = origin(ALICE_ENCLAVE);

				// Convert Alice's NFT to capsule.
				NFT::convert_to_capsule(alice, ALICE_NFT_ID, BoundedVec::default()).unwrap();

				NFT::add_capsule_shard(alice_enclave.clone(), ALICE_NFT_ID).unwrap();

				let err = NFT::add_capsule_shard(alice_enclave, ALICE_NFT_ID);

				// Should fail because enclave has already added shard.
				assert_noop!(err, Error::<Test>::EnclaveAlreadyAddedShard);
			},
		)
	}
}

mod notify_enclave_key_update {
	use super::*;

	#[test]
	fn notify_enclave_key_update() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				// Add a secret to Alice's NFT.
				NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default())
					.unwrap();

				// Change NFT State
				let nft_state =
					NFTState::new(true, false, false, false, false, false, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let ok = NFT::notify_enclave_key_update(alice, ALICE_NFT_ID);
				assert_ok!(ok);

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				assert_eq!(nft.state.is_capsule, true);
				assert_eq!(nft.state.is_syncing_capsule, true);

				// Events checks.
				let event = NFTsEvent::CapsuleKeyUpdateNotified { nft_id: ALICE_NFT_ID };
				let event = RuntimeEvent::NFT(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				let err = NFT::notify_enclave_key_update(alice, INVALID_ID);
				assert_noop!(err, Error::<Test>::NFTNotFound);
			},
		)
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				let err = NFT::notify_enclave_key_update(alice, BOB_NFT_ID);
				assert_noop!(err, Error::<Test>::NotTheNFTOwner);
			},
		)
	}

	#[test]
	fn nft_is_not_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				let err = NFT::notify_enclave_key_update(alice, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::NFTIsNotCapsule);
			},
		)
	}

	#[test]
	fn cannot_change_key_for_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				// Add a secret to Alice's NFT.
				NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default())
					.unwrap();

				// Change NFT State
				let nft_state =
					NFTState::new(true, true, false, false, false, false, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let err = NFT::notify_enclave_key_update(alice, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::CannotChangeKeyForListedNFTs);
			},
		)
	}

	#[test]
	fn cannot_change_key_for_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				// Add a secret to Alice's NFT.
				NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default())
					.unwrap();

				// Change NFT State
				let nft_state =
					NFTState::new(true, false, false, false, false, false, true, false, false);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let err = NFT::notify_enclave_key_update(alice, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::CannotChangeKeyForRentedNFTs);
			},
		)
	}

	#[test]
	fn cannot_change_key_for_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				// Add a secret to Alice's NFT.
				NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default())
					.unwrap();

				// Change NFT State
				let nft_state =
					NFTState::new(true, false, false, true, false, false, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let err = NFT::notify_enclave_key_update(alice, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::CannotChangeKeyForDelegatedNFTs);
			},
		)
	}

	#[test]
	fn cannot_change_key_for_syncing_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				// Add a secret to Alice's NFT.
				NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default())
					.unwrap();

				// Change NFT State
				let nft_state =
					NFTState::new(true, false, false, false, false, true, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let err = NFT::notify_enclave_key_update(alice, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::CannotChangeKeyForSyncingNFTs);
			},
		)
	}

	#[test]
	fn cannot_change_key_for_syncing_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				// Add a secret to Alice's NFT.
				NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default())
					.unwrap();

				// Change NFT State
				let nft_state =
					NFTState::new(true, false, false, false, false, false, false, true, false);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let err = NFT::notify_enclave_key_update(alice, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::CannotChangeKeyForSyncingCapsules);
			},
		)
	}

	#[test]
	fn cannot_change_key_for_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::RuntimeOrigin = origin(ALICE);

				// Add a secret to Alice's NFT.
				NFT::convert_to_capsule(alice.clone(), ALICE_NFT_ID, BoundedVec::default())
					.unwrap();

				// Change NFT State
				let nft_state =
					NFTState::new(true, false, false, false, false, false, false, false, true);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let err = NFT::notify_enclave_key_update(alice, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::CannotChangeKeyForNFTsInTransmission);
			},
		)
	}
}

mod set_collection_offchaindata {
	use primitives::U8BoundedVec;

	use super::*;

	#[test]
	fn set_collection_offchaindata() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: U8BoundedVec<CollectionOffchainDataLimit> =
				BoundedVec::try_from(vec![1]).unwrap().into();
			let ok =
				NFT::set_collection_offchaindata(alice, ALICE_COLLECTION_ID, offchain_data.clone());
			assert_ok!(ok);

			// Final state checks.
			let collection = NFT::collections(ALICE_COLLECTION_ID).unwrap();
			assert_eq!(collection.offchain_data, offchain_data.clone());

			// Events checks.
			let event = NFTsEvent::CollectionOffchainDataSet {
				collection_id: ALICE_COLLECTION_ID,
				offchain_data,
			};
			let event = RuntimeEvent::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: U8BoundedVec<CollectionOffchainDataLimit> =
				BoundedVec::try_from(vec![1]).unwrap().into();
			let err = NFT::set_collection_offchaindata(alice, INVALID_ID, offchain_data);
			// Should fail because collection does not exist.
			assert_noop!(err, Error::<Test>::CollectionNotFound);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let offchain_data: U8BoundedVec<CollectionOffchainDataLimit> =
				BoundedVec::try_from(vec![1]).unwrap().into();
			let err = NFT::set_collection_offchaindata(alice, BOB_COLLECTION_ID, offchain_data);
			// Should fail because alice is not owner of the collection.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}
}
