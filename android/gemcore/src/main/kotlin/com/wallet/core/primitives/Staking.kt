package com.wallet.core.primitives

data class RedelegateData(
    val delegation: Delegation,
    val toValidator: DelegationValidator,
)

sealed class StakeType {
    data class Stake(val content: DelegationValidator) : StakeType()
    data class Unstake(val content: Delegation) : StakeType()
    data class Redelegate(val content: RedelegateData) : StakeType()
    data class Rewards(val content: List<DelegationValidator>) : StakeType()
    data class Withdraw(val content: Delegation) : StakeType()
    data class Freeze(val content: Resource) : StakeType()
    data class Unfreeze(val content: Resource) : StakeType()
}
