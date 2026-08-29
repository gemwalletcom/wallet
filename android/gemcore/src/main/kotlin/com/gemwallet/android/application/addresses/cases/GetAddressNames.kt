package com.gemwallet.android.application.addresses.cases

import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.ChainAddress

interface GetAddressNames {
    suspend fun getAddressNames(requests: List<ChainAddress>): List<AddressName>
}
