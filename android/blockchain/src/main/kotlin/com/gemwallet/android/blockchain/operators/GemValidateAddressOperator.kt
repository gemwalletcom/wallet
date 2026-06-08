package com.gemwallet.android.blockchain.operators

import com.gemwallet.android.ext.isValidAddress
import com.wallet.core.primitives.Chain

class GemValidateAddressOperator : ValidateAddressOperator {
    override operator fun invoke(address: String, chain: Chain): Result<Boolean> =
        Result.success(chain.isValidAddress(address))
}
