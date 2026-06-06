package com.gemwallet.android.cases.addresses

import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.Flow

interface GetAddressName {
    fun getAddressNameFlow(chain: Chain, address: String): Flow<AddressName?>
}
