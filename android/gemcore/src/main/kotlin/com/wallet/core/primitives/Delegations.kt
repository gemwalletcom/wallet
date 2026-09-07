package com.wallet.core.primitives

import java.math.BigInteger

data class DelegationBase(
    val assetId: AssetId,
    val state: DelegationState,
    val balance: BigInteger,
    val shares: BigInteger,
    val rewards: BigInteger,
    val completionDate: Long? = null,
    val delegationId: String,
    val validatorId: String,
)

data class Delegation(
    val base: DelegationBase,
    val validator: DelegationValidator,
    val price: Price? = null,
)
