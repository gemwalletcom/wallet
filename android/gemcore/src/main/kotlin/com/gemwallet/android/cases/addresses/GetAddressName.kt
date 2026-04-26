package com.gemwallet.android.cases.addresses

import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain

interface GetAddressName {
    suspend fun getAddressName(chain: Chain, address: String): AddressName?
}
