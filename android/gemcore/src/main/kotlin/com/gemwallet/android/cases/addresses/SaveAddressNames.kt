package com.gemwallet.android.cases.addresses

import com.wallet.core.primitives.AddressName

interface SaveAddressNames {
    suspend fun saveAddressNames(addressNames: List<AddressName>)
}
