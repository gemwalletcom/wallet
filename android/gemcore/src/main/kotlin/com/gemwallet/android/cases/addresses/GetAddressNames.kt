package com.gemwallet.android.cases.addresses

import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.ChainAddress

interface GetAddressNames {
    suspend fun getAddressNames(requests: List<ChainAddress>): List<AddressName>
}
