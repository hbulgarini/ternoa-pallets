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

use frame_support::weights::{RefTimeWeight, Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn create_nft(s: u32) -> Weight;
	fn burn_nft(s: u32) -> Weight;
	fn transfer_nft() -> Weight;
	fn delegate_nft() -> Weight;
	fn set_royalty() -> Weight;
	fn set_nft_mint_fee() -> Weight;
	fn create_collection() -> Weight;
	fn burn_collection() -> Weight;
	fn close_collection() -> Weight;
	fn limit_collection() -> Weight;
	fn add_nft_to_collection(s: u32) -> Weight;
}

/// Weight functions for `ternoa_nft`.
pub struct TernoaWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for TernoaWeight<T> {
	// Storage: NFT NftMintFee (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: NFT Collections (r:1 w:1)
	// Storage: NFT NextNFTId (r:1 w:1)
	// Storage: NFT Nfts (r:0 w:1)
	fn create_nft(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: NFT Collections (r:1 w:1)
	fn burn_nft(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT Nfts (r:1 w:1)
	fn transfer_nft() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: NFT DelegatedNFTs (r:0 w:1)
	fn delegate_nft() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT Nfts (r:1 w:1)
	fn set_royalty() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT NftMintFee (r:0 w:1)
	fn set_nft_mint_fee() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT NextCollectionId (r:1 w:1)
	// Storage: NFT Collections (r:0 w:1)
	fn create_collection() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT Collections (r:1 w:1)
	fn burn_collection() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT Collections (r:1 w:1)
	fn close_collection() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT Collections (r:1 w:1)
	fn limit_collection() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: NFT Collections (r:1 w:1)
	// Storage: NFT Nfts (r:1 w:1)
	fn add_nft_to_collection(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
}
