package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.toDTO
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationValidator
import uniffi.gemstone.GemGateway

class StakeService(
    private val gateway: GemGateway,
) {
    suspend fun getValidators(
        chain: Chain,
        apr: Double
    ): List<DelegationValidator> {
        val result = try {
            gateway.getStakingValidators(
                chain = chain.string,
                apr,
            )
        } catch (_: Throwable) {
            return emptyList()
        }
        return result.mapNotNull { it.toDTO() }
    }

    suspend fun getDelegationValidators(
        chain: Chain,
        address: String,
    ): List<DelegationValidator> {
        val result = try {
            gateway.getStakingDelegationValidators(
                chain = chain.string,
                address = address,
            )
        } catch (_: Throwable) {
            return emptyList()
        }
        return result.mapNotNull { it.toDTO() }
    }

    suspend fun getStakeDelegations(
        chain: Chain,
        address: String,
    ): List<DelegationBase>? {
        val result = try {
            gateway.getStakingDelegations(
                chain = chain.string,
                address,
            )
        } catch (_: Throwable) {
            return null
        }
        return result.mapNotNull { it.toDTO() }
    }
}
