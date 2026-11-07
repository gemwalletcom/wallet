package com.gemwallet.android.testkit

import uniffi.gemstone.AddressName
import uniffi.gemstone.Chain
import uniffi.gemstone.GemAddressNameUpdate
import uniffi.gemstone.GemAddressStore

class AddressStoreMock : GemAddressStore {
    override suspend fun getAddressName(chain: Chain, address: String): AddressName? = null

    override suspend fun saveAddressNames(updates: List<GemAddressNameUpdate>) = Unit

    override suspend fun deleteAddressNames(names: List<AddressName>) = Unit
}
