// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Strongly typed wrappers around values in storage.
//!
//! This crate exports a macro `storage_items!` and traits describing behavior of generated
//! structs.
//!
//! Three kinds of data types are currently supported:
//!   - values
//!   - maps
//!   - lists
//!
//! # Examples:
//!
//! ```rust
//! #[macro_use]
//! extern crate srml_support;
//!
//! type AuthorityId = [u8; 32];
//! type Balance = u64;
//! pub type SessionKey = [u8; 32];
//!
//! storage_items! {
//!     // public value
//!     pub Value: b"putd_key" => SessionKey;
//!     // private map.
//!     Balances: b"private_map:" => map [AuthorityId => Balance];
//!     // private list.
//!     Authorities: b"auth:" => list [AuthorityId];
//! }
//!
//!# fn main() { }
//! ```

use codec;
use rstd::vec::Vec;
#[doc(hidden)]
pub use rstd::borrow::Borrow;
#[doc(hidden)]
pub use rstd::marker::PhantomData;

pub use substrate_metadata::{
	DecodeDifferent, StorageMetadata, StorageFunctionMetadata,
	StorageFunctionType, StorageFunctionModifier
};

/// Abstraction around storage.
pub trait Storage {
	/// true if the key exists in storage.
	fn exists(&self, key: &[u8]) -> bool;

	/// Load the bytes of a key from storage. Can panic if the type is incorrect.
	fn get<T: codec::Codec>(&self, key: &[u8]) -> Option<T>;

	/// Put a value in under a key.
	fn put<T: codec::Codec>(&self, key: &[u8], val: &T);

	/// Remove the bytes of a key from storage.
	fn kill(&self, key: &[u8]);

	/// Take a value from storage, deleting it after reading.
	fn take<T: codec::Codec>(&self, key: &[u8]) -> Option<T> {
		let value = self.get(key);
		self.kill(key);
		value
	}
}

/// A strongly-typed value kept in storage.
pub trait StorageValue<T: codec::Codec> {
	/// The type that get/take returns.
	type Query;

	/// Get the storage key.
	fn key() -> &'static [u8];

	/// true if the value is defined in storage.
	fn exists<S: Storage>(storage: &S) -> bool {
		storage.exists(Self::key())
	}

	/// Load the value from the provided storage instance.
	fn get<S: Storage>(storage: &S) -> Self::Query;

	/// Take a value from storage, removing it afterwards.
	fn take<S: Storage>(storage: &S) -> Self::Query;

	/// Store a value under this key into the provided storage instance.
	fn put<S: Storage>(val: &T, storage: &S) {
		storage.put(Self::key(), val)
	}

	/// Mutate this value
	fn mutate<F: FnOnce(&mut Self::Query), S: Storage>(f: F, storage: &S);

	/// Clear the storage value.
	fn kill<S: Storage>(storage: &S) {
		storage.kill(Self::key())
	}
}

/// A strongly-typed list in storage.
pub trait StorageList<T: codec::Codec> {
	/// Get the prefix key in storage.
	fn prefix() -> &'static [u8];

	/// Get the key used to put the length field.
	fn len_key() -> Vec<u8>;

	/// Get the storage key used to fetch a value at a given index.
	fn key_for(index: u32) -> Vec<u8>;

	/// Read out all the items.
	fn items<S: Storage>(storage: &S) -> Vec<T>;

	/// Set the current set of items.
	fn set_items<S: Storage>(items: &[T], storage: &S);

	/// Set the item at the given index.
	fn set_item<S: Storage>(index: u32, item: &T, storage: &S);

	/// Load the value at given index. Returns `None` if the index is out-of-bounds.
	fn get<S: Storage>(index: u32, storage: &S) -> Option<T>;

	/// Load the length of the list
	fn len<S: Storage>(storage: &S) -> u32;

	/// Clear the list.
	fn clear<S: Storage>(storage: &S);
}

/// A strongly-typed map in storage.
pub trait StorageMap<K: codec::Codec, V: codec::Codec> {
	/// The type that get/take returns.
	type Query;

	/// Get the prefix key in storage.
	fn prefix() -> &'static [u8];

	/// Get the storage key used to fetch a value corresponding to a specific key.
	fn key_for(x: &K) -> Vec<u8>;

	/// true if the value is defined in storage.
	fn exists<S: Storage>(key: &K, storage: &S) -> bool {
		storage.exists(&Self::key_for(key)[..])
	}

	/// Load the value associated with the given key from the map.
	fn get<S: Storage>(key: &K, storage: &S) -> Self::Query;

	/// Take the value under a key.
	fn take<S: Storage>(key: &K, storage: &S) -> Self::Query;

	/// Store a value to be associated with the given key from the map.
	fn insert<S: Storage>(key: &K, val: &V, storage: &S) {
		storage.put(&Self::key_for(key)[..], val);
	}

	/// Remove the value under a key.
	fn remove<S: Storage>(key: &K, storage: &S) {
		storage.kill(&Self::key_for(key)[..]);
	}

	/// Mutate the value under a key.
	fn mutate<F: FnOnce(&mut Self::Query), S: Storage>(key: &K, f: F, storage: &S);
}

// TODO: Remove this in favour of `decl_storage` macro.
/// Declares strongly-typed wrappers around codec-compatible types in storage.
#[macro_export]
macro_rules! storage_items {
	// simple values
	($name:ident : $key:expr => $ty:ty; $($t:tt)*) => {
		__storage_items_internal!(() () (OPTION_TYPE Option<$ty>) (get) (take) $name: $key => $ty);
		storage_items!($($t)*);
	};
	(pub $name:ident : $key:expr => $ty:ty; $($t:tt)*) => {
		__storage_items_internal!((pub) () (OPTION_TYPE Option<$ty>) (get) (take) $name: $key => $ty);
		storage_items!($($t)*);
	};
	($name:ident : $key:expr => default $ty:ty; $($t:tt)*) => {
		__storage_items_internal!(() () (RAW_TYPE $ty) (get_or_default) (take_or_default) $name: $key => $ty);
		storage_items!($($t)*);
	};
	(pub $name:ident : $key:expr => default $ty:ty; $($t:tt)*) => {
		__storage_items_internal!((pub) () (RAW_TYPE $ty) (get_or_default) (take_or_default) $name: $key => $ty);
		storage_items!($($t)*);
	};
	($name:ident : $key:expr => required $ty:ty; $($t:tt)*) => {
		__storage_items_internal!(() () (RAW_TYPE $ty) (require) (take_or_panic) $name: $key => $ty);
		storage_items!($($t)*);
	};
	(pub $name:ident : $key:expr => required $ty:ty; $($t:tt)*) => {
		__storage_items_internal!((pub) () (RAW_TYPE $ty) (require) (take_or_panic) $name: $key => $ty);
		storage_items!($($t)*);
	};

	($name:ident get($getfn:ident) : $key:expr => $ty:ty; $($t:tt)*) => {
		__storage_items_internal!(() ($getfn) (OPTION_TYPE Option<$ty>) (get) (take) $name: $key => $ty);
		storage_items!($($t)*);
	};
	(pub $name:ident get($getfn:ident) : $key:expr => $ty:ty; $($t:tt)*) => {
		__storage_items_internal!((pub) ($getfn) (OPTION_TYPE Option<$ty>) (get) (take) $name: $key => $ty);
		storage_items!($($t)*);
	};
	($name:ident get($getfn:ident) : $key:expr => default $ty:ty; $($t:tt)*) => {
		__storage_items_internal!(() ($getfn) (RAW_TYPE $ty) (get_or_default) (take_or_default) $name: $key => $ty);
		storage_items!($($t)*);
	};
	(pub $name:ident get($getfn:ident) : $key:expr => default $ty:ty; $($t:tt)*) => {
		__storage_items_internal!((pub) ($getfn) (RAW_TYPE $ty) (get_or_default) (take_or_default) $name: $key => $ty);
		storage_items!($($t)*);
	};
	($name:ident get($getfn:ident) : $key:expr => required $ty:ty; $($t:tt)*) => {
		__storage_items_internal!(() ($getfn) (RAW_TYPE $ty) (require) (take_or_panic) $name: $key => $ty);
		storage_items!($($t)*);
	};
	(pub $name:ident get($getfn:ident) : $key:expr => required $ty:ty; $($t:tt)*) => {
		__storage_items_internal!((pub) ($getfn) (RAW_TYPE $ty) (require) (take_or_panic) $name: $key => $ty);
		storage_items!($($t)*);
	};

	// maps
	($name:ident : $prefix:expr => map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!(() () (OPTION_TYPE Option<$ty>) (get) (take) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	(pub $name:ident : $prefix:expr => map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!((pub) () (OPTION_TYPE Option<$ty>) (get) (take) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	($name:ident : $prefix:expr => default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!(() () (RAW_TYPE $ty) (get_or_default) (take_or_default) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	(pub $name:ident : $prefix:expr => default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!((pub) () (RAW_TYPE $ty) (get_or_default) (take_or_default) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	($name:ident : $prefix:expr => required map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!(() () (RAW_TYPE $ty) (require) (take_or_panic) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	(pub $name:ident : $prefix:expr => required map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!((pub) () (RAW_TYPE $ty) (require) (take_or_panic) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};

	($name:ident get($getfn:ident) : $prefix:expr => map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!(() ($getfn) (OPTION_TYPE Option<$ty>) (get) (take) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	(pub $name:ident get($getfn:ident) : $prefix:expr => map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!((pub) ($getfn) (OPTION_TYPE Option<$ty>) (get) (take) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	($name:ident get($getfn:ident) : $prefix:expr => default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!(() ($getfn) (RAW_TYPE $ty) (get_or_default) (take_or_default) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	(pub $name:ident get($getfn:ident) : $prefix:expr => default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!((pub) ($getfn) (RAW_TYPE $ty) (get_or_default) (take_or_default) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	($name:ident get($getfn:ident) : $prefix:expr => required map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!(() ($getfn) (RAW_TYPE $ty) (require) (take_or_panic) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};
	(pub $name:ident get($getfn:ident) : $prefix:expr => required map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__storage_items_internal!((pub) ($getfn) (RAW_TYPE $ty) (require) (take_or_panic) $name: $prefix => map [$kty => $ty]);
		storage_items!($($t)*);
	};


	// lists
	($name:ident : $prefix:expr => list [$ty:ty]; $($t:tt)*) => {
		__storage_items_internal!(() $name: $prefix => list [$ty]);
		storage_items!($($t)*);
	};
	(pub $name:ident : $prefix:expr => list [$ty:ty]; $($t:tt)*) => {
		__storage_items_internal!((pub) $name: $prefix => list [$ty]);
		storage_items!($($t)*);
	};
	() => ()
}

#[macro_export]
#[doc(hidden)]
macro_rules! __storage_items_internal {
	// generator for values.
	(($($vis:tt)*) ($get_fn:ident) ($wraptype:ident $gettype:ty) ($getter:ident) ($taker:ident) $name:ident : $key:expr => $ty:ty) => {
		__storage_items_internal!{ ($($vis)*) () ($wraptype $gettype) ($getter) ($taker) $name : $key => $ty }
		pub fn $get_fn() -> $gettype { <$name as $crate::storage::generator::StorageValue<$ty>> :: get(&$crate::storage::RuntimeStorage) }
	};
	(($($vis:tt)*) () ($wraptype:ident $gettype:ty) ($getter:ident) ($taker:ident) $name:ident : $key:expr => $ty:ty) => {
		$($vis)* struct $name;

		impl $crate::storage::generator::StorageValue<$ty> for $name {
			type Query = $gettype;

			/// Get the storage key.
			fn key() -> &'static [u8] {
				$key
			}

			/// Load the value from the provided storage instance.
			fn get<S: $crate::GenericStorage>(storage: &S) -> Self::Query {
				storage.$getter($key)
			}

			/// Take a value from storage, removing it afterwards.
			fn take<S: $crate::GenericStorage>(storage: &S) -> Self::Query {
				storage.$taker($key)
			}

			/// Mutate this value.
			fn mutate<F: FnOnce(&mut Self::Query), S: $crate::GenericStorage>(f: F, storage: &S) {
				let mut val = <Self as $crate::storage::generator::StorageValue<$ty>>::get(storage);

				f(&mut val);

				__handle_wrap_internal!($wraptype {
					// raw type case
					<Self as $crate::storage::generator::StorageValue<$ty>>::put(&val, storage)
				} {
					// Option<> type case
					match val {
						Some(val) => <Self as $crate::storage::generator::StorageValue<$ty>>::put(&val, storage),
						None => <Self as $crate::storage::generator::StorageValue<$ty>>::kill(storage),
					}
				});
			}
		}
	};
	// generator for maps.
	(($($vis:tt)*) ($get_fn:ident) ($wraptype:ident $gettype:ty) ($getter:ident) ($taker:ident) $name:ident : $prefix:expr => map [$kty:ty => $ty:ty]) => {
		__storage_items_internal!{ ($($vis)*) () ($wraptype $gettype) ($getter) ($taker) $name : $prefix => map [$kty => $ty] }
		pub fn $get_fn<K: $crate::storage::generator::Borrow<$kty>>(key: K) -> $gettype {
			<$name as $crate::storage::generator::StorageMap<$kty, $ty>> :: get(key.borrow(), &$crate::storage::RuntimeStorage)
		}
	};
	(($($vis:tt)*) () ($wraptype:ident $gettype:ty) ($getter:ident) ($taker:ident) $name:ident : $prefix:expr => map [$kty:ty => $ty:ty]) => {
		$($vis)* struct $name;

		impl $crate::storage::generator::StorageMap<$kty, $ty> for $name {
			type Query = $gettype;

			/// Get the prefix key in storage.
			fn prefix() -> &'static [u8] {
				$prefix
			}

			/// Get the storage key used to fetch a value corresponding to a specific key.
			fn key_for(x: &$kty) -> Vec<u8> {
				let mut key = $prefix.to_vec();
				$crate::codec::Encode::encode_to(x, &mut key);
				key
			}

			/// Load the value associated with the given key from the map.
			fn get<S: $crate::GenericStorage>(key: &$kty, storage: &S) -> Self::Query {
				let key = <$name as $crate::storage::generator::StorageMap<$kty, $ty>>::key_for(key);
				storage.$getter(&key[..])
			}

			/// Take the value, reading and removing it.
			fn take<S: $crate::GenericStorage>(key: &$kty, storage: &S) -> Self::Query {
				let key = <$name as $crate::storage::generator::StorageMap<$kty, $ty>>::key_for(key);
				storage.$taker(&key[..])
			}

			/// Mutate the value under a key.
			fn mutate<F: FnOnce(&mut Self::Query), S: $crate::GenericStorage>(key: &$kty, f: F, storage: &S) {
				let mut val = <Self as $crate::storage::generator::StorageMap<$kty, $ty>>::take(key, storage);

				f(&mut val);

				__handle_wrap_internal!($wraptype {
					// raw type case
					<Self as $crate::storage::generator::StorageMap<$kty, $ty>>::insert(key, &val, storage)
				} {
					// Option<> type case
					match val {
						Some(val) => <Self as $crate::storage::generator::StorageMap<$kty, $ty>>::insert(key, &val, storage),
						None => <Self as $crate::storage::generator::StorageMap<$kty, $ty>>::remove(key, storage),
					}
				});
			}
		}
	};
	// generator for lists.
	(($($vis:tt)*) $name:ident : $prefix:expr => list [$ty:ty]) => {
		$($vis)* struct $name;

		impl $name {
			fn clear_item<S: $crate::GenericStorage>(index: u32, storage: &S) {
				if index < <$name as $crate::storage::generator::StorageList<$ty>>::len(storage) {
					storage.kill(&<$name as $crate::storage::generator::StorageList<$ty>>::key_for(index));
				}
			}

			fn set_len<S: $crate::GenericStorage>(count: u32, storage: &S) {
				(count..<$name as $crate::storage::generator::StorageList<$ty>>::len(storage)).for_each(|i| $name::clear_item(i, storage));
				storage.put(&<$name as $crate::storage::generator::StorageList<$ty>>::len_key(), &count);
			}
		}

		impl $crate::storage::generator::StorageList<$ty> for $name {
			/// Get the prefix key in storage.
			fn prefix() -> &'static [u8] {
				$prefix
			}

			/// Get the key used to put the length field.
			// TODO: concat macro should accept byte literals.
			fn len_key() -> Vec<u8> {
				let mut key = $prefix.to_vec();
				key.extend(b"len");
				key
			}

			/// Get the storage key used to fetch a value at a given index.
			fn key_for(index: u32) -> Vec<u8> {
				let mut key = $prefix.to_vec();
				$crate::codec::Encode::encode_to(&index, &mut key);
				key
			}

			/// Read out all the items.
			fn items<S: $crate::GenericStorage>(storage: &S) -> Vec<$ty> {
				(0..<$name as $crate::storage::generator::StorageList<$ty>>::len(storage))
					.map(|i| <$name as $crate::storage::generator::StorageList<$ty>>::get(i, storage).expect("all items within length are set; qed"))
					.collect()
			}

			/// Set the current set of items.
			fn set_items<S: $crate::GenericStorage>(items: &[$ty], storage: &S) {
				$name::set_len(items.len() as u32, storage);
				items.iter()
					.enumerate()
					.for_each(|(i, item)| <$name as $crate::storage::generator::StorageList<$ty>>::set_item(i as u32, item, storage));
			}

			fn set_item<S: $crate::GenericStorage>(index: u32, item: &$ty, storage: &S) {
				if index < <$name as $crate::storage::generator::StorageList<$ty>>::len(storage) {
					storage.put(&<$name as $crate::storage::generator::StorageList<$ty>>::key_for(index)[..], item);
				}
			}

			/// Load the value at given index. Returns `None` if the index is out-of-bounds.
			fn get<S: $crate::GenericStorage>(index: u32, storage: &S) -> Option<$ty> {
				storage.get(&<$name as $crate::storage::generator::StorageList<$ty>>::key_for(index)[..])
			}

			/// Load the length of the list.
			fn len<S: $crate::GenericStorage>(storage: &S) -> u32 {
				storage.get(&<$name as $crate::storage::generator::StorageList<$ty>>::len_key()).unwrap_or_default()
			}

			/// Clear the list.
			fn clear<S: $crate::GenericStorage>(storage: &S) {
				for i in 0..<$name as $crate::storage::generator::StorageList<$ty>>::len(storage) {
					$name::clear_item(i, storage);
				}

				storage.kill(&<$name as $crate::storage::generator::StorageList<$ty>>::len_key()[..])
			}
		}
	};
}

// TODO: revisit this idiom once we get `type`s in `impl`s.
/*impl<T: Trait> Module<T> {
	type Now = super::Now<T>;
}*/

#[macro_export]
macro_rules! __generate_genesis_config {
	(
		$traitinstance:ident $traittype:ident
		$($fieldname:ident : $fieldtype:ty = $fielddefault:expr ;)*
	) => {
		#[derive(Serialize, Deserialize)]
		#[cfg(feature = "std")]
		#[serde(rename_all = "camelCase")]
		#[serde(deny_unknown_fields)]
		pub struct GenesisConfig<$traitinstance: $traittype> {
			$(pub $fieldname : $fieldtype ,)*
		}

		#[cfg(feature = "std")]
		impl<$traitinstance: $traittype> Default for GenesisConfig<$traitinstance> {
			fn default() -> Self {
				GenesisConfig {
					$($fieldname : $fielddefault ,)*
				}
			}
		}

	};
}

/// Declares strongly-typed wrappers around codec-compatible types in storage.
///
/// For now we implement a convenience trait with pre-specialised associated types, one for each
/// storage item. This allows you to gain access to publicly visisible storage items from a
/// module type. Currently you must disambiguate by using `<Module as Store>::Item` rather than
/// the simpler `Module::Item`. Hopefully the rust guys with fix this soon.
#[macro_export]
macro_rules! decl_storage {
	(
		trait $storetype:ident for $modulename:ident<$traitinstance:ident: $traittype:ident> as $cratename:ident {
			$($t:tt)*
		}
	) => {
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
		trait $storetype {
			__decl_store_items!($($t)*);
		}
		impl<$traitinstance: $traittype> $storetype for $modulename<$traitinstance> {
			__impl_store_items!($traitinstance $($t)*);
		}
		impl<$traitinstance: $traittype> $modulename<$traitinstance> {
			__impl_store_fns!($traitinstance $($t)*);
			__impl_store_metadata!($cratename; $($t)*);
		}
		__decl_genesis_config_items!($traitinstance $traittype [] $($t)*);
	};
	(
		pub trait $storetype:ident for $modulename:ident<$traitinstance:ident: $traittype:ident> as $cratename:ident {
			$($t:tt)*
		}
	) => {
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
		pub trait $storetype {
			__decl_store_items!($($t)*);
		}
		impl<$traitinstance: $traittype> $storetype for $modulename<$traitinstance> {
			__impl_store_items!($traitinstance $($t)*);
		}
		impl<$traitinstance: $traittype> $modulename<$traitinstance> {
			__impl_store_fns!($traitinstance $($t)*);
		}
		__decl_genesis_config_items!($traitinstance $traittype [] $($t)*);
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! __decl_genesis_config_items {
	// simple values
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident : default $ty:ty;  $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
        };
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident : default $ty:ty; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident : $ty:ty; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident : $ty:ty; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};

	// without genesis config
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident get($getfn:ident) : default $ty:ty; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident get($getfn:ident) : default $ty:ty; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident get($getfn:ident) : $ty:ty; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident get($getfn:ident) : $ty:ty; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};

	// with genesis config
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident get($getfn:ident) : default $ty:ty : genesis = $default:expr; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* $getfn : $ty = $default ; ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident get($getfn:ident) : default $ty:ty : genesis = $default:expr; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* $getfn : $ty = $default ; ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident get($getfn:ident) : $ty:ty : genesis = $default:expr; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* $getfn : $ty = $default ; ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident get($getfn:ident) : $ty:ty : genesis = $default:expr; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* $getfn : $ty = $default ; ] $($res)* );
	};

	// maps
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident : default map [$kty:ty => $ty:ty]; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident : default map [$kty:ty => $ty:ty]; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident : map [$kty:ty => $ty:ty]; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident : map [$kty:ty => $ty:ty]; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};

	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident get($getfn:ident) : default map [$kty:ty => $ty:ty]; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident get($getfn:ident) : default map [$kty:ty => $ty:ty]; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* $name:ident get($getfn:ident) : map [$kty:ty => $ty:ty]; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};
	($traitinstance:ident $traittype:ident [$($t:tt)*] $(#[$doc:meta])* pub $name:ident get($getfn:ident) : map [$kty:ty => $ty:ty]; $($res:tt)*) => {
		__decl_genesis_config_items!($traitinstance $traittype [ $($t)* ] $($res)* );
	};

	// exit
	($traitinstance:ident $traittype:ident []) => {};
	($traitinstance:ident $traittype:ident [ $($t:tt)* ]) => {
		__generate_genesis_config!($traitinstance $traittype $($t)* );
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! __decl_storage_items {
	// simple values without getters:
	//  - pub
	//  - $default
	// so there are 4 cases here.
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident : $ty:ty; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: $ty = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident : $ty:ty = $default:expr; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: $ty = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident : $ty:ty; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: $ty = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident : $ty:ty = $default:expr; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: $ty = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};

	// simple values with getters:
	//  - pub
	//  - no_config
	//  - $default
	// so there are 8 cases here.
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : $ty:ty; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: $ty = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident no_config get($getfn:ident) : $ty:ty; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: $ty = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : $ty:ty = $default:expr; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: $ty = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident no_config get($getfn:ident) : $ty:ty = $default:expr; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: $ty = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : $ty:ty; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: $ty = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident no_config get($getfn:ident) : $ty:ty; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: $ty = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : $ty:ty = $default:expr; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: $ty = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident no_config get($getfn:ident) : $ty:ty = $default:expr; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: $ty = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};

	// maps:
	//  - pub
	//  - $default
	// so there are 4 cases here.
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident : Map<$kty:ty, $ty:ty>; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: map [$kty => $ty]);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident : Map<$kty:ty, $ty:ty> = $default:expr; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: map [$kty => $ty]);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident : Map<$kty:ty, $ty:ty>; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: map [$kty => $ty]);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident : Map<$kty:ty, $ty:ty> = $default:expr; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: map [$kty => $ty]);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};

	// maps:
	//  - pub
	//  - no_config
	//  - $default
	// so there are 8 cases here.
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : Map<$kty:ty, $ty:ty>; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: Map<$kty, $ty> = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident no_config get($getfn:ident) : Map<$kty:ty, $ty:ty>; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: Map<$kty, $ty> = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : Map<$kty:ty, $ty:ty> = $default:expr; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: Map<$kty, $ty> = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* $name:ident no_config get($getfn:ident) : Map<$kty:ty, $ty:ty> = $default:expr; $($t:tt)*) => {
		__decl_storage_item!(() ($traittype as $traitinstance) ($ty) $cratename $name: Map<$kty, $ty> = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : Map<$kty:ty, $ty:ty>; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: Map<$kty, $ty> = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident no_config get($getfn:ident) : Map<$kty:ty, $ty:ty>; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: Map<$kty, $ty> = Default::default());
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : Map<$kty:ty, $ty:ty> = $default:expr; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: Map<$kty, $ty> = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};
	($cratename:ident $traittype:ident $traitinstance:ident $(#[$doc:meta])* pub $name:ident no_config get($getfn:ident) : Map<$kty:ty, $ty:ty> = $default:expr; $($t:tt)*) => {
		__decl_storage_item!((pub) ($traittype as $traitinstance) ($ty) $cratename $name: Map<$kty, $ty> = $default);
		__decl_storage_items!($cratename $traittype $traitinstance $($t)*);
	};

	// exit
	($cratename:ident $traittype:ident $traitinstance:ident) => ()
}

#[macro_export]
#[doc(hidden)]
macro_rules! __decl_storage_item {
	// generator for values.
	(($($vis:tt)*) ($traittype:ident as $traitinstance:ident) ($gettype:ty) $cratename:ident $name:ident : $ty:ty = $default:expr) => {
		$($vis)* struct $name<$traitinstance: $traittype>($crate::storage::generator::PhantomData<$traitinstance>);

		impl<$traitinstance: $traittype> $crate::storage::generator::StorageValue<$ty> for $name<$traitinstance> {
			type Query = $gettype;

			/// Get the storage key.
			fn key() -> &'static [u8] {
				stringify!($cratename $name).as_bytes()
			}

			/// Load the value from the provided storage instance.
			fn get<S: $crate::GenericStorage>(storage: &S) -> Self::Query {
				storage.get(<$name<$traitinstance> as $crate::storage::generator::StorageValue<$ty>>::key())
					.unwrap_or_else(|| $default);
			}

			/// Take a value from storage, removing it afterwards.
			fn take<S: $crate::GenericStorage>(storage: &S) -> Self::Query {
				storage.take(<$name<$traitinstance> as $crate::storage::generator::StorageValue<$ty>>::key())
					.unwrap_or_else(|| $default);
			}

			/// Mutate the value under a key.
			fn mutate<F: FnOnce(&mut Self::Query), S: $crate::GenericStorage>(f: F, storage: &S) {
				let mut val = <Self as $crate::storage::generator::StorageValue<$ty>>::get(storage);
				f(&mut val);
				<Self as $crate::storage::generator::StorageValue<$ty>>::put(&val, storage)
			}
		}
	};
	// generator for maps.
	(($($vis:tt)*) ($traittype:ident as $traitinstance:ident) ($gettype:ty) $cratename:ident $name:ident : Map<$kty:ty, $ty:ty> = $default:expr) => {
		$($vis)* struct $name<$traitinstance: $traittype>($crate::storage::generator::PhantomData<$traitinstance>);

		impl<$traitinstance: $traittype> $crate::storage::generator::StorageMap<$kty, $ty> for $name<$traitinstance> {
			type Query = $gettype;

			/// Get the prefix key in storage.
			fn prefix() -> &'static [u8] {
				stringify!($cratename $name).as_bytes()
			}

			/// Get the storage key used to fetch a value corresponding to a specific key.
			fn key_for(x: &$kty) -> Vec<u8> {
				let mut key = <$name<$traitinstance> as $crate::storage::generator::StorageMap<$kty, $ty>>::prefix().to_vec();
				$crate::codec::Encode::encode_to(x, &mut key);
				key
			}

			/// Load the value associated with the given key from the map.
			fn get<S: $crate::GenericStorage>(key: &$kty, storage: &S) -> Self::Query {
				let key = <$name<$traitinstance> as $crate::storage::generator::StorageMap<$kty, $ty>>::key_for(key);
				storage.get(&key[..]).unwrap_or_else(|| $default);
			}

			/// Take the value, reading and removing it.
			fn take<S: $crate::GenericStorage>(key: &$kty, storage: &S) -> Self::Query {
				let key = <$name<$traitinstance> as $crate::storage::generator::StorageMap<$kty, $ty>>::key_for(key);
				storage.take(&key[..]).unwrap_or_else(|| $default);
			}

			/// Mutate the value under a key
			fn mutate<F: FnOnce(&mut Self::Query), S: $crate::GenericStorage>(key: &$kty, f: F, storage: &S) {
				let mut val = <Self as $crate::storage::generator::StorageMap<$kty, $ty>>::take(key, storage);
				f(&mut val);
				<Self as $crate::storage::generator::StorageMap<$kty, $ty>>::insert(key, &val, storage);
			}
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __decl_store_items {





	// simple values
	($(#[$doc:meta])* $name:ident : default $ty:ty; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* pub $name:ident : default $ty:ty; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* $name:ident : $ty:ty; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* pub $name:ident : $ty:ty; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};

	($(#[$doc:meta])* $name:ident get($getfn:ident) : default $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* pub $name:ident get($getfn:ident) : default $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* $name:ident get($getfn:ident) : $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* pub $name:ident get($getfn:ident) : $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};

	// maps
	($(#[$doc:meta])* $name:ident : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* pub $name:ident : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* $name:ident : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* pub $name:ident : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};

	($(#[$doc:meta])* $name:ident get($getfn:ident) : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* pub $name:ident get($getfn:ident) : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* $name:ident get($getfn:ident) : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};
	($(#[$doc:meta])* pub $name:ident get($getfn:ident) : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__decl_store_item!($name); __decl_store_items!($($t)*);
	};

	// exit
	() => ()
}

#[macro_export]
#[doc(hidden)]
macro_rules! __decl_store_item {
	($name:ident) => { type $name; }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_store_fns {
	// simple values
	($traitinstance:ident $(#[$doc:meta])* $name:ident : default $ty:ty; $($t:tt)*) => {
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident : default $ty:ty; $($t:tt)*) => {
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* $name:ident : $ty:ty; $($t:tt)*) => {
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident : $ty:ty; $($t:tt)*) => {
		__impl_store_fns!($traitinstance $($t)*);
	};

	($traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : default $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__impl_store_fn!($traitinstance $name $getfn ($ty) $ty);
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : default $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__impl_store_fn!($traitinstance $name $getfn ($ty) $ty);
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__impl_store_fn!($traitinstance $name $getfn (Option<$ty>) $ty);
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__impl_store_fn!($traitinstance $name $getfn (Option<$ty>) $ty);
		__impl_store_fns!($traitinstance $($t)*);
	};

	// maps
	($traitinstance:ident $(#[$doc:meta])* $name:ident : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* $name:ident : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_fns!($traitinstance $($t)*);
	};

	($traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_fn!($traitinstance $name $getfn ($ty) map [$kty => $ty]);
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_fn!($traitinstance $name $getfn ($ty) map [$kty => $ty]);
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_fn!($traitinstance $name $getfn (Option<$ty>) map [$kty => $ty]);
		__impl_store_fns!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_fn!($traitinstance $name $getfn (Option<$ty>) map [$kty => $ty]);
		__impl_store_fns!($traitinstance $($t)*);
	};

	// exit
	($traitinstance:ident) => ()
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_store_fn {
	($traitinstance:ident $name:ident $get_fn:ident ($gettype:ty) $ty:ty) => {
		pub fn $get_fn() -> $gettype {
			<$name<$traitinstance> as $crate::storage::generator::StorageValue<$ty>> :: get(&$crate::storage::RuntimeStorage)
		}
	};
	($traitinstance:ident $name:ident $get_fn:ident ($gettype:ty) map [$kty:ty => $ty:ty]) => {
		pub fn $get_fn<K: $crate::storage::generator::Borrow<$kty>>(key: K) -> $gettype {
			<$name<$traitinstance> as $crate::storage::generator::StorageMap<$kty, $ty>> :: get(key.borrow(), &$crate::storage::RuntimeStorage)
		}
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_store_items {
	// simple values
	($traitinstance:ident $(#[$doc:meta])* $name:ident : default $ty:ty; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident : default $ty:ty; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* $name:ident : $ty:ty; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident : $ty:ty; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};

	($traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : default $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : default $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : $ty:ty $(: genesis = $default:expr)*; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};

	// maps
	($traitinstance:ident $(#[$doc:meta])* $name:ident : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* $name:ident : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};

	($traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : default map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* $name:ident get($getfn:ident) : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};
	($traitinstance:ident $(#[$doc:meta])* pub $name:ident get($getfn:ident) : map [$kty:ty => $ty:ty]; $($t:tt)*) => {
		__impl_store_item!($name $traitinstance);
		__impl_store_items!($traitinstance $($t)*);
	};

	// exit
	($traitinstance:ident) => ()
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_store_item {
	($name:ident $traitinstance:ident) => { type $name = $name<$traitinstance>; }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_store_metadata {
	(
		$cratename:ident;
		$($rest:tt)*
	) => {
		pub fn store_metadata() -> $crate::storage::generator::StorageMetadata {
			$crate::storage::generator::StorageMetadata {
				prefix: $crate::storage::generator::DecodeDifferent::Encode(stringify!($cratename)),
				functions: __store_functions_to_metadata!(; $( $rest )* ),
			}
		}
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! __store_functions_to_metadata {
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		$name:ident : default $ty:ty;
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($ty), default
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		pub $name:ident : default $ty:ty;
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($ty), default
			);
			$( $t )*
		)
	};
	// simple values
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		$name:ident : $ty:ty;
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($ty)
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		pub $name:ident : $ty:ty;
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($ty)
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		$name:ident get($getfn:ident) :
			default $ty:ty $(: genesis = $default:expr)*;
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($ty), default
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		pub $name:ident get($getfn:ident) :
			default $ty:ty $(: genesis = $default:expr)*;
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($ty), default
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		$name:ident get($getfn:ident) : $ty:ty $(: genesis = $default:expr)*;
		$( $t:tt )*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($ty)
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		pub $name:ident get($getfn:ident) : $ty:ty $(: genesis = $default:expr)*;
		$( $t:tt )*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($ty)
			);
			$( $t )*
		)
	};

	// maps
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		$name:ident :
			default map [$kty:ty => $ty:ty];
		$( $t:tt )*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($kty, $ty), default
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		pub $name:ident :
			default map [$kty:ty => $ty:ty];
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($kty, $ty), default
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		$name:ident :
			map [$kty:ty => $ty:ty];
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($kty, $ty)
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		pub $name:ident :
			map [$kty:ty => $ty:ty];
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($kty, $ty)
			);
			$( $t )*
		)
	};

	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])* $name:ident get($getfn:ident) :
			default map [$kty:ty => $ty:ty];
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($kty, $ty), default
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		pub $name:ident get($getfn:ident) :
			default map [$kty:ty => $ty:ty];
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($kty, $ty), default
			);
			$( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		$name:ident get($getfn:ident) :
			map [$kty:ty => $ty:ty];
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($kty, $ty)
			); $( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
		$(#[doc = $doc_attr:tt])*
		pub $name:ident get($getfn:ident) :
			map [$kty:ty => $ty:ty];
		$($t:tt)*
	) => {
		__store_functions_to_metadata!(
			$( $metadata, )*
			__store_function_to_metadata!(
				$( $doc_attr ),*; $name, __store_type_to_metadata!($kty, $ty)
			); $( $t )*
		)
	};
	(
		$( $metadata:expr ),*;
	) => {
		$crate::storage::generator::DecodeDifferent::Encode(&[
			$( $metadata ),*
		])
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! __store_function_to_metadata {
	($( $fn_doc:expr ),*; $name:ident, $type:expr, default) => {
		__store_function_to_metadata!(
			$( $fn_doc ),*; $name; $type;
			$crate::storage::generator::StorageFunctionModifier::Default
		)
	};
	($( $fn_doc:expr ),*; $name:ident, $type:expr) => {
		__store_function_to_metadata!(
			$( $fn_doc ),*; $name; $type;
			$crate::storage::generator::StorageFunctionModifier::None
		)
	};
	($( $fn_doc:expr ),*; $name:ident; $type:expr; $modifier:expr) => {
		$crate::storage::generator::StorageFunctionMetadata {
			name: $crate::storage::generator::DecodeDifferent::Encode(stringify!($name)),
			modifier: $modifier,
			ty: $type,
			documentation: $crate::storage::generator::DecodeDifferent::Encode(&[ $( $fn_doc ),* ]),
		}
	}
}

#[macro_export]
#[doc(hidden)]
macro_rules! __store_type_to_metadata {
	($name:ty) => {
		$crate::storage::generator::StorageFunctionType::Plain(
			$crate::storage::generator::DecodeDifferent::Encode(stringify!($name)),
		)
	};
	($key: ty, $value:ty) => {
		$crate::storage::generator::StorageFunctionType::Map {
			key: $crate::storage::generator::DecodeDifferent::Encode(stringify!($key)),
			value: $crate::storage::generator::DecodeDifferent::Encode(stringify!($value)),
		}
	}
}

#[cfg(test)]
// Do not complain about unused `dispatch` and `dispatch_aux`.
#[allow(dead_code)]
mod tests {
	use std::collections::HashMap;
	use std::cell::RefCell;
	use codec::Codec;
	use super::*;

	impl Storage for RefCell<HashMap<Vec<u8>, Vec<u8>>> {
		fn exists(&self, key: &[u8]) -> bool {
			self.borrow_mut().get(key).is_some()
		}

		fn get<T: Codec>(&self, key: &[u8]) -> Option<T> {
			self.borrow_mut().get(key).map(|v| T::decode(&mut &v[..]).unwrap())
		}

		fn put<T: Codec>(&self, key: &[u8], val: &T) {
			self.borrow_mut().insert(key.to_owned(), val.encode());
		}

		fn kill(&self, key: &[u8]) {
			self.borrow_mut().remove(key);
		}
	}

	storage_items! {
		Value: b"a" => u32;
		List: b"b:" => list [u64];
		Map: b"c:" => map [u32 => [u8; 32]];
	}

	#[test]
	fn value() {
		let storage = RefCell::new(HashMap::new());
		assert!(Value::get(&storage).is_none());
		Value::put(&100_000, &storage);
		assert_eq!(Value::get(&storage), Some(100_000));
		Value::kill(&storage);
		assert!(Value::get(&storage).is_none());
	}

	#[test]
	fn list() {
		let storage = RefCell::new(HashMap::new());
		assert_eq!(List::len(&storage), 0);
		assert!(List::items(&storage).is_empty());

		List::set_items(&[0, 2, 4, 6, 8], &storage);
		assert_eq!(List::items(&storage), &[0, 2, 4, 6, 8]);
		assert_eq!(List::len(&storage), 5);

		List::set_item(2, &10, &storage);
		assert_eq!(List::items(&storage), &[0, 2, 10, 6, 8]);
		assert_eq!(List::len(&storage), 5);

		List::clear(&storage);
		assert_eq!(List::len(&storage), 0);
		assert!(List::items(&storage).is_empty());
	}

	#[test]
	fn map() {
		let storage = RefCell::new(HashMap::new());
		assert!(Map::get(&5, &storage).is_none());
		Map::insert(&5, &[1; 32], &storage);
		assert_eq!(Map::get(&5, &storage), Some([1; 32]));
		assert_eq!(Map::take(&5, &storage), Some([1; 32]));
		assert!(Map::get(&5, &storage).is_none());
		assert!(Map::get(&999, &storage).is_none());
	}

	pub trait Trait {
		 type Origin;
	}

	decl_module! {
		pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
	}

	decl_storage! {
		trait Store for Module<T: Trait> as TestStorage {
			/// Hello, this is doc!
			U32 : Option<u32>;
			GETU32 get(u32_getter): Option<u32>;
			pub PUBU32 : Option<u32>;
			pub GETPUBU32 get(pub_u32_getter): Option<u32>;

			U32Default : u32;
			GETU32Default get(get_u32_default): u32;
			pub PUBU32Default : u32;
			pub GETPUBU32Default get(pub_get_u32_default): u32;

			MAPU32 : Map<u32, Option<String>>;
			/// Hello, this is doc!
			/// Hello, this is doc 2!
			GETMAPU32 get(map_u32_getter): Map<u32, Option<String>>;
			pub PUBMAPU32 : Map<u32, Option<String>>;
			pub GETPUBMAPU32 get(map_pub_u32_getter): Map<u32, Option<String>>;

			MAPU32Default : Map<u32, String>;
			GETMAPU32Default get(map_get_u32_default): Map<u32, String>;
			pub PUBMAPU32Default : Map<u32, String>;
			pub GETPUBMAPU32Default get(map_pub_get_u32_default): Map<u32, String>;
		}
	}

	struct TraitImpl {}

	impl Trait for TraitImpl {
		type Origin = u32;
	}

	const EXPECTED_METADATA: StorageMetadata = StorageMetadata {
		prefix: DecodeDifferent::Encode("TestStorage"),
		functions: DecodeDifferent::Encode(&[
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("U32"),
				modifier: StorageFunctionModifier::None,
				ty: StorageFunctionType::Plain(DecodeDifferent::Encode("u32")),
				documentation: DecodeDifferent::Encode(&[ " Hello, this is doc!" ]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("GETU32"),
				modifier: StorageFunctionModifier::None,
				ty: StorageFunctionType::Plain(DecodeDifferent::Encode("u32")),
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("PUBU32"),
				modifier: StorageFunctionModifier::None,
				ty: StorageFunctionType::Plain(DecodeDifferent::Encode("u32")),
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("GETPUBU32"),
				modifier: StorageFunctionModifier::None,
				ty: StorageFunctionType::Plain(DecodeDifferent::Encode("u32")),
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("U32Default"),
				modifier: StorageFunctionModifier::Default,
				ty: StorageFunctionType::Plain(DecodeDifferent::Encode("u32")),
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("GETU32Default"),
				modifier: StorageFunctionModifier::Default,
				ty: StorageFunctionType::Plain(DecodeDifferent::Encode("u32")),
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("PUBU32Default"),
				modifier: StorageFunctionModifier::Default,
				ty: StorageFunctionType::Plain(DecodeDifferent::Encode("u32")),
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("GETPUBU32Default"),
				modifier: StorageFunctionModifier::Default,
				ty: StorageFunctionType::Plain(DecodeDifferent::Encode("u32")),
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("MAPU32"),
				modifier: StorageFunctionModifier::None,
				ty: StorageFunctionType::Map{
					key: DecodeDifferent::Encode("u32"), value: DecodeDifferent::Encode("String")
				},
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("GETMAPU32"),
				modifier: StorageFunctionModifier::None,
				ty: StorageFunctionType::Map{
					key: DecodeDifferent::Encode("u32"), value: DecodeDifferent::Encode("String")
				},
				documentation: DecodeDifferent::Encode(&[
					" Hello, this is doc!", " Hello, this is doc 2!"
				]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("PUBMAPU32"),
				modifier: StorageFunctionModifier::None,
				ty: StorageFunctionType::Map{
					key: DecodeDifferent::Encode("u32"), value: DecodeDifferent::Encode("String")
				},
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("GETPUBMAPU32"),
				modifier: StorageFunctionModifier::None,
				ty: StorageFunctionType::Map{
					key: DecodeDifferent::Encode("u32"), value: DecodeDifferent::Encode("String")
				},
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("MAPU32Default"),
				modifier: StorageFunctionModifier::Default,
				ty: StorageFunctionType::Map{
					key: DecodeDifferent::Encode("u32"), value: DecodeDifferent::Encode("String")
				},
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("GETMAPU32Default"),
				modifier: StorageFunctionModifier::Default,
				ty: StorageFunctionType::Map{
					key: DecodeDifferent::Encode("u32"), value: DecodeDifferent::Encode("String")
				},
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("PUBMAPU32Default"),
				modifier: StorageFunctionModifier::Default,
				ty: StorageFunctionType::Map{
					key: DecodeDifferent::Encode("u32"), value: DecodeDifferent::Encode("String")
				},
				documentation: DecodeDifferent::Encode(&[]),
			},
			StorageFunctionMetadata {
				name: DecodeDifferent::Encode("GETPUBMAPU32Default"),
				modifier: StorageFunctionModifier::Default,
				ty: StorageFunctionType::Map{
					key: DecodeDifferent::Encode("u32"), value: DecodeDifferent::Encode("String")
				},
				documentation: DecodeDifferent::Encode(&[]),
			}
		])
	};

	#[test]
	fn store_metadata() {
		let metadata = Module::<TraitImpl>::store_metadata();
		assert_eq!(EXPECTED_METADATA, metadata);
	}
}

#[cfg(test)]
// Do not complain about unused `dispatch` and `dispatch_aux`.
#[allow(dead_code)]
mod test2 {
	pub trait Trait {
		 type Origin;
	}

	decl_module! {
		pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
	}

	type PairOf<T> = (T, T);

	decl_storage! {
		trait Store for Module<T: Trait> as TestStorage {
			SingleDef : default u32;
			PairDef : default PairOf<u32>;
			Single : u32;
			Pair : (u32, u32);
		}
	}

	struct TraitImpl {}

	impl Trait for TraitImpl {
		type Origin = u32;
	}
}
